## Blinky Rust with the RP2040

This repo contains a simple example to blink a PIN on the RP2040 board.  This is different from most other examples, though, because it does it by manually configuring the control registers.

## Pre-Requisite Installation Instructions

### Lab Machines

- Everything is already installed.

### Ubuntu

- Install the compiler and support tools for the Arm Cortex Processors:  
  `sudo apt install binutils-arm-none-eabi gcc-arm-none-eabi`
- Install rust: [https://rustup.rs/](https://rustup.rs/)
- Add the ARM target for the Cortex-M0 for Rust:  
  `rustup target add thumbv6m-none-eabi`
- Install probe-rs:  
  `cargo install --locked probe-rs-tools`

### Mac (I think)

- Install the compiler and support tools for the Arm Cortex Processors:  
  `brew install gcc-arm-none-eabi`
- Install rust: [https://rustup.rs/](https://rustup.rs/)
- Add the ARM target for the Cortex-M0 for Rust:  
  `rustup target add thumbv6m-none-eabi`
- Install probe-rs:  
  `cargo install --locked probe-rs-tools`

### Windows (Maybe)

- Install Windows Subsystem for Linux (WSL2) and install Ubuntu
- Open a Linux shell and follow the Ubuntu instructions above

This gets you all the software, but you can't talk to the debug probe because the USB device isn't being passed to the Linux virtual machine.  You can fix this using [WSL USB Manager](https://github.com/nickbeth/wsl-usb-manager).  You'll need to bind and attach the CMSIS-DAP USB device.

## Running Normally

Use the Raspberry Pi debugger to connect to the Pico using the little 3-pin cable.  Power the Pico over USB, and plug the debugger into your computer.

- `cargo build` to build the binary.
- `cargo run` to install and run it using `probe-rs`

While running, you should see the `INFO` messages displayed on the console.  You should also see an LED blink.  (By default this does pin 25, which is an LED on the normal Pico.)

## Debugging

This repo includes the configuration to do single-step debugging.  In order for it to work, however, you need to do the following:

- Install the cortex-debug VSCode extension.  (The defmt output won't work properly without a modified version of the extension from me.  I'll get this setup better later.)
- Install openocd 0.12 or later:  
  `sudo apt install openocd` on Ubuntu 24.04 or newer.  Older versions of Ubuntu ship an out of date version that won't work.
- Install gdb-multiarch and symlink it to `arm-none-eabi-gdb` which is what the debugging extension expects:  
  `sudo apt install gdb-multiarch`  
  `cd /usr/local/bin; ln -s /usr/bin/gdb-multiarch arm-none-eabi-gdb`

If all of that is installed, you should be able to hit 'F5' and cause a build/upload/run cycle that automatically stops at the entry.  You can also set breakpoints, single-step, etc.
