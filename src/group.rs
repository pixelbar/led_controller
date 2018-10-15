use core::num::NonZeroU8;
use crate::serial::Color;
use embedded_hal::digital::OutputPin;

pub struct GroupColor<TPin> {
    pin: TPin,
    off_index: Option<NonZeroU8>,
}

impl<TPin> GroupColor<TPin> {
    pub fn new(pin: TPin) -> Self {
        GroupColor {
            pin,
            off_index: None,
        }
    }
}

impl<TPin: OutputPin> GroupColor<TPin> {
    fn update(&mut self, frame: u8) {
        if frame == 0 {
            if self.off_index.is_some() {
                self.pin.set_high();
            } else {
                self.pin.set_low();
            }
        } else if let Some(off_index) = self.off_index {
            if frame == off_index.get() {
                self.pin.set_low();
            }
        }
    }
}

pub struct Group<TRedPin, TGreenPin, TBluePin, TWhitePin> {
    red: GroupColor<TRedPin>,
    green: GroupColor<TGreenPin>,
    blue: GroupColor<TBluePin>,
    white: GroupColor<TWhitePin>,
}

impl<TRedPin, TGreenPin, TBluePin, TWhitePin> Group<TRedPin, TGreenPin, TBluePin, TWhitePin> {
    pub fn new(red: TRedPin, green: TGreenPin, blue: TBluePin, white: TWhitePin) -> Self {
        Group {
            red: GroupColor::new(red),
            blue: GroupColor::new(blue),
            green: GroupColor::new(green),
            white: GroupColor::new(white),
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.red.off_index = NonZeroU8::new(color.red_brightness);
        self.green.off_index = NonZeroU8::new(color.green_brightness);
        self.blue.off_index = NonZeroU8::new(color.blue_brightness);
        self.white.off_index = NonZeroU8::new(color.white_brightness);
    }
}

impl<TRedPin: OutputPin, TGreenPin: OutputPin, TBluePin: OutputPin, TWhitePin: OutputPin>
    Group<TRedPin, TGreenPin, TBluePin, TWhitePin>
{
    pub fn update(&mut self, frame: u8) {
        self.red.update(frame);
        self.blue.update(frame);
        self.green.update(frame);
        self.white.update(frame);
    }
}
