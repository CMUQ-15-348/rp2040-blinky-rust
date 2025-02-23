//! Blinks an LED on GPIO15 of a Pico board
//! by directly writing to the RP2040 registers.
//!
//! This example bypasses any Board Support Package (BSP) abstractions.
//!
//! **WARNING:** All register addresses and bit‐fields are taken from the RP2040 datasheet.
//! Double‑check them if you move to a different revision or variant.

#![no_std]
#![no_main]

use core::ptr::{read_volatile, write_volatile};
use cortex_m::delay::Delay;
use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

// We still use the HAL's clock setup, watchdog, etc.
use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};
use rp_pico as bsp;

// The Pin to deal with in this file
const PIN: u32 = 15;

// Some base addresses for pointers...
const PADS_BANK0_BASE: u32 = 0x4001_C000_u32;
const IO_BANK0_BASE: u32 = 0x4001_4000_u32;
const SIO_BASE: u32 = 0xD000_0000_u32;

// Some helper functions to directly read/write registers.
// The are unsafe because they dereference raw pointers.
// The are volatile because the compiler should not optimize them away.
fn read_reg(addr: u32) -> u32 {
    unsafe { read_volatile(addr as *const u32) }
}

fn write_reg(addr: u32, value: u32) {
    unsafe {
        write_volatile(addr as *mut u32, value);
    }
}

#[entry]
fn main() -> ! {
    info!("Program start (low-level register access)");

    // Get the device and core peripherals.
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // We aren't going to use these, but for some reason we need to do this
    // to fix some sort of bug (or omission) in the RP2040 bootloader that Rust uses.
    // Without this, our writes to GPIO??_CTRL don't work. I have no idea why.
    let sio = Sio::new(pac.SIO);
    let _pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Set up the watchdog and clocks as usual.
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // Configure the pads.  Writing 0 disables input and enables output.
    write_reg(PADS_BANK0_BASE + (PIN + 1) * 4, 0);

    // Configure IO_BANK0: Set GPIO??_CTRL.funcsel = 5, which selects SIO control.
    // The IO_BANK0 peripheral base address is 0x4001_4000. According to the datasheet,
    // each GPIO has 8 bytes of registers. For example, the GPIO15 CTRL register is located at:
    //   offset = (15 * 8) + 4 = 124 (0x7C)
    write_reg(IO_BANK0_BASE + (PIN * 8 + 4), 5);

    // Configure SIO: Enable output for GPIO??.
    // The SIO peripheral base address is 0xD000_0000.
    // The GPIO_OE_SET register is at offset 0x024.
    // We first need to enable the output driver for GPIO??.
    write_reg(SIO_BASE + 0x024, 1 << PIN);

    loop {
        // Turn LED "on": Set GPIO15 high.  The GPIO_OUT_SET register is at offset 0x014.
        info!("LED on");
        write_reg(SIO_BASE + 0x014, 1 << PIN);
        delay.delay_ms(500);

        // Turn LED "off": Clear GPIO15 high. The GPIO_OUT_CLR register is at offset 0x018.
        info!("LED off");
        write_reg(SIO_BASE + 0x018, 1 << PIN);
        delay.delay_ms(500);
    }
}
