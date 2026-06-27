use spin::Mutex;

use crate::io::{self, PortAddress, ReadWrite};
use crate::threading::with_lock_no_interrupts;
use crate::{info, warning};

const ADDRESS_PORT: PortAddress<u32, ReadWrite> = unsafe { PortAddress::new(0xCF8) };
const DATA_PORT: PortAddress<u32, ReadWrite> = unsafe { PortAddress::new(0xCFC) };

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

pub fn enumerate() {
    for bus in 0u8..=u8::MAX {
        enumerate_bus(bus);
    }
}

pub fn enumerate_bus(bus: u8) {
    for device in 0u8..32 {
        enumerate_device(bus, device);
    }
}

pub fn enumerate_device(bus: u8, device: u8) {
    let function = FunctionAddress::new(bus, device, 0);

    if get_vendor_id(function).is_none() {
        return;
    }

    print_function(function);

    let header_type = get_header_type(function);
    if !header_type.is_multi_function() {
        return;
    }

    for function in 1u8..8 {
        let function = FunctionAddress::new(bus, device, function);

        if get_vendor_id(function).is_some() {
            print_function(function);
        }
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
        let shift = ((register & 0x03) as u32) * 8;
        let mask = 0xFFu32 << shift;
        let old_value = io_read_u32(function, register);
        let new_value = (old_value & !mask) | ((value as u32) << shift);

        io_write_u32(function, register, new_value);
    });
}

pub fn write_u16(function: FunctionAddress, register: u8, value: u16) {
    assert!(register & 0x01 == 0, "unaligned PCI config u16 write");

    with_lock_no_interrupts(&LOCK, || {
        let shift = ((register & 0x02) as u32) * 8;
        let mask = 0xFFFFu32 << shift;
        let old_value = io_read_u32(function, register);
        let new_value = (old_value & !mask) | ((value as u32) << shift);

        io_write_u32(function, register, new_value);
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
