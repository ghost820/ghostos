use alloc::vec::Vec;
use elf::abi::{EM_X86_64, ET_EXEC, EV_CURRENT, PF_W, PF_X, PT_LOAD};
use elf::endian::LittleEndian;
use elf::file::Class;
use elf::{ElfBytes, ParseError};
use x86_64::VirtAddr;
use x86_64::structures::paging::{Page, Size4KiB};

use crate::memory::{USER_FRAMEBUFFER_MAPPING_ADDR, UserPageAccess};

pub const USER_EXECUTABLE_ADDR: usize = 0x0000_0000_0040_0000;

pub const USER_HEAP_ADDR: usize = 0x0000_0001_0000_0000;

pub const USER_EXECUTABLE_SIZE: usize = USER_HEAP_ADDR - USER_EXECUTABLE_ADDR;

pub const USER_STACK_TOP: usize = 0x0000_7fff_ffff_f000;
pub const USER_STACK_SIZE: usize = 16 * 4096;
pub const USER_STACK_ADDR: usize = USER_STACK_TOP - USER_STACK_SIZE;

pub const USER_STACK_GUARD_SIZE: usize = 4096;
pub const USER_STACK_GUARD_ADDR: usize = USER_STACK_ADDR - USER_STACK_GUARD_SIZE;

pub const USER_HEAP_LIMIT: usize = USER_FRAMEBUFFER_MAPPING_ADDR;

pub fn is_executable_address(address: usize) -> bool {
    address
        .checked_sub(USER_EXECUTABLE_ADDR)
        .is_some_and(|offset| offset < USER_EXECUTABLE_SIZE)
}

pub struct LoadSegment<'a> {
    pub start_address: VirtAddr,
    pub end_address: VirtAddr,
    pub memory_size: usize,
    pub access: UserPageAccess,
    pub data: &'a [u8],
}

#[derive(Debug)]
pub enum ExecutableImageError {
    Parse(ParseError),
    UnsupportedClass,
    UnsupportedVersion,
    UnsupportedMachine,
    UnsupportedType,
    MissingProgramHeaders,
    InvalidEntryPoint,
    InvalidLoadSegment,
    OverlappingLoadSegments,
}

pub struct ExecutableImage<'a> {
    pub elf: ElfBytes<'a, LittleEndian>,
}

impl<'a> ExecutableImage<'a> {
    pub fn new(image: &'a [u8]) -> Result<Self, ExecutableImageError> {
        let elf =
            ElfBytes::<LittleEndian>::minimal_parse(image).map_err(ExecutableImageError::Parse)?;

        if elf.ehdr.class != Class::ELF64 {
            return Err(ExecutableImageError::UnsupportedClass);
        }

        if elf.ehdr.version != u32::from(EV_CURRENT) {
            return Err(ExecutableImageError::UnsupportedVersion);
        }

        if elf.ehdr.e_machine != EM_X86_64 {
            return Err(ExecutableImageError::UnsupportedMachine);
        }

        if elf.ehdr.e_type != ET_EXEC {
            return Err(ExecutableImageError::UnsupportedType);
        }

        if elf.ehdr.e_phoff == 0 || elf.ehdr.e_phnum == 0 {
            return Err(ExecutableImageError::MissingProgramHeaders);
        }

        let entry_point = usize::try_from(elf.ehdr.e_entry)
            .map_err(|_| ExecutableImageError::InvalidEntryPoint)?;

        if !is_executable_address(entry_point) {
            return Err(ExecutableImageError::InvalidEntryPoint);
        }

        // TODO: Validate load segments and other stuff

        Ok(Self { elf })
    }

    pub fn entry_point(&self) -> VirtAddr {
        VirtAddr::new(self.elf.ehdr.e_entry)
    }

    pub fn load_segments(&self) -> Result<Vec<LoadSegment<'a>>, ExecutableImageError> {
        let segments = self
            .elf
            .segments()
            .expect("validated ELF is missing program headers");

        let mut load_segments = Vec::new();

        for segment in segments.iter().filter(|segment| segment.p_type == PT_LOAD) {
            if segment.p_memsz == 0 {
                continue;
            }

            if segment.p_filesz > segment.p_memsz {
                return Err(ExecutableImageError::InvalidLoadSegment);
            }

            let start_address = VirtAddr::try_new(segment.p_vaddr)
                .map_err(|_| ExecutableImageError::InvalidLoadSegment)?;

            let memory_size = usize::try_from(segment.p_memsz)
                .map_err(|_| ExecutableImageError::InvalidLoadSegment)?;

            let end_address = start_address
                .as_u64()
                .checked_add(segment.p_memsz - 1)
                .and_then(|address| VirtAddr::try_new(address).ok())
                .ok_or(ExecutableImageError::InvalidLoadSegment)?;

            let start = start_address.as_u64() as usize;
            let end = end_address.as_u64() as usize;

            if !is_executable_address(start) || !is_executable_address(end) {
                return Err(ExecutableImageError::InvalidLoadSegment);
            }

            let access = match (segment.p_flags & PF_W != 0, segment.p_flags & PF_X != 0) {
                (false, false) => UserPageAccess::ReadOnly,
                (true, false) => UserPageAccess::ReadWrite,
                (false, true) => UserPageAccess::ReadExecute,
                (true, true) => return Err(ExecutableImageError::InvalidLoadSegment),
            };

            let data = self
                .elf
                .segment_data(&segment)
                .map_err(ExecutableImageError::Parse)?;

            load_segments.push(LoadSegment {
                start_address,
                end_address,
                memory_size,
                data,
                access,
            });
        }

        load_segments.sort_unstable_by_key(|segment| segment.start_address.as_u64());

        for segments in load_segments.windows(2) {
            let previous_end_page = Page::<Size4KiB>::containing_address(segments[0].end_address);
            let next_start_page = Page::<Size4KiB>::containing_address(segments[1].start_address);

            if previous_end_page >= next_start_page {
                return Err(ExecutableImageError::OverlappingLoadSegments);
            }
        }

        Ok(load_segments)
    }
}
