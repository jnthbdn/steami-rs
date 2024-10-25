#![no_main]
#![no_std]

use cortex_m::{self, delay::Delay};
use cortex_m_rt::entry;
// These lines are part of our setup for debug printing.
use defmt_rtt as _;
use panic_probe as _;
use steami_rs::STeaMi;

#[entry]
fn main() -> ! {
    // This line is required to prevent the debugger from disconnecting on entering WFI.
    // This appears to be a limitation of many STM32 families. Not required in production code,
    // and significantly increases power consumption in low-power modes. Not required if not using WFI.
    // hal::debug_workaround();

    let mut steami = STeaMi::take().unwrap();
    let mut delay = Delay::new(steami.cortex_peripherals.SYST, steami.clocks.sysclk());

    loop {
        steami.leds.user_red.set_high();
        delay.delay_ms(500);
        steami.leds.user_red.set_low();
        steami.leds.user_green.set_high();
        delay.delay_ms(500);
        steami.leds.user_green.set_low();
        steami.leds.user_blue.set_high();
        delay.delay_ms(500);

        // low_power::sleep_now();
        // cortex_m::asm::nop();
    }
}

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}
