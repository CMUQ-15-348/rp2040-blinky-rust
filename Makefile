# Makefile for RP2040 using UF2 flashing

# Variables (adjust these as needed)
TARGET = thumbv6m-none-eabi
NAME   = rp2040_blink

# On Linux, the RP2040 UF2 drive is usually mounted at /media/<user>/RPI-RP2.
# On macOS, it might be at /Volumes/RPI-RP2.
# Define FLASH_MOUNT from the environment or default to Linux's path.
FLASH_MOUNT ?= /media/$(USER)/RPI-RP2

# Default target builds the project
build:
	cargo build --release --target=$(TARGET)
	@echo "Build complete. Converting ELF to UF2..."
	elf2uf2-rs target/$(TARGET)/release/$(NAME) target/$(TARGET)/release/$(NAME).uf2

# Flash target copies the UF2 file to the mounted drive
flash: build
	cargo run --release --target=thumbv6m-none-eabi

# Clean target for convenience
clean:
	cargo clean
