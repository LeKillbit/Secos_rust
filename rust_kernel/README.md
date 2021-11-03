# Rust Version of SECOS

The build system is also in rust.  

To start the kernel, run `cargo run kvm` or `cargo run qemu`.  

To clean generated files, run `cargo run clean`.  

## Organization of the project

* `src/main.rs` : script to run the kernel in qemu/kvm
* `build.rs` : script to build the kernel
* `kernel_core/src/*` : code of the kernel
* `kernel_core/utils/linker.lds` : linker script, comes from the original secos


