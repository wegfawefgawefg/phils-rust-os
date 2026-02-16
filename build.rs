use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=kernel/src");
    println!("cargo:rerun-if-changed=kernel/Cargo.toml");

    let kernel_path = env::vars_os()
        .find_map(|(k, v)| {
            let key = k.to_string_lossy();
            if key.starts_with("CARGO_BIN_FILE_KERNEL_") {
                Some(PathBuf::from(v))
            } else {
                None
            }
        })
        .expect("kernel artifact path env var not found");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR not set"));
    let bios_path = out_dir.join("phils-rust-os-bios.img");

    let mut boot_config = bootloader::BootConfig::default();
    boot_config.frame_buffer.minimum_framebuffer_width = Some(800);
    boot_config.frame_buffer.minimum_framebuffer_height = Some(600);
    boot_config.frame_buffer_logging = false;

    let mut bios = bootloader::BiosBoot::new(&kernel_path);
    bios.set_boot_config(&boot_config);
    bios.create_disk_image(&bios_path)
        .expect("failed to create BIOS disk image");

    println!("cargo:rustc-env=BIOS_IMAGE={}", bios_path.display());
}
