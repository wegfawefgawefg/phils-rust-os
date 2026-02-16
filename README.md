# phils-rust-os

Modernized rust-osdev layout (`bootloader 0.11`):

- `kernel/`: no_std kernel crate
- workspace root: host launcher that builds a BIOS disk image and runs QEMU

## Run

```bash
cargo run
```

This builds `kernel` for `x86_64-unknown-none`, creates a BIOS image via `bootloader`, then launches QEMU.
