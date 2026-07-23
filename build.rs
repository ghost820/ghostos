use std::env::var_os;
use std::path::PathBuf;

use bootloader::BiosBoot;

fn main() {
    let out_dir = PathBuf::from(var_os("OUT_DIR").expect("OUT_DIR is unavailable"));

    let kernel64_path = PathBuf::from(
        var_os("CARGO_BIN_FILE_KERNEL64_kernel64").expect("kernel64 path is unavailable"),
    );

    let image_path = out_dir.join("ghostos.img");

    BiosBoot::new(&kernel64_path)
        .create_disk_image(&image_path)
        .expect("failed to create disk image");

    println!("cargo:rustc-env=IMAGE_PATH={}", image_path.display());
}
