#![no_std]

use hal::{
    clocks::Clocks,
    dma::{self, Dma},
    gpio::{Pin, PinMode, Port},
    i2c::I2c,
    pac::{self, DMA1, SPI1, SYST, USART1},
    spi::{BaudRate, Spi, SpiConfig},
    usart::{Usart, UsartConfig},
};

pub struct Buttons {
    pub a: Pin,
    pub b: Pin,
    pub menu: Pin,
}

pub struct Leds {
    pub user_red: Pin,
    pub user_green: Pin,
    pub user_blue: Pin,
    // pub ble: Pin,
}

pub struct InterruptPins {
    int_magnetometer: Pin,
    int_accelerometer: Pin,
    int_distance: Pin,
    int_gpio_expander: Pin,
}

pub struct AnalogPins {
    p0: Pin,
    p1: Pin,
    p2: Pin,
    p3: Pin,
    p4: Pin,
    p10: Pin,
}

pub struct Pins {
    p6: Pin,
    p7: Pin,
    p8: Pin,
    p9: Pin,
    p12: Pin,
    // p16: Pin,
    p19: Pin,
    p20: Pin,
}

pub struct Screen {
    spi: Spi<SPI1>,
    mosi: Pin,
    clk: Pin,
    reset: Pin,
    cs: Pin,
    d_c: Pin,
}

pub struct Jacdac {
    pub rx: Pin,
    pub tx: Pin,
    pub usart: Usart<USART1>,
}

pub struct DevicePeripherals {
    pub dma1: DMA1,
}

pub struct STeaMi {
    /// LED pins
    pub leds: Leds,

    /// Button pins
    pub buttons: Buttons,

    /// Peripheral interrupt pins
    pub interrupt_pins: InterruptPins,

    /// Analog pins
    pub analogs: AnalogPins,

    /// Remaining pin for no specific use
    pub pins: Pins,

    /// Screen pins and spi
    pub screen: Screen,

    pub jacdac: Jacdac,

    /// Internal I2C used to communicate with sensors and peripherals on the board
    pub internal_i2c: I2c<pac::I2C1>,

    /// External I2C available on the micro:bit connector and QWIC socket
    pub external_i2c: I2c<pac::I2C3>,

    // pub external_spi: Spi<pac::SPI2>,

    // pub cortex_peripheral: cortex_m::Peripherals,
    pub clocks: Clocks,

    pub cortex_peripherals: cortex_m::Peripherals,

    pub device_peripherals: DevicePeripherals,
}

impl STeaMi {
    /// Take the STeaMi board safely
    ///
    /// The methode only return a Self object the first time and only if the cortex peripheral is available
    pub fn take() -> Option<Self> {
        // let cp = cortex_m::Peripherals::take()?;
        let dp = pac::Peripherals::take()?;

        let clocks = Clocks::default();
        clocks.setup().ok()?;

        Some(Self {
            leds: Leds {
                user_red: Pin::new(Port::C, 12, PinMode::Output),
                user_green: Pin::new(Port::C, 11, PinMode::Output),
                user_blue: Pin::new(Port::C, 10, PinMode::Output),
                // ble: Pin::new(Port::H, 3, PinMode::Output),
            },

            buttons: Buttons {
                a: Pin::new(Port::A, 7, PinMode::Input),
                b: Pin::new(Port::A, 8, PinMode::Input),
                menu: Pin::new(Port::A, 0, PinMode::Input),
            },

            screen: Screen {
                mosi: Pin::new(Port::B, 5, PinMode::Alt(5)),
                clk: Pin::new(Port::A, 1, PinMode::Alt(5)),
                spi: Spi::new(dp.SPI1, SpiConfig::default(), BaudRate::Div2),
                reset: Pin::new(Port::A, 12, PinMode::Output),
                cs: Pin::new(Port::D, 0, PinMode::Output),
                d_c: Pin::new(Port::B, 4, PinMode::Output),
            },

            interrupt_pins: InterruptPins {
                int_magnetometer: Pin::new(Port::D, 1, PinMode::Input),
                int_accelerometer: Pin::new(Port::C, 13, PinMode::Input),
                int_distance: Pin::new(Port::B, 12, PinMode::Input),
                int_gpio_expander: Pin::new(Port::B, 0, PinMode::Input),
            },

            analogs: AnalogPins {
                p0: Pin::new(Port::C, 4, PinMode::Analog),
                p1: Pin::new(Port::A, 5, PinMode::Analog),
                p2: Pin::new(Port::C, 5, PinMode::Analog),
                p3: Pin::new(Port::A, 2, PinMode::Analog),
                p4: Pin::new(Port::A, 4, PinMode::Analog),
                p10: Pin::new(Port::A, 6, PinMode::Analog),
            },

            pins: Pins {
                p6: Pin::new(Port::C, 3, PinMode::Output),
                p7: Pin::new(Port::A, 9, PinMode::Output),
                p8: Pin::new(Port::A, 15, PinMode::Output),
                p9: Pin::new(Port::C, 2, PinMode::Output),
                p12: Pin::new(Port::C, 6, PinMode::Output),
                // p16: Pin::new(Port::E, 4,PinMode::Output),
                p19: Pin::new(Port::C, 0, PinMode::Output),
                p20: Pin::new(Port::C, 1, PinMode::Output),
            },

            jacdac: Self::configure_jacdac(dp.USART1, &clocks),

            internal_i2c: I2c::new(dp.I2C1, Default::default(), &clocks),
            external_i2c: I2c::new(dp.I2C3, Default::default(), &clocks),
            // external_spi: Spi::new(dp.SPI2, SpiConfig::default(), BaudRate::Div2),
            clocks,
            cortex_peripherals: cortex_m::Peripherals::take()?,
            device_peripherals: DevicePeripherals { dma1: dp.DMA1 },
        })
    }

    fn configure_jacdac(usart_reg: USART1, clocks: &Clocks) -> Jacdac {
        usart_reg
            .cr2
            .modify(|_, w| w.linen().clear_bit().clken().clear_bit());
        usart_reg
            .cr3
            .modify(|_, w| w.hdsel().set_bit().scen().clear_bit().iren().clear_bit());

        let jacdac_rx = Pin::new(Port::B, 7, PinMode::Alt(7));
        let mut jacdac_tx = Pin::new(Port::B, 6, PinMode::Alt(7));
        jacdac_tx.output_type(hal::gpio::OutputType::OpenDrain);

        let jacdac_usart = Usart::new(
            usart_reg,
            1_000_000,
            UsartConfig {
                word_len: hal::usart::WordLen::W8,
                stop_bits: hal::usart::StopBits::S1,
                parity: hal::usart::Parity::Disabled,
                oversampling: hal::usart::OverSampling::O8,
                ..Default::default()
            },
            clocks,
        );

        jacdac_usart.regs.cr1.modify(|_, w| w.fifoen().clear_bit());

        Jacdac {
            rx: jacdac_rx,
            tx: jacdac_tx,
            usart: jacdac_usart,
        }
    }
}
