use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let bios_image = env!("BIOS_IMAGE");

    let status = Command::new("qemu-system-x86_64")
        .args([
            "-drive",
            &format!("format=raw,file={bios_image}"),
            "-serial",
            "stdio",
            "-device",
            "isa-debug-exit,iobase=0xf4,iosize=0x04",
        ])
        .status();

    match status {
        Ok(status) if status.success() => ExitCode::SUCCESS,
        Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
        Err(err) => {
            eprintln!("failed to run qemu-system-x86_64: {err}");
            ExitCode::FAILURE
        }
    }
}
