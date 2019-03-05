#![no_std]
#![no_main]

// pick a panicking behavior
// dev profile: easier to debug panics; can put a breakpoint on `rust_begin_unwind`
#[cfg(debug_assertions)]
extern crate panic_semihosting;

// release profile: minimize the binary size of the application
#[cfg(not(debug_assertions))]
extern crate panic_abort;

extern crate embedded_hal;
extern crate stm32f103xx_hal;
// extern crate panic_abort; // requires nightly
// extern crate panic_itm; // logs messages over ITM; requires ITM support
// extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

// use core::fmt::Write;
use cortex_m_rt::entry;
use cortex_m_semihosting::hio;

use stm32f103xx_hal::delay::Delay;
use stm32f103xx_hal::prelude::*;
use stm32f103xx_hal::stm32f103xx;

mod group;
mod serial;

use self::group::Group;
use self::serial::SerialConnector;

#[entry]
fn main() -> ! {
    let dp = stm32f103xx::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    let _delay = Delay::new(cp.SYST, clocks);
    let _stdout = hio::hstdout().unwrap();

    let mut kitchen = Group::new(
        gpioa.pa0.into_push_pull_output(&mut gpioa.crl),
        gpioa.pa1.into_push_pull_output(&mut gpioa.crl),
        gpioa.pa2.into_push_pull_output(&mut gpioa.crl),
        gpioa.pa3.into_push_pull_output(&mut gpioa.crl),
    );
    let mut beamer = Group::new(
        gpioa.pa4.into_push_pull_output(&mut gpioa.crl),
        gpioa.pa5.into_push_pull_output(&mut gpioa.crl),
        gpioa.pa6.into_push_pull_output(&mut gpioa.crl),
        gpioa.pa7.into_push_pull_output(&mut gpioa.crl),
    );
    let mut stairs = Group::new(
        gpiob.pb8.into_push_pull_output(&mut gpiob.crh),
        gpiob.pb9.into_push_pull_output(&mut gpiob.crh),
        gpiob.pb10.into_push_pull_output(&mut gpiob.crh),
        gpiob.pb11.into_push_pull_output(&mut gpiob.crh),
    );
    let mut door = Group::new(
        gpiob.pb12.into_push_pull_output(&mut gpiob.crh),
        gpiob.pb13.into_push_pull_output(&mut gpiob.crh),
        gpiob.pb14.into_push_pull_output(&mut gpiob.crh),
        gpiob.pb15.into_push_pull_output(&mut gpiob.crh),
    );

    let mut serial = SerialConnector::new(
        dp.USART1,
        gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl),
        gpiob.pb7,
        &mut afio.mapr,
        clocks,
        &mut rcc.apb2,
    );

    let mut frame = 0;
    loop {
        kitchen.update(frame);
        beamer.update(frame);
        stairs.update(frame);
        door.update(frame);
        frame += 1;
        if frame == 100 {
            frame = 0;
        }

        if let Some(message) = serial.read() {
            kitchen.set_color(message.kitchen);
            beamer.set_color(message.beamer);
            stairs.set_color(message.stairs);
            door.set_color(message.door);
        }
    }
}
