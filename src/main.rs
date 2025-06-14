#![no_std]
#![no_main]

extern crate panic_halt;

use avr_device::interrupt;
use core::cell::RefCell;

type Console = arduino_hal::hal::usart::Usart0<arduino_hal::DefaultClock>;
static CONSOLE: interrupt::Mutex<RefCell<Option<Console>>> =
    interrupt::Mutex::new(RefCell::new(None));

fn put_console(console: Console) {
    interrupt::free(|cs| {
        *CONSOLE.borrow(cs).borrow_mut() = Some(console);
    })
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    put_console(serial);

    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    let my_pin = pins.a0.into_analog_input(&mut adc);

    loop {
        let dat = my_pin.analog_read(&mut adc).to_le_bytes();
        interrupt::free(|cs| {
            if let Some(console) = CONSOLE.borrow(cs).borrow_mut().as_mut() {
                for d in dat {
                    let _ = console.write_byte(d);
                }
                console.flush();
            }
        });

        arduino_hal::delay_ms(100);
    }
}
