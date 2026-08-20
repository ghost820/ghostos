use core::cell::Cell;
use core::marker::PhantomData;
use core::mem::{align_of, size_of};
use core::ptr;

use x86_64::{PhysAddr, VirtAddr};

use crate::memory;

#[derive(Debug, PartialEq, Eq)]
pub struct MmioRegion {
    base: VirtAddr,
    len: usize,
    not_sync: PhantomData<Cell<()>>,
}

impl MmioRegion {
    /// # Safety
    ///
    /// The caller must guarantee that:
    ///
    /// - the physical range refers to device MMIO;
    /// - the whole physical range is mapped into the kernel virtual address space;
    /// - the platform provides a suitable memory type for the MMIO range;
    /// - this region has exclusive logical ownership of the device registers.
    pub(crate) unsafe fn new(physical_base: PhysAddr, len: usize) -> Self {
        assert!(len != 0, "MMIO region cannot be empty");

        let base = memory::phys_to_virt(physical_base);

        Self {
            base,
            len,
            not_sync: PhantomData,
        }
    }

    pub const fn base(&self) -> VirtAddr {
        self.base
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub fn read_u8(&self, offset: usize) -> u8 {
        self.read(offset)
    }

    pub fn read_u16(&self, offset: usize) -> u16 {
        self.read(offset)
    }

    pub fn read_u32(&self, offset: usize) -> u32 {
        self.read(offset)
    }

    pub fn read_u64(&self, offset: usize) -> u64 {
        self.read(offset)
    }

    pub fn write_u8(&mut self, offset: usize, value: u8) {
        self.write(offset, value);
    }

    pub fn write_u16(&mut self, offset: usize, value: u16) {
        self.write(offset, value);
    }

    pub fn write_u32(&mut self, offset: usize, value: u32) {
        self.write(offset, value);
    }

    pub fn write_u64(&mut self, offset: usize, value: u64) {
        self.write(offset, value);
    }

    fn read<T>(&self, offset: usize) -> T {
        let address = self.address::<T>(offset);

        unsafe { ptr::read_volatile(address.cast_const()) }
    }

    fn write<T>(&mut self, offset: usize, value: T) {
        let address = self.address::<T>(offset);

        unsafe {
            ptr::write_volatile(address, value);
        }
    }

    fn address<T>(&self, offset: usize) -> *mut T {
        let end = offset
            .checked_add(size_of::<T>())
            .expect("MMIO access range overflow");

        assert!(end <= self.len, "MMIO access outside region");

        let offset = u64::try_from(offset).expect("MMIO offset does not fit u64");
        let address = self
            .base
            .as_u64()
            .checked_add(offset)
            .expect("MMIO virtual address overflow");

        assert!(
            address.is_multiple_of(align_of::<T>() as u64),
            "unaligned MMIO access"
        );

        VirtAddr::try_new(address)
            .expect("invalid MMIO virtual address")
            .as_mut_ptr()
    }
}
