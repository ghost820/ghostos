use core::array;
use core::ptr;
use core::time::Duration;

use x86_64::PhysAddr;

use crate::dma::DmaPage;
use crate::drivers::pci::controller::{
    Bar, BarIndex, FunctionAddress, IoBar, PciIterator, find_io_bar, get_bar, get_command,
    get_device_id, get_vendor_id, set_command,
};
use crate::io::{self, PortAddress, ReadWrite, WriteOnly};
use crate::mmio::MmioRegion;
use crate::net::MacAddress;
use crate::time::sleep;

const INTEL_VENDOR_ID: u16 = 0x8086;
const I82540EM_DEVICE_ID: u16 = 0x100E;

const REGISTER_SPACE_SIZE: usize = 128 * 1024;
const STATUS_REGISTER_OFFSET: usize = 0x0008;
const CTRL_REGISTER_OFFSET: usize = 0x0000;
const ICR_REGISTER_OFFSET: usize = 0x00c0;
const IMC_REGISTER_OFFSET: usize = 0x00d8;
const RCTL_REGISTER_OFFSET: usize = 0x0100;
const TCTL_REGISTER_OFFSET: usize = 0x0400;
const RAL0_REGISTER_OFFSET: usize = 0x5400;
const RAH0_REGISTER_OFFSET: usize = 0x5404;
const RDBAL_REGISTER_OFFSET: usize = 0x2800;
const RDBAH_REGISTER_OFFSET: usize = 0x2804;
const RDLEN_REGISTER_OFFSET: usize = 0x2808;
const RDH_REGISTER_OFFSET: usize = 0x2810;
const RDT_REGISTER_OFFSET: usize = 0x2818;

const RX_DESCRIPTOR_COUNT: usize = 64;
const RX_DESCRIPTOR_RING_SIZE: usize = RX_DESCRIPTOR_COUNT * size_of::<RxDescriptor>();
const _: () = assert!(RX_DESCRIPTOR_RING_SIZE <= 4096);
const _: () = assert!(RX_DESCRIPTOR_RING_SIZE.is_multiple_of(128));
const RX_BUFFER_SIZE: usize = 2048;
const RX_BUFFER_POOL_PAGE_COUNT: usize = 32;

#[derive(Debug)]
pub struct E1000 {
    function_addr: FunctionAddress,
    mac_addr: MacAddress,
    io_bar: IoBar,
    registers: MmioRegion,
    rx_ring: RxDescriptorRing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    IoBarUnavailable,
    RegistersBarUnavailable,
    IoSpaceEnableFailed,
    MemorySpaceEnableFailed,
    BusMasterEnableFailed,
    ResetFailed,
    RxRingAllocationFailed,
}

impl E1000 {
    pub fn init(function_addr: FunctionAddress) -> Result<Self, Error> {
        let io_bar = find_io_bar(function_addr).ok_or(Error::IoBarUnavailable)?;

        let registers_bar = match get_bar(function_addr, BarIndex::Bar0) {
            Some(Bar::Memory(bar)) => bar,
            _ => return Err(Error::RegistersBarUnavailable),
        };

        let command = get_command(function_addr)
            .with_io_space_enabled()
            .with_memory_space_enabled()
            .with_bus_master_enabled();

        set_command(function_addr, command);

        let command = get_command(function_addr);

        if !command.io_space_enabled() {
            return Err(Error::IoSpaceEnableFailed);
        }

        if !command.memory_space_enabled() {
            return Err(Error::MemorySpaceEnableFailed);
        }

        if !command.bus_master_enabled() {
            return Err(Error::BusMasterEnableFailed);
        }

        let registers = unsafe { MmioRegion::new(registers_bar.address(), REGISTER_SPACE_SIZE) };

        let rx_ring = RxDescriptorRing::new().ok_or(Error::RxRingAllocationFailed)?;

        let mut e1000 = Self {
            function_addr,
            mac_addr: MacAddress::new(),
            io_bar,
            registers,
            rx_ring,
        };

        e1000.reset()?;

        let mac_low = e1000.registers.read_u32(RAL0_REGISTER_OFFSET);
        let mac_high = e1000.registers.read_u32(RAH0_REGISTER_OFFSET);
        e1000.mac_addr = MacAddress::from_parts(mac_high, mac_low);

        e1000.configure_rx_ring();

        Ok(e1000)
    }

    pub fn function_addr(&self) -> FunctionAddress {
        self.function_addr
    }

    pub fn mac_address(&self) -> MacAddress {
        self.mac_addr
    }

    pub fn status(&self) -> StatusRegister {
        StatusRegister::from_raw(self.registers.read_u32(STATUS_REGISTER_OFFSET))
    }

    fn reset(&mut self) -> Result<(), Error> {
        const CTRL_RST: u32 = 1 << 26;
        const TCTL_PSP: u32 = 1 << 3;

        self.registers.write_u32(IMC_REGISTER_OFFSET, u32::MAX);
        self.registers.write_u32(RCTL_REGISTER_OFFSET, 0);
        self.registers.write_u32(TCTL_REGISTER_OFFSET, TCTL_PSP);

        let _ = self.registers.read_u32(STATUS_REGISTER_OFFSET);

        sleep(Duration::from_millis(10));

        let ctrl = self.registers.read_u32(CTRL_REGISTER_OFFSET);
        self.write_register_io(CTRL_REGISTER_OFFSET, ctrl | CTRL_RST);

        sleep(Duration::from_millis(5));

        if self.registers.read_u32(CTRL_REGISTER_OFFSET) & CTRL_RST != 0 {
            return Err(Error::ResetFailed);
        }

        self.registers.write_u32(IMC_REGISTER_OFFSET, u32::MAX);
        let _ = self.registers.read_u32(ICR_REGISTER_OFFSET);

        Ok(())
    }

    fn configure_rx_ring(&mut self) {
        let address = self.rx_ring.physical_address().as_u64();

        self.registers
            .write_u32(RDBAL_REGISTER_OFFSET, address as u32);
        self.registers
            .write_u32(RDBAH_REGISTER_OFFSET, (address >> 32) as u32);
        self.registers.write_u32(
            RDLEN_REGISTER_OFFSET,
            u32::try_from(RX_DESCRIPTOR_RING_SIZE)
                .expect("E1000 RX descriptor ring size does not fit u32"),
        );
        self.registers.write_u32(RDH_REGISTER_OFFSET, 0);
        self.registers.write_u32(RDT_REGISTER_OFFSET, 0);
    }

    fn write_register_io(&mut self, offset: usize, value: u32) {
        let address_port = unsafe { PortAddress::<u32, WriteOnly>::new(self.io_bar.base()) };

        let data_port = unsafe {
            PortAddress::<u32, ReadWrite>::new(
                self.io_bar
                    .base()
                    .checked_add(4)
                    .expect("E1000 I/O data port overflow"),
            )
        };

        io::write(
            address_port,
            u32::try_from(offset).expect("E1000 register offset does not fit u32"),
        );
        io::write(data_port, value);
    }
}

pub fn find() -> Option<FunctionAddress> {
    PciIterator::new().find(|&function_addr| {
        get_vendor_id(function_addr) == Some(INTEL_VENDOR_ID)
            && get_device_id(function_addr) == I82540EM_DEVICE_ID
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusRegister(u32);

impl StatusRegister {
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }
}

// TODO: Custom types
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct RxDescriptor {
    buffer_address: u64,
    length: u16,
    checksum: u16,
    status: u8,
    errors: u8,
    special: u16,
}
const _: () = assert!(size_of::<RxDescriptor>() == 16);

// TODO: Custom types
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct TxDescriptor {
    buffer_address: u64,
    length: u16,
    checksum_offset: u8,
    command: u8,
    status: u8,
    checksum_start: u8,
    special: u16,
}
const _: () = assert!(size_of::<TxDescriptor>() == 16);

#[derive(Debug, PartialEq, Eq)]
struct RxDescriptorRing {
    descriptor_page: DmaPage,
    buffer_pages: [DmaPage; RX_BUFFER_POOL_PAGE_COUNT],
}

impl RxDescriptorRing {
    fn new() -> Option<Self> {
        let descriptor_page = DmaPage::new()?;

        let mut buffer_pages: [Option<DmaPage>; RX_BUFFER_POOL_PAGE_COUNT] =
            array::from_fn(|_| None);

        // TODO: Release memory on failure
        for page in &mut buffer_pages {
            *page = Some(DmaPage::new()?);
        }

        let buffer_pages =
            buffer_pages.map(|page| page.expect("E1000 RX buffer page initialization incomplete"));

        let mut ring = Self {
            descriptor_page,
            buffer_pages,
        };

        for index in 0..RX_DESCRIPTOR_COUNT {
            let page = &ring.buffer_pages[index / 2];
            let offset = (index % 2) * RX_BUFFER_SIZE;

            let buffer_address = page
                .physical_address()
                .as_u64()
                .checked_add(offset as u64)
                .expect("E1000 RX buffer physical address overflow");

            ring.write(
                index,
                RxDescriptor {
                    buffer_address,
                    ..RxDescriptor::default()
                },
            );
        }

        Some(ring)
    }

    fn physical_address(&self) -> PhysAddr {
        self.descriptor_page.physical_address()
    }

    fn read(&self, index: usize) -> RxDescriptor {
        assert!(index < RX_DESCRIPTOR_COUNT);

        unsafe { ptr::read_volatile(self.descriptor_page.as_ptr::<RxDescriptor>().add(index)) }
    }

    fn write(&mut self, index: usize, descriptor: RxDescriptor) {
        assert!(index < RX_DESCRIPTOR_COUNT);

        unsafe {
            ptr::write_volatile(
                self.descriptor_page.as_mut_ptr::<RxDescriptor>().add(index),
                descriptor,
            );
        }
    }
}
