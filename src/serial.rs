use arrayref::array_refs;
use nb::block;
use nb::Error;

use stm32f1xx_hal::{
    afio::MAPR,
    pac::USART1,
    prelude::*,
    rcc::Clocks,
    serial::{Config, Pins, Rx, Serial, Tx},
};

pub struct Color {
    pub red_brightness: u8,
    pub green_brightness: u8,
    pub blue_brightness: u8,
    pub white_brightness: u8,
}

impl Color {
    pub fn from_buffer(buffer: &[u8; 4]) -> Color {
        Color {
            red_brightness: buffer[0],
            green_brightness: buffer[1],
            blue_brightness: buffer[2],
            white_brightness: buffer[3],
        }
    }
}

pub struct Message {
    pub kitchen: Color,
    pub beamer: Color,
    pub stairs: Color,
    pub door: Color,
}

pub struct SerialConnector {
    tx: Tx<USART1>,
    rx: Rx<USART1>,
    buffer: [u8; 17],
    current_buffer_index: usize,
}

impl SerialConnector {
    pub fn new<TTxPin, TRxPin>(
        usart: USART1,
        tx: TTxPin,
        rx: TRxPin,
        mapr: &mut MAPR,
        clocks: Clocks,
    ) -> Self
    where
        (TTxPin, TRxPin): Pins<USART1>,
    {
        let serial = Serial::usart1(
            usart,
            (tx, rx),
            mapr,
            Config::default().baudrate(9600.bps()),
            clocks,
        );
        let (tx, rx) = serial.split();
        SerialConnector {
            tx,
            rx,
            buffer: [0u8; 17],
            current_buffer_index: 0,
        }
    }

    pub fn read(&mut self) -> Option<Message> {
        let val = match self.rx.read() {
            Ok(v) => v,
            Err(Error::WouldBlock) => return None,
            Err(Error::Other(_e)) => {
                // TODO: Log error
                return None;
            }
        };
        if self.current_buffer_index == 0 {
            if val != 255 {
                return None;
            }
        }
        self.buffer[self.current_buffer_index] = val;
        self.current_buffer_index += 1;
        if self.current_buffer_index == self.buffer.len() {
            let (_, kitchen, beamer, stairs, door) = array_refs!(&self.buffer, 1, 4, 4, 4, 4);
            let message = Message {
                kitchen: Color::from_buffer(kitchen),
                beamer: Color::from_buffer(beamer),
                stairs: Color::from_buffer(stairs),
                door: Color::from_buffer(door),
            };
            self.current_buffer_index = 0;
            let _ = block!(self.tx.write(255));
            Some(message)
        } else {
            None
        }
    }
}
