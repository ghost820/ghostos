#![allow(clippy::missing_safety_doc)]

use core::ptr;

use alloc::vec::Vec;
use bootloader_api::info::{FrameBuffer, MemoryRegionKind, MemoryRegions};
use conquer_once::spin::OnceCell;
use linked_list_allocator::LockedHeap;
use spin::Mutex;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
    Translate,
};
use x86_64::{PhysAddr, VirtAddr};

pub const KERNEL_SPACE_ADDR: usize = 0xffff_8000_0000_0000;
pub const KERNEL_STACK_SIZE: usize = 64 * 1024;

pub const HEAP_ADDR: usize = 0xffff_c000_0000_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

pub const USER_SPACE_ADDR: usize = 0x0000_0000_0000_1000;
pub const USER_SPACE_SIZE: usize = 0x0000_7fff_ffff_f000;

pub const USER_FRAMEBUFFER_MAPPING_ADDR: usize = 0x0000_1000_0000_0000;

#[repr(align(16))]
struct KernelStack(#[allow(dead_code)] [u8; KERNEL_STACK_SIZE]);

static mut KERNEL_STACK: KernelStack = KernelStack([0; KERNEL_STACK_SIZE]);

pub fn kernel_stack_top() -> VirtAddr {
    let start = VirtAddr::from_ptr(&raw const KERNEL_STACK);

    start + KERNEL_STACK_SIZE as u64
}

struct MemoryManager {
    mapper: Mutex<OffsetPageTable<'static>>,
    frame_allocator: Mutex<PhysicalFrameAllocator>,
}

impl MemoryManager {
    fn with_mapper_and_frame_allocator<R>(
        &self,
        f: impl FnOnce(&mut OffsetPageTable<'static>, &mut PhysicalFrameAllocator) -> R,
    ) -> R {
        let mut mapper = self.mapper.lock();
        let mut frame_allocator = self.frame_allocator.lock();

        f(&mut mapper, &mut frame_allocator)
    }

    fn with_frame_allocator<R>(&self, f: impl FnOnce(&mut PhysicalFrameAllocator) -> R) -> R {
        let mut frame_allocator = self.frame_allocator.lock();

        f(&mut frame_allocator)
    }
}

static MEMORY_MANAGER: OnceCell<MemoryManager> = OnceCell::uninit();

fn memory_manager() -> &'static MemoryManager {
    MEMORY_MANAGER
        .try_get()
        .expect("memory manager not initialized")
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

static PHYSICAL_MEMORY_OFFSET: OnceCell<VirtAddr> = OnceCell::uninit();

/// After init we can use Box, Rc, Vec, String etc.
pub fn init(
    memory_regions: &'static MemoryRegions,
    physical_memory_offset: VirtAddr,
) -> Result<(), MapToError<Size4KiB>> {
    MEMORY_MANAGER
        .try_init_once(|| MemoryManager {
            mapper: Mutex::new(unsafe { get_offset_page_table(physical_memory_offset) }),
            frame_allocator: Mutex::new(unsafe { PhysicalFrameAllocator::new(memory_regions) }),
        })
        .expect("memory manager already initialized");

    PHYSICAL_MEMORY_OFFSET
        .try_init_once(|| physical_memory_offset)
        .expect("physical memory offset already initialized");

    memory_manager().with_mapper_and_frame_allocator(
        |mapper, frame_allocator| -> Result<(), MapToError<Size4KiB>> {
            for entry in mapper.level_4_table_mut().iter_mut().take(256) {
                entry.set_unused();
            }

            x86_64::instructions::tlb::flush_all();

            let page_range = {
                let heap_start = VirtAddr::new(HEAP_ADDR as u64);
                let heap_end = heap_start + (HEAP_SIZE as u64) - 1;
                let heap_start_page = Page::containing_address(heap_start);
                let heap_end_page = Page::containing_address(heap_end);
                Page::range_inclusive(heap_start_page, heap_end_page)
            };

            for page in page_range {
                let frame = frame_allocator
                    .allocate_frame()
                    .ok_or(MapToError::FrameAllocationFailed)?;

                let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

                unsafe {
                    mapper.map_to(page, frame, flags, frame_allocator)?.flush();
                }
            }

            Ok(())
        },
    )?;

    unsafe {
        ALLOCATOR.lock().init(HEAP_ADDR, HEAP_SIZE);
    }

    Ok(())
}

pub fn phys_to_virt(address: PhysAddr) -> VirtAddr {
    let offset = *PHYSICAL_MEMORY_OFFSET
        .try_get()
        .expect("physical memory offset not initialized");

    let address = offset
        .as_u64()
        .checked_add(address.as_u64())
        .expect("physical to virtual address overflow");

    VirtAddr::try_new(address).expect("non-canonical virtual address")
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserPage(Page<Size4KiB>);

impl UserPage {
    pub fn containing_address(address: VirtAddr) -> Self {
        let page = Page::containing_address(address);
        let page_address = page.start_address().as_u64() as usize;

        assert!(
            page_address
                .checked_sub(USER_SPACE_ADDR)
                .is_some_and(|offset| offset < USER_SPACE_SIZE),
            "page is outside user address space"
        );

        Self(page)
    }

    pub fn page(self) -> Page<Size4KiB> {
        self.0
    }

    pub fn start_address(self) -> VirtAddr {
        self.0.start_address()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserPageAccess {
    ReadOnly,
    ReadWrite,
    ReadExecute,
}

impl UserPageAccess {
    fn flags(self) -> PageTableFlags {
        let flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;

        match self {
            Self::ReadOnly => flags | PageTableFlags::NO_EXECUTE,
            Self::ReadWrite => flags | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
            Self::ReadExecute => flags,
        }
    }
}

pub struct AddressSpace {
    level_4_frame: PhysFrame<Size4KiB>,
}

impl AddressSpace {
    pub fn new(plan: MappingTransaction) -> Option<Self> {
        const HIGHER_HALF_P4_START_INDEX: usize = 256;

        let physical_memory_offset = *PHYSICAL_MEMORY_OFFSET
            .try_get()
            .expect("physical memory offset not initialized");

        let required_frames = plan
            .frame_estimate()
            .total()
            .checked_add(1)
            .expect("address space frame count overflow");

        memory_manager().with_mapper_and_frame_allocator(|mapper, frame_allocator| {
            let mut reservation = frame_allocator.reserve(required_frames)?;

            let level_4_frame = reservation.allocate_zeroed_frame();
            let mut address_space = Self { level_4_frame };

            // TODO: Kernel space is mapped to user space once and never updated
            for (destination, source) in address_space
                .level_4_table_mut()
                .iter_mut()
                .skip(HIGHER_HALF_P4_START_INDEX)
                .zip(
                    mapper
                        .level_4_table()
                        .iter()
                        .skip(HIGHER_HALF_P4_START_INDEX),
                )
            {
                assert!(
                    !source.flags().contains(PageTableFlags::USER_ACCESSIBLE),
                    "kernel PML4 entry is user accessible"
                );

                *destination = source.clone();
            }

            let mut user_mapper = unsafe {
                OffsetPageTable::new(address_space.level_4_table_mut(), physical_memory_offset)
            };

            for request in plan.requests {
                for page in
                    Page::range_inclusive(request.start_page.page(), request.end_page.page())
                {
                    let frame = reservation.allocate_zeroed_frame();

                    unsafe {
                        user_mapper
                            .map_to(page, frame, request.access.flags(), &mut reservation)
                            .expect("mapping failed after reserving required frames")
                            .ignore();
                    }
                }
            }

            assert_eq!(
                reservation.remaining, 0,
                "address space frame estimate was incorrect"
            );

            Some(address_space)
        })
    }

    // TODO: Thread safety?
    pub fn activate(&self) {
        use x86_64::registers::control::Cr3;

        let (_, flags) = Cr3::read();

        unsafe {
            Cr3::write(self.level_4_frame, flags);
        }
    }

    pub fn write_user_memory(&mut self, address: VirtAddr, mut data: &[u8]) {
        let start = address.as_u64() as usize;
        let end = start
            .checked_add(data.len())
            .expect("user memory write range overflow");

        assert!(
            start >= USER_SPACE_ADDR && end <= USER_SPACE_ADDR + USER_SPACE_SIZE,
            "attempted to write outside user address space"
        );

        let physical_memory_offset = *PHYSICAL_MEMORY_OFFSET
            .try_get()
            .expect("physical memory offset not initialized");

        let mapper =
            unsafe { OffsetPageTable::new(self.level_4_table_mut(), physical_memory_offset) };

        let mut current_address = address;

        while !data.is_empty() {
            let physical_address = mapper
                .translate_addr(current_address)
                .expect("attempted to write to unmapped user memory");

            let page_offset = current_address.as_u64() as usize % 4096;
            let write_size = data.len().min(4096 - page_offset);
            let destination = phys_to_virt(physical_address);

            unsafe {
                ptr::copy_nonoverlapping(data.as_ptr(), destination.as_mut_ptr::<u8>(), write_size);
            }

            current_address += write_size as u64;
            data = &data[write_size..];
        }
    }

    pub(crate) fn map_framebuffer(&mut self, framebuffer: &FrameBuffer) -> Option<VirtAddr> {
        let framebuffer_start = VirtAddr::from_ptr(framebuffer.buffer().as_ptr());
        let framebuffer_len = framebuffer.info().byte_len;

        assert!(framebuffer_len != 0, "framebuffer is empty");

        let source_start_page = Page::<Size4KiB>::containing_address(framebuffer_start);
        let source_end_address = framebuffer_start
            + u64::try_from(framebuffer_len - 1).expect("framebuffer size does not fit u64");
        let source_end_page = Page::<Size4KiB>::containing_address(source_end_address);

        let page_offset = framebuffer_start.as_u64() - source_start_page.start_address().as_u64();

        let user_start = VirtAddr::new(USER_FRAMEBUFFER_MAPPING_ADDR as u64);

        memory_manager().with_mapper_and_frame_allocator(|mapper, frame_allocator| {
            let physical_memory_offset = *PHYSICAL_MEMORY_OFFSET
                .try_get()
                .expect("physical memory offset not initialized");

            let mut user_mapper =
                unsafe { OffsetPageTable::new(self.level_4_table_mut(), physical_memory_offset) };

            for (index, source_page) in
                Page::range_inclusive(source_start_page, source_end_page).enumerate()
            {
                let physical_address = mapper
                    .translate_addr(source_page.start_address())
                    .expect("framebuffer virtual address is not mapped");

                let frame = PhysFrame::containing_address(physical_address);

                let offset = u64::try_from(index)
                    .expect("framebuffer page index does not fit u64")
                    .checked_mul(4096)
                    .expect("framebuffer mapping offset overflow");

                let user_page = UserPage::containing_address(user_start + offset).page();

                unsafe {
                    user_mapper
                        .map_to(
                            user_page,
                            frame,
                            UserPageAccess::ReadWrite.flags(),
                            frame_allocator,
                        )
                        .ok()?
                        .ignore();
                }
            }

            Some(user_start + page_offset)
        })
    }

    fn level_4_table_mut(&mut self) -> &mut PageTable {
        let address = phys_to_virt(self.level_4_frame.start_address());

        unsafe { &mut *address.as_mut_ptr() }
    }
}

#[derive(Clone, Copy)]
struct MappingRequest {
    start_page: UserPage,
    end_page: UserPage,
    access: UserPageAccess,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FrameEstimate {
    data_frames: usize,
    page_table_frames: usize,
}

impl FrameEstimate {
    fn total(self) -> usize {
        self.data_frames
            .checked_add(self.page_table_frames)
            .expect("frame estimate overflow")
    }
}

#[must_use]
pub struct MappingTransaction {
    requests: Vec<MappingRequest>,
}

impl MappingTransaction {
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    pub fn map_user_page(self, page: UserPage, access: UserPageAccess) -> Self {
        self.map_user_pages(page, page, access)
    }

    pub fn map_user_pages(
        mut self,
        start_page: UserPage,
        end_page: UserPage,
        access: UserPageAccess,
    ) -> Self {
        let start_address = start_page.start_address();
        let end_address = end_page.start_address();

        assert!(
            start_address <= end_address,
            "mapping range starts after it ends"
        );

        let insert_index = self
            .requests
            .partition_point(|request| request.start_page.start_address() < start_address);

        if insert_index > 0 {
            assert!(
                self.requests[insert_index - 1].end_page.start_address() < start_address,
                "mapping transaction contains overlapping ranges"
            );
        }

        if let Some(next) = self.requests.get(insert_index) {
            assert!(
                end_address < next.start_page.start_address(),
                "mapping transaction contains overlapping ranges"
            );
        }

        self.requests.insert(
            insert_index,
            MappingRequest {
                start_page,
                end_page,
                access,
            },
        );

        self
    }

    fn frame_estimate(&self) -> FrameEstimate {
        const PAGE_SIZE: usize = 4096;
        const LEVEL_1_TABLE_COVERAGE: usize = 2 * 1024 * 1024;
        const LEVEL_2_TABLE_COVERAGE: usize = 1024 * 1024 * 1024;
        const LEVEL_3_TABLE_COVERAGE: usize = 512 * 1024 * 1024 * 1024;

        let mut data_frames = 0usize;
        let mut level_1_tables = 0usize;
        let mut level_2_tables = 0usize;
        let mut level_3_tables = 0usize;

        let mut previous_level_1_region = None;
        let mut previous_level_2_region = None;
        let mut previous_level_3_region = None;

        for request in &self.requests {
            let start_address = request.start_page.start_address().as_u64() as usize;
            let end_address = request.end_page.start_address().as_u64() as usize;

            let request_data_frames = end_address
                .checked_sub(start_address)
                .map(|size| size / PAGE_SIZE)
                .and_then(|count| count.checked_add(1))
                .expect("mapping data frame count overflow");

            data_frames = data_frames
                .checked_add(request_data_frames)
                .expect("mapping data frame count overflow");

            level_1_tables = level_1_tables
                .checked_add(Self::count_new_regions(
                    start_address,
                    end_address,
                    LEVEL_1_TABLE_COVERAGE,
                    &mut previous_level_1_region,
                ))
                .expect("level 1 table count overflow");

            level_2_tables = level_2_tables
                .checked_add(Self::count_new_regions(
                    start_address,
                    end_address,
                    LEVEL_2_TABLE_COVERAGE,
                    &mut previous_level_2_region,
                ))
                .expect("level 2 table count overflow");

            level_3_tables = level_3_tables
                .checked_add(Self::count_new_regions(
                    start_address,
                    end_address,
                    LEVEL_3_TABLE_COVERAGE,
                    &mut previous_level_3_region,
                ))
                .expect("level 3 table count overflow");
        }

        let page_table_frames = level_1_tables
            .checked_add(level_2_tables)
            .and_then(|count| count.checked_add(level_3_tables))
            .expect("mapping page table frame count overflow");

        FrameEstimate {
            data_frames,
            page_table_frames,
        }
    }

    fn count_new_regions(
        start_address: usize,
        end_address: usize,
        region_size: usize,
        previous_end_region: &mut Option<usize>,
    ) -> usize {
        let start_region = start_address / region_size;
        let end_region = end_address / region_size;

        let region_count = end_region
            .checked_sub(start_region)
            .and_then(|count| count.checked_add(1))
            .expect("mapping region count overflow");

        let already_counted = previous_end_region.map_or(0, |previous_end| {
            if previous_end < start_region {
                0
            } else {
                previous_end
                    .min(end_region)
                    .checked_sub(start_region)
                    .and_then(|count| count.checked_add(1))
                    .expect("mapping region overlap count overflow")
            }
        });

        *previous_end_region = Some(end_region);

        region_count - already_counted
    }
}

impl Default for MappingTransaction {
    fn default() -> Self {
        Self::new()
    }
}

/// Use OffsetPageTable.translate_addr() to translate a virtual address to a physical address.
/// Use OffsetPageTable.map_to() + flush() to map a virtual page to a physical frame.
///
/// This function should only be called once.
unsafe fn get_offset_page_table(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    use x86_64::registers::control::Cr3;

    let (pml4, _) = Cr3::read();

    let phys = pml4.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { OffsetPageTable::new(&mut *page_table_ptr, physical_memory_offset) }
}

// TODO: Optimize this
struct PhysicalFrameAllocator {
    memory_regions: &'static MemoryRegions,
    next: usize,
}

impl PhysicalFrameAllocator {
    unsafe fn new(memory_regions: &'static MemoryRegions) -> Self {
        Self {
            memory_regions,
            next: 0,
        }
    }

    fn allocate_zeroed_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.allocate_frame()?;
        let address = phys_to_virt(frame.start_address());

        unsafe {
            ptr::write_bytes(address.as_mut_ptr::<u8>(), 0, 4096);
        }

        Some(frame)
    }

    fn reserve(&mut self, count: usize) -> Option<FrameReservation<'_>> {
        if self.remaining_frames() < count {
            return None;
        }

        Some(FrameReservation {
            allocator: self,
            remaining: count,
        })
    }

    fn remaining_frames(&self) -> usize {
        self.usable_frames()
            .count()
            .checked_sub(self.next)
            .expect("invalid physical frame allocator state")
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_regions.iter();
        let usable_regions = regions.filter(|r| r.kind == MemoryRegionKind::Usable);
        let addr_ranges = usable_regions.map(|r| r.start..r.end);
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for PhysicalFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next)?;
        self.next += 1;
        Some(frame)
    }
}

struct FrameReservation<'a> {
    allocator: &'a mut PhysicalFrameAllocator,
    remaining: usize,
}

impl FrameReservation<'_> {
    fn allocate_zeroed_frame(&mut self) -> PhysFrame<Size4KiB> {
        let frame = self
            .allocate_frame()
            .expect("reserved frame allocation failed");

        let address = phys_to_virt(frame.start_address());

        unsafe {
            ptr::write_bytes(address.as_mut_ptr::<u8>(), 0, 4096);
        }

        frame
    }
}

unsafe impl FrameAllocator<Size4KiB> for FrameReservation<'_> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        assert!(self.remaining > 0, "frame reservation exhausted");

        let frame = self
            .allocator
            .allocate_frame()
            .expect("reserved frame allocation failed");

        self.remaining -= 1;

        Some(frame)
    }
}
