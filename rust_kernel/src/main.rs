use std::process::Command;
use std::error::Error;
use std::path::Path;

fn spawn_qemu(kvm : bool, debug : bool) -> Result<(), Box<dyn Error>> {
    if !Command::new("cp").args(
        &["build/kernel.elf", "."]).status()?.success() {
        return Err("Couldn't find kernel.elf in build".into());
    }

    let command = match kvm {
        true => "kvm",
        false => "qemu-system-i386",
    };

    let mut args : Vec<&str> = Vec::new();
    args.extend_from_slice(
        &["-drive","media=disk,format=raw,if=floppy,file=../utils/grub.floppy",
        "-drive", "media=disk,format=raw,if=ide,index=0,file=fat:rw:.",
        "-serial", "mon:stdio",
        "-d", "int,pcall,cpu_reset,unimp,guest_errors",
        "-boot", "a",
        "-nographic"]
    );

    if debug {
        args.extend_from_slice(&["-s", "-S"]);
    }

    Command::new(command).args(
        args).status()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>>{
    let args : Vec<String> = std::env::args().collect();

    if args.len() == 2 {
        match args[1].as_str() {
            "clean" => {
                if Path::new("build").is_dir() {
                    std::fs::remove_dir_all("build")?;
                }
            }
            "qemu" => {
                spawn_qemu(false, false)?;
            }
            "kvm" => {
                spawn_qemu(true, false)?;
            }
            "debug" => {
                spawn_qemu(false, true)?;
            }
            _ => {
                return Err("usage : cargo run {qemu, kvm, clean}".into());
            }
        }
    }

    Ok(())
}
