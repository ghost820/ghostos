use x86_64::VirtAddr;

use crate::drivers::pci::controller::{
    BarIndex, FunctionAddress, PciIterator, get_command, get_device_id, get_memory_bar,
    get_vendor_id, set_command,
};
use crate::memory;

const INTEL_VENDOR_ID: u16 = 0x8086;
const I82540EM_DEVICE_ID: u16 = 0x100E;

#[derive(Debug)]
pub struct E1000 {
    function_addr: FunctionAddress,
    registers_addr: VirtAddr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    RegistersBarUnavailable,
    MemorySpaceEnableFailed,
}

impl E1000 {
    pub fn init(function_addr: FunctionAddress) -> Result<Self, Error> {
        let registers_bar =
            get_memory_bar(function_addr, BarIndex::Bar0).ok_or(Error::RegistersBarUnavailable)?;

        let registers_addr = memory::phys_to_virt(registers_bar.address());

        let command = get_command(function_addr).with_memory_space_enabled();
        set_command(function_addr, command);

        if !get_command(function_addr).memory_space_enabled() {
            return Err(Error::MemorySpaceEnableFailed);
        }

        Ok(Self {
            function_addr,
            registers_addr,
        })
    }

    pub fn function_addr(&self) -> FunctionAddress {
        self.function_addr
    }
}

pub fn find() -> Option<FunctionAddress> {
    PciIterator::new().find(|&function_addr| {
        get_vendor_id(function_addr) == Some(INTEL_VENDOR_ID)
            && get_device_id(function_addr) == I82540EM_DEVICE_ID
    })
}
