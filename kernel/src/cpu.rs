use core::arch::asm;
use core::arch::x86_64::{__cpuid, __cpuid_count};

use conquer_once::spin::OnceCell;
use x86_64::registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags};
use x86_64::registers::xcontrol::{XCr0, XCr0Flags};

static EXTENDED_STATE_CONFIGURATION: OnceCell<ExtendedStateConfiguration> = OnceCell::uninit();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExtendedStateConfiguration {
    mask: u64,
    size: usize,
}

pub fn init() {
    const PROCESSOR_FEATURES_LEAF: u32 = 0x01;
    const EXTENDED_STATE_LEAF: u32 = 0x0d;

    const XSAVE_FEATURE_BIT: u32 = 1 << 26;
    const SSE3_FEATURE_BIT: u32 = 1 << 0;
    const SSSE3_FEATURE_BIT: u32 = 1 << 9;
    const SSE4_1_FEATURE_BIT: u32 = 1 << 19;
    const SSE4_2_FEATURE_BIT: u32 = 1 << 20;
    const AVX_FEATURE_BIT: u32 = 1 << 28;
    const AVX2_FEATURE_BIT: u32 = 1 << 5;

    let maximum_basic_leaf = __cpuid(0).eax;

    assert!(
        maximum_basic_leaf >= EXTENDED_STATE_LEAF,
        "CPU does not expose extended state enumeration"
    );

    let features = __cpuid(PROCESSOR_FEATURES_LEAF);

    assert!(
        features.ecx & XSAVE_FEATURE_BIT != 0,
        "CPU does not support XSAVE"
    );

    assert!(
        features.ecx & SSE3_FEATURE_BIT != 0,
        "CPU does not support SSE3"
    );

    assert!(
        features.ecx & SSSE3_FEATURE_BIT != 0,
        "CPU does not support SSSE3"
    );

    assert!(
        features.ecx & SSE4_1_FEATURE_BIT != 0,
        "CPU does not support SSE4.1"
    );

    assert!(
        features.ecx & SSE4_2_FEATURE_BIT != 0,
        "CPU does not support SSE4.2"
    );

    unsafe {
        Cr0::update(|flags| {
            flags.insert(Cr0Flags::MONITOR_COPROCESSOR | Cr0Flags::NUMERIC_ERROR);
            flags.remove(Cr0Flags::EMULATE_COPROCESSOR | Cr0Flags::TASK_SWITCHED);
        });

        Cr4::update(|flags| {
            flags.insert(Cr4Flags::OSFXSR | Cr4Flags::OSXMMEXCPT_ENABLE | Cr4Flags::OSXSAVE);
        });
    }

    let enumeration = __cpuid_count(EXTENDED_STATE_LEAF, 0);
    let supported_mask = u64::from(enumeration.eax) | (u64::from(enumeration.edx) << 32);

    assert!(
        features.ecx & AVX_FEATURE_BIT != 0,
        "CPU does not support AVX"
    );

    let extended_feature_flags = __cpuid_count(7, 0);

    assert!(
        extended_feature_flags.ebx & AVX2_FEATURE_BIT != 0,
        "CPU does not support AVX2"
    );

    let required = XCr0Flags::X87 | XCr0Flags::SSE | XCr0Flags::AVX;

    assert!(
        supported_mask & required.bits() == required.bits(),
        "CPU does not support required extended processor state"
    );

    unsafe {
        XCr0::write_raw(required.bits());
    }

    let enumeration = __cpuid_count(EXTENDED_STATE_LEAF, 0);
    let size = enumeration.ebx as usize;

    assert!(size != 0, "invalid XSAVE area size");

    EXTENDED_STATE_CONFIGURATION
        .try_init_once(|| ExtendedStateConfiguration {
            mask: required.bits(),
            size,
        })
        .expect("extended processor state already initialized");
}

pub fn extended_state_mask() -> u64 {
    configuration().mask
}

pub fn extended_state_size() -> usize {
    configuration().size
}

fn configuration() -> &'static ExtendedStateConfiguration {
    EXTENDED_STATE_CONFIGURATION
        .try_get()
        .expect("CPU extended state not initialized")
}

const XSAVE_AREA_SIZE: usize = 832;

#[repr(C, align(64))]
struct XsaveArea([u8; XSAVE_AREA_SIZE]);

pub struct ExtendedProcessorState {
    area: XsaveArea,
}

impl ExtendedProcessorState {
    pub fn new() -> Self {
        const MXCSR_OFFSET: usize = 24;
        const INITIAL_MXCSR: u32 = 0x1f80;

        assert!(
            extended_state_size() <= XSAVE_AREA_SIZE,
            "XSAVE area exceeds supported size"
        );

        let mut area = XsaveArea([0; XSAVE_AREA_SIZE]);

        area.0[MXCSR_OFFSET..MXCSR_OFFSET + size_of::<u32>()]
            .copy_from_slice(&INITIAL_MXCSR.to_le_bytes());

        Self { area }
    }

    pub fn save(&mut self) {
        let mask = extended_state_mask();

        unsafe {
            asm!(
                "xsave64 [{}]",
                in(reg) self.area.0.as_mut_ptr(),
                in("eax") mask as u32,
                in("edx") (mask >> 32) as u32,
                options(nostack),
            );
        }
    }

    pub fn restore(&self) {
        let mask = extended_state_mask();

        unsafe {
            asm!(
                "xrstor64 [{}]",
                in(reg) self.area.0.as_ptr(),
                in("eax") mask as u32,
                in("edx") (mask >> 32) as u32,
                options(nostack),
            );
        }
    }
}

impl Default for ExtendedProcessorState {
    fn default() -> Self {
        Self::new()
    }
}
