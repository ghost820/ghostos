use core::time::Duration;

use x86_64::VirtAddr;

use crate::cpu::ExtendedProcessorState;
use crate::memory::{AddressSpace, MappingTransaction, UserPage, UserPageAccess};
use crate::userspace::context::UserContext;
use crate::userspace::loader::{
    ExecutableImage, ExecutableImageError, USER_HEAP_ADDR, USER_HEAP_LIMIT, USER_STACK_ADDR,
    USER_STACK_TOP,
};
use bootloader_api::info::FrameBuffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Running,
    Sleeping(Duration),
}

pub struct Process {
    state: ProcessState,
    context: UserContext,
    extended_state: ExtendedProcessorState,
    address_space: AddressSpace,
    framebuffer_address: VirtAddr,
}

#[derive(Debug)]
pub enum CreateProcessError {
    Executable(ExecutableImageError),
    OutOfMemory,
}

impl Process {
    pub fn new(
        executable: &ExecutableImage<'_>,
        heap_size: usize,
        framebuffer: &FrameBuffer,
    ) -> Result<Self, CreateProcessError> {
        let segments = executable
            .load_segments()
            .map_err(CreateProcessError::Executable)?;

        let heap_end = USER_HEAP_ADDR
            .checked_add(heap_size)
            .expect("user heap address overflow");

        assert!(
            heap_end <= USER_HEAP_LIMIT,
            "user heap exceeds its allowed address range"
        );

        let mut mappings = MappingTransaction::new();

        for segment in &segments {
            mappings = mappings.map_user_pages(
                UserPage::containing_address(segment.start_address),
                UserPage::containing_address(segment.end_address),
                segment.access,
            );
        }

        if heap_size != 0 {
            mappings = mappings.map_user_pages(
                UserPage::containing_address(VirtAddr::new(USER_HEAP_ADDR as u64)),
                UserPage::containing_address(VirtAddr::new((heap_end - 1) as u64)),
                UserPageAccess::ReadWrite,
            );
        }

        mappings = mappings.map_user_pages(
            UserPage::containing_address(VirtAddr::new(USER_STACK_ADDR as u64)),
            UserPage::containing_address(VirtAddr::new((USER_STACK_TOP - 1) as u64)),
            UserPageAccess::ReadWrite,
        );

        let mut address_space =
            AddressSpace::new(mappings).ok_or(CreateProcessError::OutOfMemory)?;

        // TODO: Handle OOM for additional frames required for this mapping
        let framebuffer_address = address_space
            .map_framebuffer(framebuffer)
            .expect("failed to map framebuffer");

        for segment in segments {
            address_space.write_user_memory(segment.start_address, segment.data);
        }

        let entry_point = executable.entry_point();
        let stack_pointer = VirtAddr::new((USER_STACK_TOP - size_of::<u64>()) as u64);

        let mut context = UserContext::new(entry_point, stack_pointer);
        context.rdi = framebuffer_address.as_u64();

        Ok(Self {
            state: ProcessState::Ready,
            address_space,
            extended_state: ExtendedProcessorState::new(),
            context,
            framebuffer_address,
        })
    }

    pub fn state(&self) -> ProcessState {
        self.state
    }

    pub fn set_state(&mut self, state: ProcessState) {
        self.state = state;
    }

    pub fn context(&self) -> &UserContext {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut UserContext {
        &mut self.context
    }

    pub fn extended_state(&self) -> &ExtendedProcessorState {
        &self.extended_state
    }

    pub fn extended_state_mut(&mut self) -> &mut ExtendedProcessorState {
        &mut self.extended_state
    }

    pub fn address_space_mut(&mut self) -> &mut AddressSpace {
        &mut self.address_space
    }

    pub fn activate_address_space(&self) {
        self.address_space.activate();
    }
}
