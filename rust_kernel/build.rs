use std::process::Command;
use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {

    println!("cargo:rerun-if-changed=kernel_core/*");

    let build_dir = Path::new("build");

    std::fs::create_dir_all(build_dir)?;

    let entry = Path::new("kernel_core/src/entry.asm");
    if !Command::new("nasm").args(
            &["-f", "elf32", entry.to_str().unwrap(),
            "-o", build_dir.join("entry.o").to_str().unwrap()]
        ).status()?.success() {
        return Err("Failed to assemble entry".into());
    }

    if !Command::new("cargo")
        .current_dir("kernel_core")
        .args(
            &["build", "--release", 
            "--target-dir", build_dir.canonicalize()?.to_str().unwrap()]
        ).status()?.success() {
        return Err("Failed to compile kernel".into());
    }

    if !Command::new("ld").args(
            &["-melf_i386", "--warn-common", "--no-check-sections", "-n",
            "--gc-sections", "-T", "kernel_core/utils/linker.lds",
            build_dir.join("entry.o").to_str().unwrap(),
            build_dir.join("kernel_target32")
                .join("release")
                .join("libkernel_core.a").to_str().unwrap(),
            "-o", build_dir.join("kernel.elf").to_str().unwrap()]
        ).status()?.success() {
        return Err("Failed to link kernel".into());
    }

    Ok(())
}
