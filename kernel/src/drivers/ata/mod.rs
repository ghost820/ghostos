use lazy_static::lazy_static;
use spin::Mutex;

use crate::drivers::pci::controller::{FunctionAddress, PciIterator, get_class};
use crate::io::{self, PortAddress, ReadWrite};

const MASS_STORAGE_CLASS: u8 = 0x01;
const IDE_SUBCLASS: u8 = 0x01;

const STATUS_POLL_LIMIT: usize = 1_000_000;

lazy_static! {
    pub static ref PRIMARY_CHANNEL: Mutex<Channel> =
        Mutex::new(Channel::compatibility(ChannelId::Primary));
    pub static ref SECONDARY_CHANNEL: Mutex<Channel> =
        Mutex::new(Channel::compatibility(ChannelId::Secondary));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelMode {
    Compatibility,
    Native,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdeProgrammingInterface(u8);

impl IdeProgrammingInterface {
    const PRIMARY_NATIVE: u8 = 1 << 0;
    const PRIMARY_SWITCHABLE: u8 = 1 << 1;
    const SECONDARY_NATIVE: u8 = 1 << 2;
    const SECONDARY_SWITCHABLE: u8 = 1 << 3;
    const BUS_MASTER_DMA: u8 = 1 << 7;

    pub const fn from_raw(raw: u8) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u8 {
        self.0
    }

    pub const fn primary_mode(self) -> ChannelMode {
        if self.0 & Self::PRIMARY_NATIVE != 0 {
            ChannelMode::Native
        } else {
            ChannelMode::Compatibility
        }
    }

    pub const fn primary_is_switchable(self) -> bool {
        self.0 & Self::PRIMARY_SWITCHABLE != 0
    }

    pub const fn secondary_mode(self) -> ChannelMode {
        if self.0 & Self::SECONDARY_NATIVE != 0 {
            ChannelMode::Native
        } else {
            ChannelMode::Compatibility
        }
    }

    pub const fn secondary_is_switchable(self) -> bool {
        self.0 & Self::SECONDARY_SWITCHABLE != 0
    }

    pub const fn supports_bus_master_dma(self) -> bool {
        self.0 & Self::BUS_MASTER_DMA != 0
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceId {
    Master = 0,
    Slave = 1 << 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceSignature(u16);

impl DeviceSignature {
    pub const fn from_registers(lba_mid: u8, lba_high: u8) -> Self {
        Self((lba_mid as u16) | ((lba_high as u16) << 8))
    }

    pub const fn raw(self) -> u16 {
        self.0
    }

    pub const fn is_ata(self) -> bool {
        self.0 == 0
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    ReadSectors = 0x20,
    ReadSectorsExt = 0x24,
    WriteSectors = 0x30,
    WriteSectorsExt = 0x34,
    FlushCache = 0xE7,
    FlushCacheExt = 0xEA,
    IdentifyDevice = 0xEC,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Status(u8);

impl Status {
    const ERROR: u8 = 1 << 0;
    const DATA_REQUEST: u8 = 1 << 3;
    const DEVICE_FAULT: u8 = 1 << 5;
    const DEVICE_READY: u8 = 1 << 6;
    const DEVICE_BUSY: u8 = 1 << 7;

    pub const fn from_raw(raw: u8) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u8 {
        self.0
    }

    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub const fn is_floating_bus(self) -> bool {
        self.0 == u8::MAX
    }

    pub const fn has_error(self) -> bool {
        self.0 & Self::ERROR != 0
    }

    pub const fn data_requested(self) -> bool {
        self.0 & Self::DATA_REQUEST != 0
    }

    pub const fn has_device_fault(self) -> bool {
        self.0 & Self::DEVICE_FAULT != 0
    }

    pub const fn device_ready(self) -> bool {
        self.0 & Self::DEVICE_READY != 0
    }

    pub const fn device_busy(self) -> bool {
        self.0 & Self::DEVICE_BUSY != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorRegister(u8);

impl ErrorRegister {
    const ADDRESS_MARK_NOT_FOUND: u8 = 1 << 0;
    const TRACK_ZERO_NOT_FOUND: u8 = 1 << 1;
    const COMMAND_ABORTED: u8 = 1 << 2;
    const MEDIA_CHANGE_REQUESTED: u8 = 1 << 3;
    const ID_NOT_FOUND: u8 = 1 << 4;
    const MEDIA_CHANGED: u8 = 1 << 5;
    const UNCORRECTABLE_DATA: u8 = 1 << 6;
    const INTERFACE_CRC_ERROR: u8 = 1 << 7;

    pub const fn from_raw(raw: u8) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u8 {
        self.0
    }

    pub const fn address_mark_not_found(self) -> bool {
        self.0 & Self::ADDRESS_MARK_NOT_FOUND != 0
    }

    pub const fn track_zero_not_found(self) -> bool {
        self.0 & Self::TRACK_ZERO_NOT_FOUND != 0
    }

    pub const fn command_aborted(self) -> bool {
        self.0 & Self::COMMAND_ABORTED != 0
    }

    pub const fn media_change_requested(self) -> bool {
        self.0 & Self::MEDIA_CHANGE_REQUESTED != 0
    }

    pub const fn id_not_found(self) -> bool {
        self.0 & Self::ID_NOT_FOUND != 0
    }

    pub const fn media_changed(self) -> bool {
        self.0 & Self::MEDIA_CHANGED != 0
    }

    pub const fn uncorrectable_data(self) -> bool {
        self.0 & Self::UNCORRECTABLE_DATA != 0
    }

    pub const fn interface_crc_error(self) -> bool {
        self.0 & Self::INTERFACE_CRC_ERROR != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lba28(u32);

impl Lba28 {
    pub const MAX: u32 = (1 << 28) - 1;

    pub const fn new(value: u32) -> Self {
        assert!(value <= Self::MAX);

        Self(value)
    }

    pub const fn value(self) -> u32 {
        self.0
    }

    pub const fn is_range_addressable(self, sector_count: u16) -> bool {
        if sector_count == 0 {
            return false;
        }

        let last_sector_offset = sector_count as u32 - 1;

        last_sector_offset <= Self::MAX - self.0
    }

    pub const fn is_range_addressable_on_device(
        self,
        sector_count: u16,
        total_sector_count: u64,
    ) -> bool {
        self.is_range_addressable(sector_count)
            && self.0 as u64 + sector_count as u64 <= total_sector_count
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lba48(u64);

impl Lba48 {
    pub const MAX: u64 = (1 << 48) - 1;

    pub const fn new(value: u64) -> Self {
        assert!(value <= Self::MAX);

        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }

    pub const fn as_lba28(self) -> Option<Lba28> {
        if self.0 <= Lba28::MAX as u64 {
            Some(Lba28::new(self.0 as u32))
        } else {
            None
        }
    }

    pub const fn is_range_addressable_on_device(
        self,
        sector_count: u32,
        total_sector_count: u64,
    ) -> bool {
        if sector_count == 0 {
            return false;
        }

        let last_sector_offset = sector_count as u64 - 1;

        if last_sector_offset > Self::MAX - self.0 {
            return false;
        }

        self.0 + sector_count as u64 <= total_sector_count
    }
}

pub struct CommandBlock {
    data: PortAddress<u16, ReadWrite>,
    error_features: PortAddress<u8, ReadWrite>,
    sector_count: PortAddress<u8, ReadWrite>,
    lba_low: PortAddress<u8, ReadWrite>,
    lba_mid: PortAddress<u8, ReadWrite>,
    lba_high: PortAddress<u8, ReadWrite>,
    device: PortAddress<u8, ReadWrite>,
    status_command: PortAddress<u8, ReadWrite>,
}

impl CommandBlock {
    const LBA_MODE: u8 = 1 << 6;
    const DEVICE_OBSOLETE_BITS: u8 = (1 << 7) | (1 << 5);

    unsafe fn new(base: u16) -> Self {
        Self {
            data: unsafe { PortAddress::new(base) },
            error_features: unsafe { PortAddress::new(base + 1) },
            sector_count: unsafe { PortAddress::new(base + 2) },
            lba_low: unsafe { PortAddress::new(base + 3) },
            lba_mid: unsafe { PortAddress::new(base + 4) },
            lba_high: unsafe { PortAddress::new(base + 5) },
            device: unsafe { PortAddress::new(base + 6) },
            status_command: unsafe { PortAddress::new(base + 7) },
        }
    }

    pub fn read_device_signature(&self) -> DeviceSignature {
        DeviceSignature::from_registers(io::read(self.lba_mid), io::read(self.lba_high))
    }

    pub fn read_status(&self) -> Status {
        Status::from_raw(io::read(self.status_command))
    }

    pub fn read_error(&self) -> ErrorRegister {
        ErrorRegister::from_raw(io::read(self.error_features))
    }

    pub fn read_diagnostic_code(&self) -> DiagnosticCode {
        DiagnosticCode::from_raw(io::read(self.error_features))
    }

    pub fn read_data(&self) -> u16 {
        io::read(self.data)
    }

    pub fn read_data_into(&self, buffer: &mut [u16]) {
        for word in buffer {
            *word = io::read(self.data);
        }
    }

    pub fn write_command(&mut self, command: Command) {
        io::write(self.status_command, command as u8);
    }

    // TODO: Custom type
    pub fn write_features(&mut self, features: u8) {
        io::write(self.error_features, features);
    }

    pub fn write_lba28(&mut self, device: DeviceId, lba: Lba28) {
        let lba = lba.value();

        io::write(self.lba_low, lba as u8);
        io::write(self.lba_mid, (lba >> 8) as u8);
        io::write(self.lba_high, (lba >> 16) as u8);
        io::write(
            self.device,
            Self::DEVICE_OBSOLETE_BITS | Self::LBA_MODE | device as u8 | ((lba >> 24) as u8 & 0x0F),
        );
    }

    pub fn write_lba48(&mut self, device: DeviceId, lba: Lba48, count: u32) {
        assert!(count >= 1 && count <= 65_536);

        let lba = lba.value();
        let count = if count == 65_536 { 0 } else { count as u16 };

        io::write(
            self.device,
            Self::DEVICE_OBSOLETE_BITS | Self::LBA_MODE | device as u8,
        );

        io::write(self.sector_count, (count >> 8) as u8);
        io::write(self.lba_low, (lba >> 24) as u8);
        io::write(self.lba_mid, (lba >> 32) as u8);
        io::write(self.lba_high, (lba >> 40) as u8);

        io::write(self.sector_count, count as u8);
        io::write(self.lba_low, lba as u8);
        io::write(self.lba_mid, (lba >> 8) as u8);
        io::write(self.lba_high, (lba >> 16) as u8);
    }

    pub fn write_sector_count(&mut self, count: u16) {
        assert!(count >= 1 && count <= 256);

        let encoded = if count == 256 { 0 } else { count as u8 };

        io::write(self.sector_count, encoded);
    }

    pub fn write_data_from(&mut self, buffer: &[u16]) {
        for word in buffer {
            io::write(self.data, *word);
        }
    }

    pub fn clear_address_and_count_registers(&mut self) {
        io::write(self.sector_count, 0);
        io::write(self.lba_low, 0);
        io::write(self.lba_mid, 0);
        io::write(self.lba_high, 0);
    }

    pub fn select_device(&mut self, device: DeviceId) {
        io::write(self.device, Self::DEVICE_OBSOLETE_BITS | device as u8);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceControl(u8);

impl DeviceControl {
    const INTERRUPTS_DISABLED: u8 = 1 << 1;
    const SOFTWARE_RESET: u8 = 1 << 2;

    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn with_interrupts_disabled(mut self) -> Self {
        self.0 |= Self::INTERRUPTS_DISABLED;
        self
    }

    pub const fn with_software_reset(mut self) -> Self {
        self.0 |= Self::SOFTWARE_RESET;
        self
    }

    pub const fn raw(self) -> u8 {
        self.0
    }
}

impl Default for DeviceControl {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ControlBlock {
    alternate_status_device_control: PortAddress<u8, ReadWrite>,
}

impl ControlBlock {
    unsafe fn new(port: u16) -> Self {
        Self {
            alternate_status_device_control: unsafe { PortAddress::new(port) },
        }
    }

    pub fn read_alternate_status(&self) -> Status {
        Status::from_raw(io::read(self.alternate_status_device_control))
    }

    pub fn write_device_control(&mut self, value: DeviceControl) {
        io::write(self.alternate_status_device_control, value.raw());
    }

    fn delay_2ms(&self) {
        for _ in 0..5_000 {
            self.delay_400ns();
        }
    }

    fn delay_5us(&self) {
        for _ in 0..13 {
            self.delay_400ns();
        }
    }

    fn delay_400ns(&self) {
        for _ in 0..4 {
            let _ = self.read_alternate_status();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelId {
    Primary,
    Secondary,
}

pub struct IdentifyData([u16; 256]);

impl IdentifyData {
    const LBA_SUPPORTED: u16 = 1 << 9;
    const LBA48_SUPPORTED: u16 = 1 << 10;
    const COMMAND_SET_SUPPORT_VALID: u16 = 1 << 14;
    const COMMAND_SET_SUPPORT_INVALID: u16 = 1 << 15;

    const LONG_LOGICAL_SECTOR: u16 = 1 << 12;
    const SECTOR_SIZE_INFO_VALID: u16 = 1 << 14;
    const SECTOR_SIZE_INFO_INVALID: u16 = 1 << 15;

    pub const fn supports_lba(&self) -> bool {
        self.0[49] & Self::LBA_SUPPORTED != 0
    }

    pub const fn supports_lba48(&self) -> bool {
        let capabilities = self.0[83];

        capabilities & Self::COMMAND_SET_SUPPORT_VALID != 0
            && capabilities & Self::COMMAND_SET_SUPPORT_INVALID == 0
            && capabilities & Self::LBA48_SUPPORTED != 0
    }

    pub const fn logical_sector_size(&self) -> u32 {
        let info = self.0[106];

        let info_valid =
            info & Self::SECTOR_SIZE_INFO_VALID != 0 && info & Self::SECTOR_SIZE_INFO_INVALID == 0;

        if !info_valid || info & Self::LONG_LOGICAL_SECTOR == 0 {
            return 512;
        }

        let words = (self.0[117] as u32) | ((self.0[118] as u32) << 16);

        words * 2
    }

    // TODO: Option?
    pub const fn logical_sector_count(&self) -> Option<u64> {
        if self.supports_lba48() {
            let count = (self.0[100] as u64)
                | ((self.0[101] as u64) << 16)
                | ((self.0[102] as u64) << 32)
                | ((self.0[103] as u64) << 48);

            Some(count)
        } else if self.supports_lba() {
            let count = (self.0[60] as u64) | ((self.0[61] as u64) << 16);

            Some(count)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticCode(u8);

impl DiagnosticCode {
    const PASSED: u8 = 0x01;

    pub const fn from_raw(raw: u8) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u8 {
        self.0
    }

    pub const fn is_success(self) -> bool {
        self.0 == Self::PASSED
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtaError {
    CommandFailed {
        status: Status,
        error: ErrorRegister,
    },
    DeviceFault(Status),
    DiagnosticFailed(DiagnosticCode),
    FloatingBus,
    NoDevice,
    NotAtaDevice(DeviceSignature),
    PollLimitExceeded(Status),
}

pub struct Channel {
    id: ChannelId,
    command: CommandBlock,
    control: ControlBlock,
}

impl Channel {
    fn compatibility(id: ChannelId) -> Self {
        let (command_base, control_port) = match id {
            ChannelId::Primary => (0x1F0, 0x3F6),
            ChannelId::Secondary => (0x170, 0x376),
        };

        let mut channel = Self {
            id,
            command: unsafe { CommandBlock::new(command_base) },
            control: unsafe { ControlBlock::new(control_port) },
        };

        channel
            .control
            .write_device_control(DeviceControl::new().with_interrupts_disabled());

        channel
    }

    pub const fn id(&self) -> ChannelId {
        self.id
    }

    pub fn identify(&mut self, device: DeviceId) -> Result<IdentifyData, AtaError> {
        self.select_device(device)?;
        self.wait_until_idle()?;

        self.command.clear_address_and_count_registers();
        self.write_command(Command::IdentifyDevice);

        let status = self.wait_until_not_busy()?;
        let signature = self.command.read_device_signature();

        if !signature.is_ata() {
            return Err(AtaError::NotAtaDevice(signature));
        }

        self.validate_command_status(status)?;

        if !status.data_requested() {
            self.wait_for_data_request()?;
        }

        let mut words = [0; 256];

        self.command.read_data_into(&mut words);

        let final_status = self.wait_until_idle()?;
        self.validate_command_status(final_status)?;

        Ok(IdentifyData(words))
    }

    pub fn software_reset(&mut self) -> Result<(), AtaError> {
        let control = DeviceControl::new().with_interrupts_disabled();

        self.control.write_device_control(control);
        self.control.delay_5us();

        self.control
            .write_device_control(control.with_software_reset());
        self.control.delay_5us();

        self.control.write_device_control(control);
        self.control.delay_2ms();

        self.wait_until_reset_complete()?;

        let diagnostic_code = self.command.read_diagnostic_code();

        if !diagnostic_code.is_success() {
            return Err(AtaError::DiagnosticFailed(diagnostic_code));
        }

        Ok(())
    }

    pub fn read_alternate_status(&self) -> Status {
        self.control.read_alternate_status()
    }

    pub fn write_command(&mut self, command: Command) {
        self.command.write_command(command);
        self.control.delay_400ns();
    }

    pub fn select_device(&mut self, device: DeviceId) -> Result<(), AtaError> {
        self.wait_until_device_can_be_selected()?;
        self.command.select_device(device);
        self.control.delay_400ns();
        Ok(())
    }

    pub fn flush_cache(&mut self, device: DeviceId) -> Result<(), AtaError> {
        self.select_device(device)?;
        self.wait_until_idle()?;

        self.write_command(Command::FlushCache);

        let status = self.wait_until_idle()?;
        self.validate_command_status(status)?;

        Ok(())
    }

    pub fn flush_cache_ext(&mut self, device: DeviceId) -> Result<(), AtaError> {
        self.select_device(device)?;
        self.wait_until_idle()?;

        self.write_command(Command::FlushCacheExt);

        let status = self.wait_until_idle()?;
        self.validate_command_status(status)?;

        Ok(())
    }

    fn read_sectors_lba28(
        &mut self,
        device: DeviceId,
        lba: Lba28,
        sector_count: u16,
        words_per_sector: usize,
        buffer: &mut [u16],
    ) -> Result<(), AtaError> {
        self.select_device(device)?;
        self.wait_until_idle()?;

        self.command.write_lba28(device, lba);
        self.command.write_sector_count(sector_count);
        self.write_command(Command::ReadSectors);

        for sector in buffer.chunks_exact_mut(words_per_sector) {
            self.wait_for_data_request()?;
            self.command.read_data_into(sector);
        }

        let status = self.wait_until_idle()?;
        self.validate_command_status(status)?;

        Ok(())
    }

    fn read_sectors_lba48(
        &mut self,
        device: DeviceId,
        lba: Lba48,
        sector_count: u32,
        words_per_sector: usize,
        buffer: &mut [u16],
    ) -> Result<(), AtaError> {
        self.select_device(device)?;
        self.wait_until_idle()?;

        self.command.write_lba48(device, lba, sector_count);
        self.write_command(Command::ReadSectorsExt);

        for sector in buffer.chunks_exact_mut(words_per_sector) {
            self.wait_for_data_request()?;
            self.command.read_data_into(sector);
        }

        let status = self.wait_until_idle()?;
        self.validate_command_status(status)?;

        Ok(())
    }

    fn write_sectors_lba28(
        &mut self,
        device: DeviceId,
        lba: Lba28,
        sector_count: u16,
        words_per_sector: usize,
        buffer: &[u16],
    ) -> Result<(), AtaError> {
        self.select_device(device)?;
        self.wait_until_idle()?;

        self.command.write_lba28(device, lba);
        self.command.write_sector_count(sector_count);
        self.write_command(Command::WriteSectors);

        for sector in buffer.chunks_exact(words_per_sector) {
            self.wait_for_data_request()?;
            self.command.write_data_from(sector);
        }

        let status = self.wait_until_idle()?;
        self.validate_command_status(status)?;

        Ok(())
    }

    fn write_sectors_lba48(
        &mut self,
        device: DeviceId,
        lba: Lba48,
        sector_count: u32,
        words_per_sector: usize,
        buffer: &[u16],
    ) -> Result<(), AtaError> {
        self.select_device(device)?;
        self.wait_until_idle()?;

        self.command.write_lba48(device, lba, sector_count);
        self.write_command(Command::WriteSectorsExt);

        for sector in buffer.chunks_exact(words_per_sector) {
            self.wait_for_data_request()?;
            self.command.write_data_from(sector);
        }

        let status = self.wait_until_idle()?;
        self.validate_command_status(status)?;

        Ok(())
    }

    fn wait_until_device_can_be_selected(&self) -> Result<Status, AtaError> {
        let mut status = self.control.read_alternate_status();

        // TODO: Use a timeout
        for _ in 0..STATUS_POLL_LIMIT {
            if status.is_floating_bus() {
                return Err(AtaError::FloatingBus);
            }

            if !status.device_busy() && !status.data_requested() {
                return Ok(status);
            }

            core::hint::spin_loop();
            status = self.control.read_alternate_status();
        }

        Err(AtaError::PollLimitExceeded(status))
    }

    fn wait_until_idle(&self) -> Result<Status, AtaError> {
        let mut status = self.control.read_alternate_status();

        // TODO: Use a timeout
        for _ in 0..STATUS_POLL_LIMIT {
            Self::validate_presence(status)?;

            if !status.device_busy() && !status.data_requested() {
                return Ok(status);
            }

            core::hint::spin_loop();
            status = self.control.read_alternate_status();
        }

        Err(AtaError::PollLimitExceeded(status))
    }

    fn wait_until_not_busy(&self) -> Result<Status, AtaError> {
        let mut status = self.control.read_alternate_status();

        // TODO: Use a timeout
        for _ in 0..STATUS_POLL_LIMIT {
            Self::validate_presence(status)?;

            if !status.device_busy() {
                return Ok(status);
            }

            core::hint::spin_loop();
            status = self.control.read_alternate_status();
        }

        Err(AtaError::PollLimitExceeded(status))
    }

    fn wait_for_data_request(&self) -> Result<Status, AtaError> {
        let mut status = self.control.read_alternate_status();

        // TODO: Use a timeout
        for _ in 0..STATUS_POLL_LIMIT {
            Self::validate_presence(status)?;

            if !status.device_busy() {
                self.validate_command_status(status)?;

                if status.data_requested() {
                    return Ok(status);
                }
            }

            core::hint::spin_loop();
            status = self.control.read_alternate_status();
        }

        Err(AtaError::PollLimitExceeded(status))
    }

    fn wait_until_reset_complete(&self) -> Result<Status, AtaError> {
        let mut status = self.control.read_alternate_status();

        // TODO: Use a timeout
        for _ in 0..STATUS_POLL_LIMIT {
            if status.is_floating_bus() {
                return Err(AtaError::FloatingBus);
            }

            if !status.device_busy() {
                return Ok(status);
            }

            core::hint::spin_loop();
            status = self.control.read_alternate_status();
        }

        Err(AtaError::PollLimitExceeded(status))
    }

    fn validate_command_status(&self, status: Status) -> Result<(), AtaError> {
        if status.has_device_fault() {
            return Err(AtaError::DeviceFault(status));
        }

        if status.has_error() {
            return Err(AtaError::CommandFailed {
                status,
                error: self.command.read_error(),
            });
        }

        Ok(())
    }

    fn validate_presence(status: Status) -> Result<(), AtaError> {
        if status.is_zero() {
            return Err(AtaError::NoDevice);
        }

        if status.is_floating_bus() {
            return Err(AtaError::FloatingBus);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtaDeviceError {
    Ata(AtaError),
    InvalidSectorCount(u64),
    InvalidSectorSize(u32),
    LbaOutOfRange, // TODO: Add value
    LbaUnsupported,
    Lba48Unsupported,
}

#[derive(Debug)]
pub struct AtaDevice {
    channel: ChannelId,
    device: DeviceId,
    sector_size: u32,
    sector_count: u64,
    supports_lba48: bool,
}

impl AtaDevice {
    pub fn probe(channel: ChannelId, device: DeviceId) -> Result<Self, AtaDeviceError> {
        let identify = {
            let mut channel = match channel {
                ChannelId::Primary => PRIMARY_CHANNEL.lock(),
                ChannelId::Secondary => SECONDARY_CHANNEL.lock(),
            };

            channel.identify(device).map_err(AtaDeviceError::Ata)?
        };

        Self::from_identify(channel, device, &identify)
    }

    pub fn from_identify(
        channel: ChannelId,
        device: DeviceId,
        identify: &IdentifyData,
    ) -> Result<Self, AtaDeviceError> {
        let sector_size = identify.logical_sector_size();

        if sector_size == 0 || !sector_size.is_multiple_of(2) {
            return Err(AtaDeviceError::InvalidSectorSize(sector_size));
        }

        let sector_count = identify
            .logical_sector_count()
            .ok_or(AtaDeviceError::LbaUnsupported)?;

        if sector_count == 0 {
            return Err(AtaDeviceError::InvalidSectorCount(sector_count));
        }

        Ok(Self {
            channel,
            device,
            sector_size,
            sector_count,
            supports_lba48: identify.supports_lba48(),
        })
    }

    pub fn read_sectors(&self, lba: Lba48, buffer: &mut [u16]) -> Result<(), AtaDeviceError> {
        let words_per_sector = self.sector_size as usize / 2;

        assert!(!buffer.is_empty());
        assert!(buffer.len().is_multiple_of(words_per_sector));

        let sector_count = buffer.len() / words_per_sector;
        let sector_count = u32::try_from(sector_count).expect("ATA sector count exceeds u32");

        if sector_count <= 256
            && let Some(lba28) = lba.as_lba28()
            && lba28.is_range_addressable(sector_count as u16)
        {
            return self.read_sectors_lba28(lba28, sector_count as u16, buffer);
        }

        if !self.supports_lba48 {
            return Err(AtaDeviceError::Lba48Unsupported);
        }

        self.read_sectors_lba48(lba, sector_count, buffer)
    }

    pub fn write_sectors(&mut self, lba: Lba48, buffer: &[u16]) -> Result<(), AtaDeviceError> {
        let words_per_sector = self.sector_size as usize / 2;

        assert!(!buffer.is_empty());
        assert!(buffer.len().is_multiple_of(words_per_sector));

        let sector_count = buffer.len() / words_per_sector;
        let sector_count = u32::try_from(sector_count).expect("ATA sector count exceeds u32");

        if sector_count <= 256
            && let Some(lba28) = lba.as_lba28()
            && lba28.is_range_addressable(sector_count as u16)
        {
            return self.write_sectors_lba28(lba28, sector_count as u16, buffer);
        }

        if !self.supports_lba48 {
            return Err(AtaDeviceError::Lba48Unsupported);
        }

        self.write_sectors_lba48(lba, sector_count, buffer)
    }

    pub fn flush(&mut self) -> Result<(), AtaDeviceError> {
        let mut channel = match self.channel {
            ChannelId::Primary => PRIMARY_CHANNEL.lock(),
            ChannelId::Secondary => SECONDARY_CHANNEL.lock(),
        };

        // TODO: This is not correct
        if self.supports_lba48 {
            channel
                .flush_cache_ext(self.device)
                .map_err(AtaDeviceError::Ata)
        } else {
            channel
                .flush_cache(self.device)
                .map_err(AtaDeviceError::Ata)
        }
    }

    fn read_sectors_lba28(
        &self,
        lba: Lba28,
        sector_count: u16,
        buffer: &mut [u16],
    ) -> Result<(), AtaDeviceError> {
        assert!(sector_count >= 1 && sector_count <= 256);

        let words_per_sector = self.sector_size as usize / 2;

        assert!(buffer.len() == words_per_sector * sector_count as usize);

        if !lba.is_range_addressable_on_device(sector_count, self.sector_count) {
            return Err(AtaDeviceError::LbaOutOfRange);
        }

        let mut channel = match self.channel {
            ChannelId::Primary => PRIMARY_CHANNEL.lock(),
            ChannelId::Secondary => SECONDARY_CHANNEL.lock(),
        };

        channel
            .read_sectors_lba28(self.device, lba, sector_count, words_per_sector, buffer)
            .map_err(AtaDeviceError::Ata)
    }

    fn read_sectors_lba48(
        &self,
        lba: Lba48,
        sector_count: u32,
        buffer: &mut [u16],
    ) -> Result<(), AtaDeviceError> {
        assert!(sector_count >= 1 && sector_count <= 65_536);

        let words_per_sector = self.sector_size as usize / 2;

        assert!(buffer.len() == words_per_sector * sector_count as usize);

        if !lba.is_range_addressable_on_device(sector_count, self.sector_count) {
            return Err(AtaDeviceError::LbaOutOfRange);
        }

        let mut channel = match self.channel {
            ChannelId::Primary => PRIMARY_CHANNEL.lock(),
            ChannelId::Secondary => SECONDARY_CHANNEL.lock(),
        };

        channel
            .read_sectors_lba48(self.device, lba, sector_count, words_per_sector, buffer)
            .map_err(AtaDeviceError::Ata)
    }

    fn write_sectors_lba28(
        &self,
        lba: Lba28,
        sector_count: u16,
        buffer: &[u16],
    ) -> Result<(), AtaDeviceError> {
        assert!(sector_count >= 1 && sector_count <= 256);

        let words_per_sector = self.sector_size as usize / 2;

        assert!(buffer.len() == words_per_sector * sector_count as usize);

        if !lba.is_range_addressable_on_device(sector_count, self.sector_count) {
            return Err(AtaDeviceError::LbaOutOfRange);
        }

        let mut channel = match self.channel {
            ChannelId::Primary => PRIMARY_CHANNEL.lock(),
            ChannelId::Secondary => SECONDARY_CHANNEL.lock(),
        };

        channel
            .write_sectors_lba28(self.device, lba, sector_count, words_per_sector, buffer)
            .map_err(AtaDeviceError::Ata)
    }

    fn write_sectors_lba48(
        &self,
        lba: Lba48,
        sector_count: u32,
        buffer: &[u16],
    ) -> Result<(), AtaDeviceError> {
        assert!(sector_count >= 1 && sector_count <= 65_536);

        let words_per_sector = self.sector_size as usize / 2;

        assert!(buffer.len() == words_per_sector * sector_count as usize);

        if !lba.is_range_addressable_on_device(sector_count, self.sector_count) {
            return Err(AtaDeviceError::LbaOutOfRange);
        }

        let mut channel = match self.channel {
            ChannelId::Primary => PRIMARY_CHANNEL.lock(),
            ChannelId::Secondary => SECONDARY_CHANNEL.lock(),
        };

        channel
            .write_sectors_lba48(self.device, lba, sector_count, words_per_sector, buffer)
            .map_err(AtaDeviceError::Ata)
    }
}

pub fn controllers() -> impl Iterator<Item = FunctionAddress> {
    PciIterator::new().filter(|&function| {
        let class = get_class(function);

        class.class() == MASS_STORAGE_CLASS && class.subclass() == IDE_SUBCLASS
    })
}
