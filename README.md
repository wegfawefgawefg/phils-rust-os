# phils-rust-os

This is my Rust OS training project, originally started in late 2023 and revived in 2026.

## Context

- Initial commit: `2023-09-27` (`Initial commit`)
- Early progress in 2023: booting, terminal output, serial, basic interrupts/keyboard, VGA experiments
- 2026 revisit: migrated to modern `bootloader 0.11` flow and built a framebuffer graphics demo

This project follows Philipp Oppermann's Rust OS dev material:

- https://os.phil-opp.com/

## Current Layout

- `kernel/`: `no_std` kernel crate
- workspace root: host launcher that builds a BIOS disk image and runs QEMU

## Run

```bash
cargo run
```

This builds `kernel` for `x86_64-unknown-none`, creates a BIOS image via `bootloader`, then launches QEMU.

## Screenshot

![Raymarched SDF balls demo](image.png)
