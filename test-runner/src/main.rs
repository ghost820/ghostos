use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::process::{Command, ExitCode};
use std::thread;
use std::time::{Duration, Instant};

use bootloader::BiosBoot;

const TEST_TIMEOUT: Duration = Duration::from_secs(15);
const TEST_DISK_SIZE: u64 = 1024 * 1024;

const QEMU_TEST_SUCCESS: i32 = 33;

fn main() -> ExitCode {
    let kernel_path = PathBuf::from(
        env::args_os()
            .nth(1)
            .expect("test kernel path is unavailable"),
    );

    let temp_dir = tempfile::tempdir().expect("failed to create temporary directory");
    let image_path = temp_dir.path().join("test.img");

    let ata_image_path = temp_dir.path().join("ata-test.img");

    File::create(&ata_image_path)
        .and_then(|file| file.set_len(TEST_DISK_SIZE))
        .expect("failed to create ATA test disk");

    BiosBoot::new(&kernel_path)
        .create_disk_image(&image_path)
        .expect("failed to create test disk image");

    let mut qemu = Command::new("qemu-system-x86_64")
        .arg("-drive")
        .arg(format!("format=raw,file={}", image_path.display()))
        .arg("-drive")
        .arg(format!(
            "format=raw,file={},if=ide,index=1",
            ata_image_path.display()
        ))
        .args([
            "-device",
            "isa-debug-exit,iobase=0xf4,iosize=0x04",
            "-serial",
            "stdio",
            "-display",
            "none",
        ])
        .spawn()
        .expect("failed to start QEMU");

    let started_at = Instant::now();

    loop {
        match qemu.try_wait().expect("failed to query QEMU status") {
            Some(status) => {
                return if status.code() == Some(QEMU_TEST_SUCCESS) {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                };
            }
            None if started_at.elapsed() >= TEST_TIMEOUT => {
                qemu.kill().expect("failed to kill timed out QEMU");
                qemu.wait().expect("failed to reap timed out QEMU");

                eprintln!("test timed out after {} seconds", TEST_TIMEOUT.as_secs());

                return ExitCode::FAILURE;
            }
            None => thread::sleep(Duration::from_millis(10)),
        }
    }
}
