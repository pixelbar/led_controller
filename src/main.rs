//! Prints "Hello, world!" on the OpenOCD console using semihosting
//!
//! ---

#![feature(used, const_size_of)]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate cortex_m_semihosting;
extern crate peripherals;

use cortex_m::asm;

struct Frame {
    pub side_a: u32,
    pub side_b: u32,
}

fn configure(rcc: &peripherals::RCC, gpioa: &peripherals::GPIOA, gpiob: &peripherals::GPIOB){
    // TODO: Enable the USB port
    // Enable the 2 IO strips (A and B)
    rcc.apb2enr.modify(|_, w| 
        w.iopaen().enabled()
         .iopben().enabled()
    );

    // Configure the A pins to be output + push
    gpioa.crl.modify(|_, w| 
        w.mode0().output().cnf0().push()
         .mode1().output().cnf1().push()
         .mode2().output().cnf2().push()
         .mode3().output().cnf3().push()
         .mode4().output().cnf4().push()
         .mode5().output().cnf5().push()
         .mode6().output().cnf6().push()
         .mode7().output().cnf7().push()
    );
        
    // Configure the B pins to be output + push
    gpiob.crh.modify(|_, w| 
        w.mode8().output().cnf8().push()
         .mode9().output().cnf9().push()
         .mode10().output().cnf10().push()
         .mode11().output().cnf11().push()
         .mode12().output().cnf12().push()
         .mode13().output().cnf13().push()
         .mode14().output().cnf14().push()
         .mode15().output().cnf15().push()
    );
}


fn make_go_faster(rcc: &peripherals::RCC, flash: &peripherals::FLASH) {
    rcc.cr.modify(|_, w| w.hseon().enabled());
    while !rcc.cr.read().hserdy().is_ready() {}
    flash.acr.modify(|_, w| w.prftbe().enabled());
    flash.acr.modify(|_, w| w.latency().two());
    rcc.cfgr.modify(|_, w| w
                    .hpre().div1()
                    .ppre2().div1()
                    .ppre1().div2()
                    // .adcpre().bits(8)
                    .pllsrc().external()
                    .pllxtpre().div1()
                    .pllmul().mul9()
    );
    rcc.cr.modify(|_, w| w.pllon().enabled());
    while rcc.cr.read().pllrdy().is_unlocked() {}
    rcc.cfgr.modify(|_,w| w.sw().pll());
    while !rcc.cfgr.read().sws().is_pll() {}
}

enum Block {
    Block1,
    Block2,
    Block3,
    Block4
}

enum Color {
    Red,
    Green,
    Blue,
    White
}

trait BlockColorTrait {
    fn apply(&self, frame: &mut Frame);
}

impl BlockColorTrait for (Block, Color) {
    fn apply(&self, frame: &mut Frame) {
        let color_mask = match self.1 {
            Color::Red => 0b0001,
            Color::Green => 0b0010,
            Color::Blue => 0b0100,
            Color::White => 0b1000,
        };
        match self.0 {
            Block::Block1 => {
                frame.side_a |= color_mask;
            },
            Block::Block2 => {
                frame.side_a |= color_mask << 4;
            }
            Block::Block3 => {
                frame.side_b |= color_mask << 8;
            },
            Block::Block4 => {
                frame.side_b |= color_mask << 12;
            }
        }
    }
}

fn configure_usart(usart: &peripherals::USART1, rcc: &peripherals::RCC, afio: &peripherals::AFIO, gpiob: &peripherals::GPIOB) {
    rcc.apb2enr.modify(|_, w|
        w.usart1en().enabled()
         .afioen().enabled()
    );
    afio.mapr.modify(|_, w|
        w.usart1_remap().set_bit()
    );
    gpiob.crl.modify(|_, w|
        w.mode6().output().cnf6().alt_push() // alt_open?
         .mode7().input().cnf7().bits(0b01)
    );
    usart.cr2.write(|w| unsafe {
        w.stop().bits(0b00)
    });
    usart.cr3.write(|w| 
        w.rtse().clear_bit()
         .ctse().clear_bit()
         .dmat().set_bit()
         .dmar().set_bit()
    );
    const BAUD_RATE: u32 = 115200;
    const BRR: u16 = (8_000_000 / BAUD_RATE) as u16;
    usart.brr.write(|w| unsafe {
        w.div_fraction().bits((BRR & 0b1111) as u8)
         .div_mantissa().bits(BRR >> 4)
    });
    usart.cr1.write(|w|
        w.ue().set_bit()
         .re().set_bit()
         .te().set_bit()
         .pce().clear_bit()
         .rxneie().clear_bit()
         // .over8().clear_bit()
    );
}

// const BUFFER_SIZE: usize = 100 * core::mem::size_of::<Frame>();

fn main() {
    let rcc: &peripherals::RCC = unsafe { &*peripherals::RCC.get() };
    let flash: &peripherals::FLASH = unsafe { &*peripherals::FLASH.get() };
    let gpioa: &peripherals::GPIOA = unsafe { &*peripherals::GPIOA.get() };
    let gpiob: &peripherals::GPIOB = unsafe { &*peripherals::GPIOB.get() };
    let usart: &peripherals::USART1 = unsafe { &*peripherals::USART1.get() };
    let afio: &peripherals::AFIO = unsafe { &*peripherals::AFIO.get() };

    let mut frames: [Frame;100] = unsafe { ::core::mem::zeroed() };
    // let mut buffer: [u8; BUFFER_SIZE] = unsafe { ::core::mem::uninitialized() };
    // let mut buffer_index: usize = 0;

    (Block::Block1, Color::Red).apply(&mut frames[0]);
    (Block::Block2, Color::Green).apply(&mut frames[0]);
    (Block::Block3, Color::Blue).apply(&mut frames[0]);
    (Block::Block4, Color::White).apply(&mut frames[0]);

    configure(&rcc, &gpioa, &gpiob);
    configure_usart(&usart, &rcc, &afio, &gpiob);
    make_go_faster(&rcc, &flash);
    usart.dr.write(|w| unsafe {
        w.dr().bits(u16::from('X' as u8))
    });

    loop {
        for frame in frames.iter() {
            gpioa.bsrr.write(|w| unsafe {
                w.bits(frame.side_a)
            });
            gpiob.bsrr.write(|w| unsafe {
                w.bits(frame.side_b)
            });

            // TODO: If there is a byte to read from USB,
            // read it, append it to `buffer`, and increase `buffer_index`
            // If `buffer_index` is larger than `BUFFER_SIZE`
            // memcopy `buffer` to `frames`, reset `buffer_index`
            // and send a signal to the host machine that we're done
        }
    }
}

// As we are not using interrupts, we just register a dummy catch all handler
#[link_section = ".vector_table.interrupts"]
#[used]
static INTERRUPTS: [extern "C" fn(); 240] = [default_handler; 240];

extern "C" fn default_handler() {
    asm::bkpt();
}
