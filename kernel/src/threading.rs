use spin::Mutex;

use crate::interrupts::without_interrupts;

pub fn with_lock_no_interrupts<T, F, R>(lock: &Mutex<T>, f: F) -> R
where
    F: FnOnce() -> R,
{
    without_interrupts(|| {
        let _guard = lock.lock();
        f()
    })
}
