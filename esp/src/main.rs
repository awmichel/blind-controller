#![no_std]
#![no_main]

use core::cell::RefCell;

use critical_section::Mutex;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    gpio::{GpioPin, Io},
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
};
use motor::{Motor, MotorEncoder};

pub mod motor;

static ENCODER1: Mutex<RefCell<Option<MotorEncoder<GpioPin<39>, GpioPin<36>>>>> =
    Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);

    let clocks = ClockControl::max(system.clock_control).freeze();
    let delay = Delay::new(&clocks);

    esp_println::logger::init_logger_from_env();

    let mut io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    io.set_interrupt_handler(interrupt_handler);

    let mut blind1 = Motor::take(
        io.pins.gpio33,
        io.pins.gpio32,
        io.pins.gpio25,
        peripherals.MCPWM0,
        &clocks,
    );
    let encoder1 = blind1.with_encoder(io.pins.gpio39, io.pins.gpio36);

    critical_section::with(|cs| {
        ENCODER1.borrow_ref_mut(cs).replace(encoder1);
    });

    loop {
        log::info!("CW 0.75");
        blind1.drive(0.75);
        delay.delay(2.secs());

        // log::info!("CCW 0.50");
        // blind1.drive(-0.5);
        // delay.delay(2.secs());

        log::info!("Stop");
        blind1.stop();
        critical_section::with(|cs| {
            let mut binding = ENCODER1.borrow_ref_mut(cs);
            let encoder1 = binding.as_mut().unwrap();
            log::info!("Motor ran for {} counts", encoder1.counter);
        });
        delay.delay(5.secs());
    }
}

#[handler]
#[ram]
fn interrupt_handler() {
    critical_section::with(|cs| {
        let mut binding = ENCODER1.borrow_ref_mut(cs);
        let encoder1 = binding.as_mut().unwrap();
        encoder1.update()
    });
}
