#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr;
use core::time::Duration;

use game::{self, Framebuffer, State};

const FRAME_TIME: f64 = 1.0 / 30.0;

const HEAP_ADDR: usize = 0x0000_0001_0000_0000;

const FRAMEBUFFER_WIDTH: usize = 1280;
const FRAMEBUFFER_HEIGHT: usize = 720;
const FRAMEBUFFER_BPP: usize = 3;
const FRAMEBUFFER_BYTE_LEN: usize = FRAMEBUFFER_WIDTH * FRAMEBUFFER_HEIGHT * FRAMEBUFFER_BPP;

static mut BUFFER: [u8; FRAMEBUFFER_BYTE_LEN] = [0u8; FRAMEBUFFER_BYTE_LEN];

#[unsafe(no_mangle)]
pub extern "C" fn _start(framebuffer: *mut u8) -> ! {
    let game_state = HEAP_ADDR as *mut State;

    unsafe {
        ptr::write(game_state, State::default());
    }

    let game_state = unsafe { &mut *game_state };

    let mut deadline = ghostos_syscall::time_now();
    let frame_time = Duration::from_secs_f64(FRAME_TIME);

    loop {
        #[allow(static_mut_refs)]
        let game_buffer = Framebuffer::new(
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
            FRAMEBUFFER_BPP,
            unsafe { &mut BUFFER },
        );

        game::update_and_render(game_state, game_buffer);

        deadline += frame_time;
        ghostos_syscall::sleep_until(deadline);

        // TODO: Volatile?
        #[allow(static_mut_refs)]
        unsafe {
            ptr::copy_nonoverlapping(BUFFER.as_ptr(), framebuffer, FRAMEBUFFER_BYTE_LEN);
        }
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
