use x86_64::structures::paging::{PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

use crate::memory;

#[derive(Debug, PartialEq, Eq)]
pub struct DmaPage {
    frame: PhysFrame<Size4KiB>,
}

impl DmaPage {
    pub fn new() -> Option<Self> {
        Some(Self {
            frame: memory::allocate_dma32_frame()?,
        })
    }

    pub fn physical_address(&self) -> PhysAddr {
        self.frame.start_address()
    }

    pub fn virtual_address(&self) -> VirtAddr {
        memory::phys_to_virt(self.physical_address())
    }

    pub fn as_ptr<T>(&self) -> *const T {
        self.virtual_address().as_ptr()
    }

    pub fn as_mut_ptr<T>(&mut self) -> *mut T {
        self.virtual_address().as_mut_ptr()
    }
}
