use spin::Mutex;
use x86_64::PhysAddr;

use crate::io::{self, PortAddress, ReadWrite};
use crate::threading::with_lock_no_interrupts;
use crate::{info, warning};

const ADDRESS_PORT: PortAddress<u32, ReadWrite> = unsafe { PortAddress::new(0xCF8) };
const DATA_PORT: PortAddress<u32, ReadWrite> = unsafe { PortAddress::new(0xCFC) };

fn data_port_u8(register: u8) -> PortAddress<u8, ReadWrite> {
    unsafe { PortAddress::new(0xCFC + u16::from(register & 0x03)) }
}

fn data_port_u16(register: u8) -> PortAddress<u16, ReadWrite> {
    unsafe { PortAddress::new(0xCFC + u16::from(register & 0x03)) }
}

const CONFIG_ENABLE: u32 = 1 << 31;
const NO_VENDOR: u16 = 0xFFFF;

static LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderKind {
    Device,
    PciToPciBridge,
    CardBusBridge,
    Unknown(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeaderType {
    raw: u8,
}

impl HeaderType {
    const MULTI_FUNCTION: u8 = 1 << 7;
    const KIND_MASK: u8 = 0x7F;

    pub const fn from_raw(raw: u8) -> Self {
        Self { raw }
    }

    pub const fn raw(self) -> u8 {
        self.raw
    }

    pub const fn is_multi_function(self) -> bool {
        self.raw & Self::MULTI_FUNCTION != 0
    }

    pub const fn kind(self) -> HeaderKind {
        match self.raw & Self::KIND_MASK {
            0x00 => HeaderKind::Device,
            0x01 => HeaderKind::PciToPciBridge,
            0x02 => HeaderKind::CardBusBridge,
            value => HeaderKind::Unknown(value),
        }
    }
}

pub fn get_header_type(function: FunctionAddress) -> HeaderType {
    HeaderType::from_raw(read_u8(function, 0x0E))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Class {
    class: u8,
    subclass: u8,
    programming_interface: u8,
}

impl Class {
    pub const fn new(class: u8, subclass: u8, programming_interface: u8) -> Self {
        Self {
            class,
            subclass,
            programming_interface,
        }
    }

    pub const fn class(self) -> u8 {
        self.class
    }

    pub const fn subclass(self) -> u8 {
        self.subclass
    }

    pub const fn programming_interface(self) -> u8 {
        self.programming_interface
    }
}

pub fn get_class(function: FunctionAddress) -> Class {
    Class::new(
        read_u8(function, 0x0B),
        read_u8(function, 0x0A),
        read_u8(function, 0x09),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptPin {
    None,
    IntA,
    IntB,
    IntC,
    IntD,
    Unknown(u8),
}

impl InterruptPin {
    pub const fn from_raw(raw: u8) -> Self {
        match raw {
            0 => Self::None,
            1 => Self::IntA,
            2 => Self::IntB,
            3 => Self::IntC,
            4 => Self::IntD,
            value => Self::Unknown(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interrupt {
    line: Option<u8>,
    pin: InterruptPin,
}

impl Interrupt {
    const NO_LINE: u8 = 0xFF;

    pub const fn new(line: Option<u8>, pin: InterruptPin) -> Self {
        Self {
            line: match line {
                Some(Self::NO_LINE) | None => None,
                Some(line) => Some(line),
            },
            pin,
        }
    }

    pub const fn from_raw(line: u8, pin: u8) -> Self {
        Self::new(Some(line), InterruptPin::from_raw(pin))
    }

    pub const fn line(self) -> Option<u8> {
        self.line
    }

    pub const fn pin(self) -> InterruptPin {
        self.pin
    }

    pub const fn raw_line(self) -> u8 {
        match self.line {
            Some(line) => line,
            None => Self::NO_LINE,
        }
    }
}

pub fn get_interrupt(function: FunctionAddress) -> Interrupt {
    Interrupt::from_raw(read_u8(function, 0x3C), read_u8(function, 0x3D))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FunctionAddress {
    bus: u8,
    device: u8,
    function: u8,
}

impl FunctionAddress {
    pub const fn new(bus: u8, device: u8, function: u8) -> Self {
        assert!(device < 32, "device number must be less than 32");
        assert!(function < 8, "function number must be less than 8");

        Self {
            bus,
            device,
            function,
        }
    }

    pub const fn bus(self) -> u8 {
        self.bus
    }

    pub const fn device(self) -> u8 {
        self.device
    }

    pub const fn function(self) -> u8 {
        self.function
    }

    const fn config_address(self, register: u8) -> u32 {
        CONFIG_ENABLE
            | ((self.bus as u32) << 16)
            | ((self.device as u32) << 11)
            | ((self.function as u32) << 8)
            | (register as u32 & 0xFC)
    }
}

pub struct PciIterator {
    next: Option<FunctionAddress>,
    function_limit: u8,
}

impl PciIterator {
    const DEVICES_PER_BUS: u8 = 32;
    const FUNCTIONS_PER_DEVICE: u8 = 8;

    pub const fn new() -> Self {
        Self {
            next: Some(FunctionAddress::new(0, 0, 0)),
            function_limit: 1,
        }
    }

    fn next_device(current: FunctionAddress) -> Option<FunctionAddress> {
        if current.device() + 1 < Self::DEVICES_PER_BUS {
            return Some(FunctionAddress::new(current.bus(), current.device() + 1, 0));
        }

        if current.bus() < u8::MAX {
            return Some(FunctionAddress::new(current.bus() + 1, 0, 0));
        }

        None
    }

    fn advance(&mut self, current: FunctionAddress) {
        let next_function = current.function() + 1;

        self.next = if next_function < self.function_limit {
            Some(FunctionAddress::new(
                current.bus(),
                current.device(),
                next_function,
            ))
        } else {
            Self::next_device(current)
        };
    }
}

impl Default for PciIterator {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for PciIterator {
    type Item = FunctionAddress;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let current = self.next?;
            let present = get_vendor_id(current).is_some();

            if current.function() == 0 {
                if !present {
                    self.next = Self::next_device(current);
                    continue;
                }

                self.function_limit = if get_header_type(current).is_multi_function() {
                    Self::FUNCTIONS_PER_DEVICE
                } else {
                    1
                };
            }

            self.advance(current);

            if present {
                return Some(current);
            }
        }
    }
}

impl core::iter::FusedIterator for PciIterator {}

pub fn enumerate() {
    for function in PciIterator::new() {
        print_function(function);
    }
}

pub fn print_function(function: FunctionAddress) {
    let Some(vendor_id) = get_vendor_id(function) else {
        warning!(
            "PCI function disappeared or is invalid: bus={:#04x}, device={:#04x}, function={:#04x}",
            function.bus(),
            function.device(),
            function.function()
        );

        #[allow(unreachable_code)]
        return;
    };

    let device_id = get_device_id(function);
    let header_type = get_header_type(function);
    let class = get_class(function);

    info!(
        "PCI device function found: bus={:#04x}, device={:#04x}, function={:#04x}, vendor_id={:#06x}, device_id={:#06x}, header={:?}, class={:#04x}, subclass={:#04x}, prog_if={:#04x}",
        function.bus(),
        function.device(),
        function.function(),
        vendor_id,
        device_id,
        header_type.kind(),
        class.class(),
        class.subclass(),
        class.programming_interface()
    );
}

pub fn get_vendor_id(function: FunctionAddress) -> Option<u16> {
    let vendor_id = read_u16(function, 0x00);

    if vendor_id == NO_VENDOR {
        None
    } else {
        Some(vendor_id)
    }
}

pub fn get_device_id(function: FunctionAddress) -> u16 {
    read_u16(function, 0x02)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandRegister(u16);

impl CommandRegister {
    const IO_SPACE_ENABLE: u16 = 1 << 0;
    const MEMORY_SPACE_ENABLE: u16 = 1 << 1;
    const BUS_MASTER_ENABLE: u16 = 1 << 2;

    pub const fn with_io_space_enabled(mut self) -> Self {
        self.0 |= Self::IO_SPACE_ENABLE;
        self
    }

    pub const fn with_memory_space_enabled(mut self) -> Self {
        self.0 |= Self::MEMORY_SPACE_ENABLE;
        self
    }

    pub const fn with_bus_master_enabled(mut self) -> Self {
        self.0 |= Self::BUS_MASTER_ENABLE;
        self
    }

    pub const fn from_raw(raw: u16) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u16 {
        self.0
    }

    pub const fn io_space_enabled(self) -> bool {
        self.0 & Self::IO_SPACE_ENABLE != 0
    }

    pub const fn memory_space_enabled(self) -> bool {
        self.0 & Self::MEMORY_SPACE_ENABLE != 0
    }

    pub const fn bus_master_enabled(self) -> bool {
        self.0 & Self::BUS_MASTER_ENABLE != 0
    }
}

pub fn get_command(function: FunctionAddress) -> CommandRegister {
    CommandRegister::from_raw(read_u16(function, 0x04))
}

pub fn set_command(function: FunctionAddress, command: CommandRegister) {
    write_u16(function, 0x04, command.raw());
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarIndex {
    Bar0 = 0,
    Bar1 = 1,
    Bar2 = 2,
    Bar3 = 3,
    Bar4 = 4,
    Bar5 = 5,
}

impl BarIndex {
    pub const fn register_offset(self) -> u8 {
        0x10 + (self as u8) * 4
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bar {
    Io(IoBar),
    Memory(MemoryBar),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IoBar {
    base: u16,
}

impl IoBar {
    pub const fn base(self) -> u16 {
        self.base
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryBarWidth {
    Bits32,
    Bits64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryBar {
    address: PhysAddr,
    width: MemoryBarWidth,
    prefetchable: bool,
}

impl MemoryBar {
    pub const fn address(self) -> PhysAddr {
        self.address
    }

    pub const fn width(self) -> MemoryBarWidth {
        self.width
    }

    pub const fn is_prefetchable(self) -> bool {
        self.prefetchable
    }
}

pub fn get_bar(function_addr: FunctionAddress, index: BarIndex) -> Option<Bar> {
    const IO_SPACE: u32 = 1;
    const IO_ADDRESS_MASK: u32 = !0x3;
    const MEMORY_TYPE_MASK: u32 = 0b11 << 1;
    const MEMORY_TYPE_32_BIT: u32 = 0b00 << 1;
    const MEMORY_TYPE_64_BIT: u32 = 0b10 << 1;
    const MEMORY_PREFETCHABLE: u32 = 1 << 3;
    const MEMORY_ADDRESS_MASK: u32 = !0xF;

    let low = read_u32(function_addr, index.register_offset());

    if low & IO_SPACE != 0 {
        let address = low & IO_ADDRESS_MASK;

        if address == 0 {
            return None;
        }

        let base = u16::try_from(address).ok()?;

        return Some(Bar::Io(IoBar { base }));
    }

    let low_address = (low & MEMORY_ADDRESS_MASK) as u64;

    let (address, width) = match low & MEMORY_TYPE_MASK {
        MEMORY_TYPE_32_BIT => (low_address, MemoryBarWidth::Bits32),
        MEMORY_TYPE_64_BIT => {
            if index == BarIndex::Bar5 {
                return None;
            }

            let high = read_u32(function_addr, index.register_offset() + 4) as u64;

            ((high << 32) | low_address, MemoryBarWidth::Bits64)
        }
        _ => return None,
    };

    if address == 0 {
        return None;
    }

    Some(Bar::Memory(MemoryBar {
        address: PhysAddr::try_new(address).ok()?,
        width,
        prefetchable: low & MEMORY_PREFETCHABLE != 0,
    }))
}

pub fn find_io_bar(function_addr: FunctionAddress) -> Option<IoBar> {
    let indices = [
        BarIndex::Bar0,
        BarIndex::Bar1,
        BarIndex::Bar2,
        BarIndex::Bar3,
        BarIndex::Bar4,
        BarIndex::Bar5,
    ];

    let mut skip_next = false;

    for index in indices {
        if skip_next {
            skip_next = false;
            continue;
        }

        match get_bar(function_addr, index) {
            Some(Bar::Io(bar)) => return Some(bar),
            Some(Bar::Memory(bar)) => {
                skip_next = bar.width() == MemoryBarWidth::Bits64;
            }
            None => {}
        }
    }

    None
}

pub fn read_u8(function: FunctionAddress, register: u8) -> u8 {
    with_lock_no_interrupts(&LOCK, || {
        let value = io_read_u32(function, register);
        let shift = ((register & 0x03) as u32) * 8;

        (value >> shift) as u8
    })
}

pub fn read_u16(function: FunctionAddress, register: u8) -> u16 {
    assert!(register & 0x01 == 0, "unaligned PCI config u16 read");

    with_lock_no_interrupts(&LOCK, || {
        let value = io_read_u32(function, register);
        let shift = ((register & 0x02) as u32) * 8;

        (value >> shift) as u16
    })
}

pub fn read_u32(function: FunctionAddress, register: u8) -> u32 {
    assert!(register & 0x03 == 0, "unaligned PCI config u32 read");

    with_lock_no_interrupts(&LOCK, || io_read_u32(function, register))
}

pub fn write_u8(function: FunctionAddress, register: u8, value: u8) {
    with_lock_no_interrupts(&LOCK, || {
        io::write(ADDRESS_PORT, function.config_address(register));
        io::write(data_port_u8(register), value);
    });
}

pub fn write_u16(function: FunctionAddress, register: u8, value: u16) {
    assert!(register & 0x01 == 0, "unaligned PCI config u16 write");

    with_lock_no_interrupts(&LOCK, || {
        io::write(ADDRESS_PORT, function.config_address(register));
        io::write(data_port_u16(register), value);
    });
}

pub fn write_u32(function: FunctionAddress, register: u8, value: u32) {
    assert!(register & 0x03 == 0, "unaligned PCI config u32 write");

    with_lock_no_interrupts(&LOCK, || io_write_u32(function, register, value));
}

fn io_read_u32(function: FunctionAddress, register: u8) -> u32 {
    io::write(ADDRESS_PORT, function.config_address(register));
    io::read(DATA_PORT)
}

fn io_write_u32(function: FunctionAddress, register: u8, value: u32) {
    io::write(ADDRESS_PORT, function.config_address(register));
    io::write(DATA_PORT, value);
}
