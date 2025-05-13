#![no_std]
#![no_main]

// Declare that there is a lib348 library inside this project
pub mod lib348;

// Use some crates provides by others
use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
use rp2040_boot2;

// Use our own lib348 library components that is inside this project
use crate::lib348::control_registers::*;
use crate::lib348::sys_clock;

/* The stage 2 bootloader. */
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

/*
 * Code to initialize the pin/pad for a single output pin
 */
fn init_output_pin(pin: u32) {
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
    let pin = 25;
    // Initialize the clocks and IO pins
    sys_clock::init_clocks();
    init_output_pin(pin);

    let mut x = 0;
    loop {
        info!("LED on {}", x);
        write_reg(SIO_BASE + 0x014, 1 << pin);
        sys_clock::delay(1000);

        info!("LED off");
        write_reg(SIO_BASE + 0x018, 1 << pin);
        sys_clock::delay(1000);

        x += 1;
    }
}
