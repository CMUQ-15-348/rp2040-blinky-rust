/* Blinks an LED on GPIO25 of a Pico board
 * by directly writing to the RP2040 registers.
 */
#![no_std]
#![no_main]

/* Some imports of modules written by others */
use core::ptr::{read_volatile, write_volatile};
use cortex_m;
use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

/* The Pin we want to blink. GPIO25 is the onboard LED for the pico */
const PIN: u32 = 25;

/* The stage 2 bootloader.
 * This is taken from https://github.com/rp-rs/rp2040-boot2/blob/main/bin/boot2_w25q080.padded.bin
 * xxd --include <file.bin> can format the bytes for you.
 */
#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = [
    0x00, 0xb5, 0x32, 0x4b, 0x21, 0x20, 0x58, 0x60, 0x98, 0x68, 0x02, 0x21, 0x88, 0x43, 0x98, 0x60,
    0xd8, 0x60, 0x18, 0x61, 0x58, 0x61, 0x2e, 0x4b, 0x00, 0x21, 0x99, 0x60, 0x02, 0x21, 0x59, 0x61,
    0x01, 0x21, 0xf0, 0x22, 0x99, 0x50, 0x2b, 0x49, 0x19, 0x60, 0x01, 0x21, 0x99, 0x60, 0x35, 0x20,
    0x00, 0xf0, 0x44, 0xf8, 0x02, 0x22, 0x90, 0x42, 0x14, 0xd0, 0x06, 0x21, 0x19, 0x66, 0x00, 0xf0,
    0x34, 0xf8, 0x19, 0x6e, 0x01, 0x21, 0x19, 0x66, 0x00, 0x20, 0x18, 0x66, 0x1a, 0x66, 0x00, 0xf0,
    0x2c, 0xf8, 0x19, 0x6e, 0x19, 0x6e, 0x19, 0x6e, 0x05, 0x20, 0x00, 0xf0, 0x2f, 0xf8, 0x01, 0x21,
    0x08, 0x42, 0xf9, 0xd1, 0x00, 0x21, 0x99, 0x60, 0x1b, 0x49, 0x19, 0x60, 0x00, 0x21, 0x59, 0x60,
    0x1a, 0x49, 0x1b, 0x48, 0x01, 0x60, 0x01, 0x21, 0x99, 0x60, 0xeb, 0x21, 0x19, 0x66, 0xa0, 0x21,
    0x19, 0x66, 0x00, 0xf0, 0x12, 0xf8, 0x00, 0x21, 0x99, 0x60, 0x16, 0x49, 0x14, 0x48, 0x01, 0x60,
    0x01, 0x21, 0x99, 0x60, 0x01, 0xbc, 0x00, 0x28, 0x00, 0xd0, 0x00, 0x47, 0x12, 0x48, 0x13, 0x49,
    0x08, 0x60, 0x03, 0xc8, 0x80, 0xf3, 0x08, 0x88, 0x08, 0x47, 0x03, 0xb5, 0x99, 0x6a, 0x04, 0x20,
    0x01, 0x42, 0xfb, 0xd0, 0x01, 0x20, 0x01, 0x42, 0xf8, 0xd1, 0x03, 0xbd, 0x02, 0xb5, 0x18, 0x66,
    0x18, 0x66, 0xff, 0xf7, 0xf2, 0xff, 0x18, 0x6e, 0x18, 0x6e, 0x02, 0xbd, 0x00, 0x00, 0x02, 0x40,
    0x00, 0x00, 0x00, 0x18, 0x00, 0x00, 0x07, 0x00, 0x00, 0x03, 0x5f, 0x00, 0x21, 0x22, 0x00, 0x00,
    0xf4, 0x00, 0x00, 0x18, 0x22, 0x20, 0x00, 0xa0, 0x00, 0x01, 0x00, 0x10, 0x08, 0xed, 0x00, 0xe0,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x74, 0xb2, 0x4e, 0x7a,
];

/* Some helper functions to directly read/write registers.
 * The are unsafe because they dereference raw pointers.
 * The are volatile because the compiler should not optimize them away.
 */
pub fn read_reg(addr: u32) -> u32 {
    unsafe { read_volatile(addr as *const u32) }
}

pub fn write_reg(addr: u32, value: u32) {
    unsafe {
        write_volatile(addr as *mut u32, value);
    }
}

pub fn set_bits(addr: u32, mask: u32) {
    unsafe {
        // Atomic set on write.  See 2.1.2 in the datasheet
        write_volatile((addr + 0x2000) as *mut u32, mask);
    }
}

pub fn clear_bits(addr: u32, mask: u32) {
    unsafe {
        // Atomic clear on write.  See 2.1.2 in the datasheet
        write_volatile((addr + 0x3000) as *mut u32, mask);
    }
}

/* Some control register addresses that we need.  These come from the
 * datasheet. */
pub const RESETS_BASE: u32 = 0x4000_c000_u32;
pub const PADS_BANK0_BASE: u32 = 0x4001_c000_u32;
pub const IO_BANK0_BASE: u32 = 0x4001_4000_u32;
pub const SIO_BASE: u32 = 0xd000_0000_u32;
pub const XOSC_BASE: u32 = 0x4002_4000_u32;
pub const CLOCKS_BASE: u32 = 0x4000_8000_u32;
pub const PLL_SYS_BASE: u32 = 0x4002_8000_u32;
pub const PLL_USB_BASE: u32 = 0x4002_c000_u32;
pub const WATCHDOG_BASE: u32 = 0x4005_8000_u32;
pub const TIMER_BASE: u32 = 0x4005_4000_u32;

/*
 * Configure the system clock to 125 MHz.
 * There is a nice reference in the SDK:
 * https://github.com/raspberrypi/pico-sdk/blob/ee68c78d0afae2b69c03ae1a72bf5cc267a2d94c/src/rp2_common/pico_runtime_init/runtime_init_clocks.c#L40
 *
 * In the early weeks of the class, you are not required to understand what
 * this code does. Once we cover the clocks, you should be able to follow
 * along here while using the datasheet as a reference.
 *
 * This use of this code is optional.  If you don't do it, then the default
 * system clock comes from the ring oscillator, which is about 6 Mhz.
 */
pub fn init_clocks() {
    // Enable the XOSC (2.16.7)
    write_reg(XOSC_BASE + 0x00, 0x00fabaa0);
    while read_reg(XOSC_BASE + 0x04) & 0x8000_0000_u32 == 0 {
        // Wait for the XOSC to be ready
    }

    // Set the CLK_REF glitchless mux to 2 (Crystal oscillator)
    write_reg(CLOCKS_BASE + 0x30, 2);
    while read_reg(CLOCKS_BASE + 0x30) != 2 {
        // Wait for the glitchless mux to be set to 2
    }

    // Set the CLK_SYS glitchless mux to 0 (CLK_REF) so that we can mess with the
    // CLK_SYS sources without causing issues.
    clear_bits(CLOCKS_BASE + 0x3c, 0x0000_0001);
    while read_reg(CLOCKS_BASE + 0x3c) & 0x0000_0001 != 0 {
        // Wait for the glitchless mux to be set to 0
    }

    // Reset, then deassert the reset on PLL_SYS
    // See Section 2.14 in the datasheet for details
    set_bits(RESETS_BASE, 1 << 12); // Write 1 to reset
    clear_bits(RESETS_BASE, 1 << 12); // Write 0 to deassert reset

    // Reset, then deassert the reset on PLL_USB
    // See Section 2.14 in the datasheet for details
    set_bits(RESETS_BASE, 1 << 13); // Write 1 to reset
    clear_bits(RESETS_BASE, 1 << 13); // Write 0 to deassert reset

    // Disable the PLL while we reconfigure it
    write_reg(PLL_SYS_BASE + 0x04, 1 << 5 | 1); // Set the PD and VCO bits to 1 to disable the PLL

    // Configure the PLL System divider (See 2.18.2.1)
    write_reg(PLL_SYS_BASE + 0x08, 125); // fbdiv = 125
    write_reg(PLL_SYS_BASE + 0x0C, 0x0006_2000); // PD1=6, PD2=2
    clear_bits(PLL_SYS_BASE + 0x04, 0x0000_0011); // Clear the PD and VCO bits to enable the PLL
    while read_reg(PLL_SYS_BASE + 0x00) & 0x8000_0000 == 0 {
        // Wait for the PLL to be ready
    }

    // Configure the PLL USB divider
    write_reg(PLL_USB_BASE + 0x08, 100); // fbdiv = 100
    write_reg(PLL_USB_BASE + 0x0C, 0x0005_5000); // PD1=5, PD2=5
    clear_bits(PLL_USB_BASE + 0x04, 0x0000_0011); // Clear the PD and VCO bits to enable the PLL
    while read_reg(PLL_USB_BASE + 0x00) & 0x8000_0000 == 0 {
        // Wait for the PLL to be ready
    }

    // Configure the CLK_SYS_CTRL register to configure the muxes and ultimately set
    // CLK_SYS to the PLL.  (2.15.3.2) Set the aux mux to PLL_SYS (0 written to
    // bits 5-7).  This is only safe to do right now because we set the glitchless
    // mux to 0 above.
    write_reg(
        CLOCKS_BASE + 0x3c,
        read_reg(CLOCKS_BASE + 0x3c) & !(0b111 << 5),
    );
    set_bits(CLOCKS_BASE + 0x3c, 0x0000_0001); // Set the glitchless mux to 1 (CLKSRC_CLK_SYS_AUX) so that we now use the PLL
                                               // coming in on AUX.

    // Set the peripheral clock to be the same as clk_sys
    write_reg(CLOCKS_BASE + 0x48, 0); // Disable the clock by clearing bit 11.  This also sets AUXSRC to 0, which is
                                      // CLK_SYS
    let _ = read_reg(CLOCKS_BASE); // Read the register just to stall for some cycles (we're waiting to make sure
                                   // the clock peripheral clock is actually stopped)
    write_reg(CLOCKS_BASE + 0x48, 1 << 11); // Enable it

    // Configure the watchdog tick counter so that it divides by 12, leading to one
    // tick every us.  (Because the XOSC is 12MHz.) Without this being set
    // properly, the TIMER doesn't count at the correct interval.
    write_reg(WATCHDOG_BASE + 0x2c, 12 | 1 << 9); // Set the divider to 12 and enable the watchdog

    // Configure the timer not to pause during debugging. Otherwise, the watchdog timer always returns 0
    // See...
    // https://github.com/raspberrypi/debugprobe/issues/45
    // https://github.com/raspberrypi/pico-sdk/issues/1586
    write_reg(TIMER_BASE + 0x2c, 0);
}

/*
 * Code to initialize the pins/pads
 */
fn init_io(pin: u32) {
    // Reset, then deassert the reset on IO_BANK0
    // See Section 2.14 in the datasheet for details
    set_bits(RESETS_BASE, 1 << 5); // Write 1 to reset
    clear_bits(RESETS_BASE, 1 << 5); // Write 0 to deassert reset

    // Reset, then deassert the reset on PADS_BANK0
    // See Section 2.14 in the datasheet for details
    set_bits(RESETS_BASE, 1 << 8); // Write 1 to reset
    clear_bits(RESETS_BASE, 1 << 8); // Write 0 to deassert reset

    // Configure the pads.  Writing 0 disables input and enables output for that
    // pad. See Table 339 and Table 341 in the datasheet for details
    write_reg(PADS_BANK0_BASE + (pin + 1) * 4, 0);

    // Configure IO_BANK0: Set GPIO??_CTRL.funcsel = 5, which selects SIO control.
    // The IO_BANK0 peripheral base address is 0x4001_4000. According to the
    // datasheet, each GPIO has 8 bytes of registers. For example, the GPIO15
    // CTRL register is located at:   offset = (15 * 8) + 4 = 124 (0x7C)
    // See Table 283, Table 285, and Table 279 in the datasheet for details
    write_reg(IO_BANK0_BASE + (pin * 8 + 4), 5);

    // Configure SIO: Enable output for GPIO??.
    // The SIO peripheral base address is 0xD000_0000.
    // The GPIO_OE_SET register is at offset 0x024.
    // We first need to enable the output driver for GPIO??.
    // See Table 16 and Table 25 in the datasheet for details
    write_reg(SIO_BASE + 0x024, 1 << pin);
}

/*
 * The actual main function.
 */
#[entry]
fn main() -> ! {
    // Initialize the clocks and IO pins
    init_clocks();
    init_io(PIN);

    let mut x = 0;
    loop {
        // Turn LED "on": Set GPIO?? high.  The GPIO_OUT_SET register is at offset
        // 0x014. See Table 16 and Table 21 in the datasheet for details
        info!("LED on {}", x);
        write_reg(SIO_BASE + 0x014, 1 << PIN);
        for _ in 0..800000 {
            cortex_m::asm::nop();
        }

        // Turn LED "off": Clear GPIO?? high. The GPIO_OUT_CLR register is at offset
        // 0x018. See Table 16 and Table 21 in the datasheet for details
        info!("LED off");
        write_reg(SIO_BASE + 0x018, 1 << PIN);
        for _ in 0..800000 {
            cortex_m::asm::nop();
        }

        x += 1;
    }
}
