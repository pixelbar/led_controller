use nb::Error;
use stm32f103xx_hal::afio::MAPR;
use stm32f103xx_hal::prelude::*;
use stm32f103xx_hal::rcc::{Clocks, APB2};
use stm32f103xx_hal::serial::{Pins, Rx, Serial, Tx};
use stm32f103xx_hal::stm32f103xx::USART1;

pub struct Color {
    pub red_brightness: u8,
    pub green_brightness: u8,
    pub blue_brightness: u8,
    pub white_brightness: u8,
}

impl Color {
    pub fn from_buffer(buffer: &[u8]) -> Color {
        debug_assert_eq!(4, buffer.len());
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

impl Message {
    pub fn from_buffer(buffer: &[u8]) -> Message {
        debug_assert_eq!(16, buffer.len());
        Message {
            kitchen: Color::from_buffer(&buffer[0..4]),
            beamer: Color::from_buffer(&buffer[4..8]),
            stairs: Color::from_buffer(&buffer[8..12]),
            door: Color::from_buffer(&buffer[12..15]),
        }
    }
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
        apb2: &mut APB2,
    ) -> Self
    where
        (TTxPin, TRxPin): Pins<USART1>,
    {
        let serial = Serial::usart1(usart, (tx, rx), mapr, 9_600.bps(), clocks, apb2);
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
            let message = Message::from_buffer(&self.buffer[1..]);
            self.current_buffer_index = 0;
            block!(self.tx.write(255));
            Some(message)
        } else {
            None
        }
    }
}
