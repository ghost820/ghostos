use crate::io::{self, PortAddress, ReadWrite};

const DATA_PORT: PortAddress<u8, ReadWrite> = unsafe { PortAddress::new(0x60) };
const STATUS_COMMAND_PORT: PortAddress<u8, ReadWrite> = unsafe { PortAddress::new(0x64) };

const DATA_READY: u8 = 1 << 0;
const INPUT_BUFFER_FULL: u8 = 1 << 1;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    ReadConfigurationByte = 0x20,
    WriteConfigurationByte = 0x60,
    EnableSecondPort = 0xA8,
    WriteToSecondPort = 0xD4,
}

impl Command {
    fn value(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigurationByte(u8);

impl ConfigurationByte {
    const SECOND_PORT_INTERRUPT: u8 = 1 << 1;
    const SECOND_PORT_CLOCK_DISABLED: u8 = 1 << 5;

    pub fn from_raw(value: u8) -> ConfigurationByte {
        ConfigurationByte(value)
    }

    pub fn raw(self) -> u8 {
        self.0
    }

    pub fn with_second_port_enabled(mut self) -> ConfigurationByte {
        self.0 |= Self::SECOND_PORT_INTERRUPT;
        self.0 &= !Self::SECOND_PORT_CLOCK_DISABLED;
        self
    }
}

// TODO: Custom type
pub fn read_status() -> u8 {
    io::read(STATUS_COMMAND_PORT)
}

// TODO: Custom type
pub fn read_data() -> u8 {
    wait_data_ready();
    io::read(DATA_PORT)
}

// TODO: Custom type
pub fn read_data_nowait() -> u8 {
    io::read(DATA_PORT)
}

pub fn write_command(command: Command) {
    wait_ready_for_input();
    io::write(STATUS_COMMAND_PORT, command.value());
}

// TODO: Custom type
pub fn write_data(data: u8) {
    wait_ready_for_input();
    io::write(DATA_PORT, data);
}

pub fn ready_for_input() -> bool {
    read_status() & INPUT_BUFFER_FULL == 0
}

pub fn data_ready() -> bool {
    read_status() & DATA_READY != 0
}

pub fn wait_ready_for_input() {
    // TODO: Avoid waiting forever
    while !ready_for_input() {
        core::hint::spin_loop();
    }
}

pub fn wait_data_ready() {
    // TODO: Avoid waiting forever
    while !data_ready() {
        core::hint::spin_loop();
    }
}

pub fn read_configuration_byte() -> ConfigurationByte {
    write_command(Command::ReadConfigurationByte);
    ConfigurationByte::from_raw(read_data())
}

pub fn write_configuration_byte(config: ConfigurationByte) {
    write_command(Command::WriteConfigurationByte);
    write_data(config.raw());
}

pub fn enable_second_port() {
    write_command(Command::EnableSecondPort);

    let config = read_configuration_byte().with_second_port_enabled();
    write_configuration_byte(config);
}
