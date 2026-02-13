#![allow(clippy::missing_safety_doc)]

use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

/// Use OffsetPageTable.translate_addr() to translate a virtual address to a physical address.
/// Use OffsetPageTable.map_to() + flush() to map a virtual page to a physical frame.
///
/// This function should only be called once.
pub unsafe fn get_offset_page_table(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    use x86_64::registers::control::Cr3;

    let (pml4, _) = Cr3::read();

    let phys = pml4.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { OffsetPageTable::new(&mut *page_table_ptr, physical_memory_offset) }
}

pub struct PhysicalFrameAllocator {
    memory_map: &'static bootloader::bootinfo::MemoryMap,
    next: usize,
}

impl PhysicalFrameAllocator {
    pub unsafe fn new(memory_map: &'static bootloader::bootinfo::MemoryMap) -> Self {
        Self {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        use bootloader::bootinfo::MemoryRegionType;

        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for PhysicalFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
