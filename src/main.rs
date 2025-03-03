//! Blinks an LED on GPIO25 of a Pico board
//! by directly writing to the RP2040 registers.

#![no_std]
#![no_main]

use core::ptr::{read_volatile, write_volatile};
use cortex_m::delay::Delay;
use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

// The board support package (BSP) provides abstractions for the hardware.
// We use it here to set up the watchdog and clocks.
use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    watchdog::Watchdog,
};
use rp_pico as bsp;

// The Pin we want to blink.
const PIN: u32 = 25;

// Some base addresses for pointers.  These are taken directly from the datasheet.
// When we use them below, the comments will give datasheet references to them.
const PADS_BANK0_BASE: u32 = 0x4001_C000_u32;
const IO_BANK0_BASE: u32 = 0x4001_4000_u32;
const SIO_BASE: u32 = 0xD000_0000_u32;
const RESETS_BASE: u32 = 0x4000_C000_u32;

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

    // Set up the watchdog and clocks. There is a way to do this
    // that manipulates control registers, but we'll take a shortcut
    // and use the HAL's functions.
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

    // Reset, then deassert the reset on IO_BANK0
    // See Section 2.14 in the datasheet for details
    let the_bit = 1 << 5;
    write_reg(RESETS_BASE, read_reg(RESETS_BASE) | the_bit); // Write 1 to reset
    write_reg(RESETS_BASE, read_reg(RESETS_BASE) & !the_bit); // Write 0 to deassert reset

    // Reset, then deassert the reset on PADS_BANK0
    // See Section 2.14 in the datasheet for details
    let the_bit = 1 << 8;
    write_reg(RESETS_BASE, read_reg(RESETS_BASE) | the_bit); // Write 1 to reset
    write_reg(RESETS_BASE, read_reg(RESETS_BASE) & !the_bit); // Write 0 to deassert reset

    // Configure the pads.  Writing 0 disables input and enables output for that pad.
    // See Table 339 and Table 341 in the datasheet for details
    write_reg(PADS_BANK0_BASE + (PIN + 1) * 4, 0);

    // Configure IO_BANK0: Set GPIO??_CTRL.funcsel = 5, which selects SIO control.
    // The IO_BANK0 peripheral base address is 0x4001_4000. According to the datasheet,
    // each GPIO has 8 bytes of registers. For example, the GPIO15 CTRL register is located at:
    //   offset = (15 * 8) + 4 = 124 (0x7C)
    // See Table 283, Table 285, and Table 279 in the datasheet for details
    write_reg(IO_BANK0_BASE + (PIN * 8 + 4), 5);

    // Configure SIO: Enable output for GPIO??.
    // The SIO peripheral base address is 0xD000_0000.
    // The GPIO_OE_SET register is at offset 0x024.
    // We first need to enable the output driver for GPIO??.
    // See Table 16 and Table 25 in the datasheet for details
    write_reg(SIO_BASE + 0x024, read_reg(SIO_BASE + 0x024) | 1 << PIN);

    loop {
        // Turn LED "on": Set GPIO?? high.  The GPIO_OUT_SET register is at offset 0x014.
        // See Table 16 and Table 21 in the datasheet for details
        info!("LED on");
        write_reg(SIO_BASE + 0x014, 1 << PIN);
        delay.delay_ms(500);

        // Turn LED "off": Clear GPIO?? high. The GPIO_OUT_CLR register is at offset 0x018.
        // See Table 16 and Table 21 in the datasheet for details
        info!("LED off");
        write_reg(SIO_BASE + 0x018, 1 << PIN);
        delay.delay_ms(500);
    }
}
