use core::time::Duration;

use spin::Mutex;

use crate::interrupts;
use crate::time;
use crate::userspace::context::UserContext;
use crate::userspace::process::{Process, ProcessState};

pub const MAX_PROCESSES: usize = 32;

static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

struct Scheduler {
    processes: [Option<Process>; MAX_PROCESSES],
    process_count: usize,
    running: Option<usize>,
    last_scheduled: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerError {
    ProcessLimitReached,
}

impl Scheduler {
    const fn new() -> Self {
        Self {
            processes: [const { None }; MAX_PROCESSES],
            process_count: 0,
            running: None,
            last_scheduled: None,
        }
    }

    fn add(&mut self, process: Process) -> Result<(), SchedulerError> {
        if self.process_count == MAX_PROCESSES {
            return Err(SchedulerError::ProcessLimitReached);
        }

        self.processes[self.process_count] = Some(process);
        self.process_count += 1;

        Ok(())
    }

    fn yield_running(&mut self) {
        self.transition_running(ProcessState::Ready);
    }

    fn sleep_running(&mut self, deadline: Duration) {
        self.transition_running(ProcessState::Sleeping(deadline));
    }

    fn capture_running(&mut self, context: &UserContext) {
        let index = self.running.expect("no process is running");

        let process = self.processes[index]
            .as_mut()
            .expect("running process is missing");

        assert_eq!(
            process.state(),
            ProcessState::Running,
            "running process has invalid state"
        );

        process.extended_state_mut().save();
        process.context_mut().clone_from(context);
    }

    fn prepare_next(&mut self) -> Option<UserContext> {
        assert!(self.running.is_none(), "a process is already running");

        self.wake_sleeping_processes();

        let index = self.next_ready_index()?;

        let context = {
            let process = self.processes[index]
                .as_mut()
                .expect("ready process is missing");

            assert_eq!(
                process.state(),
                ProcessState::Ready,
                "selected process is not ready"
            );

            process.set_state(ProcessState::Running);
            process.activate_address_space();
            process.extended_state().restore();

            process.context().clone()
        };

        self.running = Some(index);
        self.last_scheduled = Some(index);

        Some(context)
    }

    fn transition_running(&mut self, state: ProcessState) {
        assert_ne!(
            state,
            ProcessState::Running,
            "running process cannot transition to Running"
        );

        let index = self.running.expect("no process is running");

        let process = self.processes[index]
            .as_mut()
            .expect("running process is missing");

        assert_eq!(
            process.state(),
            ProcessState::Running,
            "running process has invalid state"
        );

        process.set_state(state);
        self.running = None;
    }

    fn next_ready_index(&self) -> Option<usize> {
        if self.process_count == 0 {
            return None;
        }

        let start = self
            .last_scheduled
            .map_or(0, |index| (index + 1) % self.process_count);

        for offset in 0..self.process_count {
            let index = (start + offset) % self.process_count;

            if self.processes[index]
                .as_ref()
                .is_some_and(|process| process.state() == ProcessState::Ready)
            {
                return Some(index);
            }
        }

        None
    }

    fn wake_sleeping_processes(&mut self) {
        let now = time::now();

        for process in self.processes[..self.process_count].iter_mut().flatten() {
            if matches!(
                process.state(),
                ProcessState::Sleeping(deadline) if deadline <= now
            ) {
                process.set_state(ProcessState::Ready);
            }
        }
    }
}

pub fn add(process: Process) -> Result<(), SchedulerError> {
    interrupts::without_interrupts(|| SCHEDULER.lock().add(process))
}

pub(crate) fn capture_running(context: &UserContext) {
    SCHEDULER.lock().capture_running(context);
}

pub(crate) fn prepare_next() -> Option<UserContext> {
    SCHEDULER.lock().prepare_next()
}

pub(crate) fn yield_current() {
    SCHEDULER.lock().yield_running();
}

pub(crate) fn sleep_current(deadline: Duration) {
    SCHEDULER.lock().sleep_running(deadline);
}
