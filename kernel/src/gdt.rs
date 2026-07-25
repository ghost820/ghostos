use lazy_static::lazy_static;

use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::{PrivilegeLevel, VirtAddr};

const TSS_STACK_SIZE: usize = 4096 * 5;

#[repr(align(16))]
struct TssStack(#[allow(dead_code)] [u8; TSS_STACK_SIZE]);

struct Selectors {
    kernel_code: SegmentSelector,
    kernel_data: SegmentSelector,
    user_data: SegmentSelector,
    user_code: SegmentSelector,
    tss: SegmentSelector,
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        let kernel_code = gdt.append(Descriptor::kernel_code_segment());
        let kernel_data = gdt.append(Descriptor::kernel_data_segment());

        let mut user_data = gdt.append(Descriptor::user_data_segment());
        let mut user_code = gdt.append(Descriptor::user_code_segment());

        user_data.set_rpl(PrivilegeLevel::Ring3);
        user_code.set_rpl(PrivilegeLevel::Ring3);

        let tss = gdt.append(Descriptor::tss_segment(&TSS));

        let selectors = Selectors {
            kernel_code,
            kernel_data,
            user_data,
            user_code,
            tss,
        };

        (gdt, selectors)
    };
}

lazy_static! {
    static ref TSS: TaskStateSegment = {
        static mut INTERRUPT_STACK: TssStack = TssStack([0; TSS_STACK_SIZE]);
        static mut PRIVILEGE_STACK: TssStack = TssStack([0; TSS_STACK_SIZE]);

        let mut tss = TaskStateSegment::new();

        let interrupt_stack_start = VirtAddr::from_ptr(&raw const INTERRUPT_STACK);
        tss.interrupt_stack_table[0] = interrupt_stack_start + TSS_STACK_SIZE as u64;

        let privilege_stack_start = VirtAddr::from_ptr(&raw const PRIVILEGE_STACK);
        tss.privilege_stack_table[0] = privilege_stack_start + TSS_STACK_SIZE as u64;

        tss
    };
}

pub fn init() {
    use x86_64::instructions::segmentation::{CS, DS, ES, SS, Segment};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();

    unsafe {
        CS::set_reg(GDT.1.kernel_code);
        DS::set_reg(GDT.1.kernel_data);
        ES::set_reg(GDT.1.kernel_data);
        SS::set_reg(GDT.1.kernel_data);
        load_tss(GDT.1.tss);
    }
}

pub fn user_code_selector() -> SegmentSelector {
    GDT.1.user_code
}

pub fn user_data_selector() -> SegmentSelector {
    GDT.1.user_data
}
