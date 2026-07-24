use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let status = Command::new("qemu-system-x86_64")
        .args([
            "-drive",
            concat!("format=raw,file=", env!("IMAGE_PATH")),
            "-serial",
            "stdio",
            "-device",
            "e1000,netdev=net0",
            "-netdev",
            "user,id=net0",
        ])
        .status()
        .expect("failed to start QEMU");

    ExitCode::from(status.code().unwrap_or(254) as u8)
}
