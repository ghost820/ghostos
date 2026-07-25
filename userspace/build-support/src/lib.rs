use std::path::Path;

pub fn configure() {
    let linker_script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../linker.ld")
        .canonicalize()
        .expect("unable to locate userspace linker script");

    println!("cargo::rerun-if-changed={}", linker_script.display());
    println!("cargo::rustc-link-arg=-T{}", linker_script.display());
}
