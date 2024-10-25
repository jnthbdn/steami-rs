#![no_main]
#![no_std]

use core::cell::RefCell;
use core::ptr::addr_of_mut;

use circular_buffer::CircularBuffer;
use cortex_m::{self, delay::Delay};
use cortex_m_rt::entry;
use critical_section::{with, Mutex};
use defmt_rtt as _;
use embedded_alloc::LlffHeap as Heap;
use hal::pac::{interrupt, NVIC};
use hal::usart::UsartInterrupt;
use jacdac_rs::brain::brain::Brain;
use jacdac_rs::service::button::{Button, ButtonState};
use jacdac_rs::transport::frame::Frame;
use panic_probe as _;
use steami_rs::{Jacdac, STeaMi};

static JACDAC_IFACE: Mutex<RefCell<Option<Jacdac>>> = Mutex::new(RefCell::new(None));
static mut JACDAC_BUF: CircularBuffer<256, u8> = CircularBuffer::new();
static mut JACDAC_FRAME: CircularBuffer<5, Frame> = CircularBuffer::new();

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[entry]
fn main() -> ! {
    // Initialize the allocator BEFORE you use it
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE) }
    }
    // This line is required to prevent the debugger from disconnecting on entering WFI.
    // This appears to be a limitation of many STM32 families. Not required in production code,
    // and significantly increases power consumption in low-power modes. Not required if not using WFI.
    // hal::debug_workaround();

    let mut steami = STeaMi::take().unwrap();
    let mut jacdac_iface = steami.jacdac;

    let mut brain = Brain::new(|| 0);

    with(|cs| {
        jacdac_iface.usart.enable_interrupt(UsartInterrupt::Idle);
        jacdac_iface
            .usart
            .enable_interrupt(UsartInterrupt::ReadNotEmpty);
        JACDAC_IFACE.borrow(cs).replace(Some(jacdac_iface));
    });

    unsafe {
        NVIC::unmask(interrupt::USART1);
    }

    loop {
        match unsafe { JACDAC_FRAME.pop_front() } {
            Some(frame) => {
                if let Err(_e) = brain.handle_frame(frame) {
                    defmt::println!("Failed to handle frame");
                }
            }
            None => (),
        };

        match brain.get_devices().first() {
            Some(device) => {
                if let Some(button) = device.get_first_service::<Button>() {
                    match button.state() {
                        ButtonState::Release => steami.leds.user_green.set_low(),
                        ButtonState::Press => steami.leds.user_green.set_high(),
                    };
                }
            }
            None => (),
        };
    }
}

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

#[interrupt]
fn USART1() {
    with(|cs| {
        let mut jacdac = JACDAC_IFACE.borrow(cs).borrow_mut();
        let jacdac = jacdac.as_mut().unwrap();

        if jacdac.usart.check_status_flag(UsartInterrupt::Idle) {
            jacdac.usart.clear_interrupt(UsartInterrupt::Idle);

            unsafe {
                // defmt::println!("Read: {:?}", JACDAC_BUF.as_slices().0);
                match Frame::from_buffer(JACDAC_BUF.as_slices().0) {
                    Ok(frame) => JACDAC_FRAME.push_back(frame),
                    Err(_) => {
                        defmt::println!("Failed to parse buffer ({:?})", JACDAC_BUF.as_slices().0)
                    }
                };
                JACDAC_BUF.clear();
            }
        }

        if jacdac.usart.check_status_flag(UsartInterrupt::ReadNotEmpty) {
            jacdac.usart.clear_interrupt(UsartInterrupt::ReadNotEmpty);

            unsafe {
                JACDAC_BUF.push_back(jacdac.usart.read_one());
            }
        }
    });
}
