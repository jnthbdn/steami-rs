use hal::gpio::{Pin, PinState};

/// Represent the User led on STeaMi board (3 colors leds)
pub struct UserLed {
    pub(crate) red: Pin,
    pub(crate) green: Pin,
    pub(crate) blue: Pin,
}

/// Color available with the User led
pub enum ColorLed {
    Red,
    Green,
    Blue,
    Yellow,
    Magenta,
    Cyan,

    White,
    Black,
}

impl UserLed {
    /// Set the `color` of the Use led
    pub fn set_color(&mut self, color: ColorLed) {
        match color {
            ColorLed::Red => self.set_led_state(true, false, false),
            ColorLed::Green => self.set_led_state(false, true, false),
            ColorLed::Blue => self.set_led_state(false, false, true),
            ColorLed::Yellow => self.set_led_state(true, true, false),
            ColorLed::Magenta => self.set_led_state(true, false, true),
            ColorLed::Cyan => self.set_led_state(false, true, true),
            ColorLed::White => self.set_led_state(true, true, true),
            ColorLed::Black => self.set_led_state(false, false, false),
        };
    }

    /// Manually set esch led of the User led
    pub fn set_led_state(&mut self, is_red_on: bool, is_green_on: bool, is_blue_on: bool) {
        self.red.set_state(if is_red_on {
            PinState::High
        } else {
            PinState::Low
        });

        self.green.set_state(if is_green_on {
            PinState::High
        } else {
            PinState::Low
        });

        self.blue.set_state(if is_blue_on {
            PinState::High
        } else {
            PinState::Low
        });
    }

    /// Get the internal pin
    pub fn get_pins(self) -> (Pin, Pin, Pin) {
        (self.red, self.green, self.blue)
    }
}
