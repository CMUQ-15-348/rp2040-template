use crate::lib348::control_registers::*;

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

    // Set the CLK_REF glitchless mux to 2 (Ring oscillator)
    write_reg(CLOCKS_BASE + 0x30, 2);
    while read_reg(CLOCKS_BASE + 0x30) != 2 {
        // Wait for the glitchless mux to be set to 2
    }
    // Set the CLK_SYS glitchless mux to 0 (CLK_REF) so that we can mess with the CLK_SYS sources without causing issues.
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

    // Configure the CLK_SYS_CTRL register to configure the muxes and ultimately set CLK_SYS to the PLL.  (2.15.3.2)
    // Set the aux mux to PLL_SYS (0 written to bits 5-7).  This is only safe to do right now because we set the glitchless mux to 0 above.
    write_reg(
        CLOCKS_BASE + 0x3c,
        read_reg(CLOCKS_BASE + 0x3c) & !(0b111 << 5),
    );
    set_bits(CLOCKS_BASE + 0x3c, 0x0000_0001); // Set the glitchless mux to 1 (CLKSRC_CLK_SYS_AUX) so that we now use the PLL coming in on AUX.

    // Set the peripheral clock to be the same as clk_sys
    write_reg(CLOCKS_BASE + 0x48, 0); // Disable the clock by clearing bit 11.  This also sets AUXSRC to 0, which is CLK_SYS
    let _ = read_reg(CLOCKS_BASE); // Read the register just to stall for some cycles (we're waiting to make sure the clock peripheral clock is actually stopped)
    write_reg(CLOCKS_BASE + 0x48, 1 << 11); // Enable it

    // Configure the watchdog tick counter so that it divides by 12, leading to one tick every us.  (Because the XOSC is 12MHz.)
    // Without this being set properly, the TIMER doesn't count at the correct interval.
    write_reg(WATCHDOG_BASE + 0x2c, 12 | 1 << 9); // Set the divider to 12 and enable the watchdog

    // Configure the timer not to pause during debugging. Otherwise, we can't single step through a delay()
    // See...
    // https://github.com/raspberrypi/debugprobe/issues/45
    // https://github.com/raspberrypi/pico-sdk/issues/1586
    write_reg(TIMER_BASE + 0x2c, 0);
}
