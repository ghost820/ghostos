use heapless::Deque;
use spin::Mutex;

use crate::interrupts::without_interrupts;

pub struct ConcurrentQueue<T, const N: usize> {
    queue: Mutex<Deque<T, N>>,
}

impl<T, const N: usize> ConcurrentQueue<T, N> {
    pub const fn new() -> Self {
        Self {
            queue: Mutex::new(Deque::new()),
        }
    }

    pub fn len(&self) -> usize {
        without_interrupts(|| self.queue.lock().len())
    }

    pub fn push_back(&self, value: T) -> Result<(), T> {
        without_interrupts(|| self.queue.lock().push_back(value))
    }

    pub fn pop_front(&self) -> Option<T> {
        without_interrupts(|| self.queue.lock().pop_front())
    }

    pub fn is_empty(&self) -> bool {
        without_interrupts(|| self.queue.lock().is_empty())
    }
}

impl<T, const N: usize> Default for ConcurrentQueue<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
