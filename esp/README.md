# ESP32 Controller

Software components for the ESP32 microcontroller to handle HomeKit and blind operation.

## Installation

1. Install Rust using [rustup](https://rustup.rs/).
1. Install [ESP Rust toolchain](https://docs.esp-rs.org/book/installation/riscv-and-xtensa.html) for RIS-V and Xtensa targets.

## Development

If you see this:
```
error: linker `xtensa-esp32-elf-gcc` not found
  |
  = note: No such file or directory (os error 2)
```

Then be sure to source the `export-esp.sh` file in your shell from the ESP Rust toolchain, like so:
```sh
source ~/export-esp.sh
```

Refer to [Set Up the Environment Variables](https://docs.esp-rs.org/book/installation/riscv-and-xtensa.html#3-set-up-the-environment-variables) from the ESP-RS docs.
