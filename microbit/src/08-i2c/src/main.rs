#![no_main]
#![no_std]

use core::fmt::Write;
use cortex_m_rt::entry;
use heapless::Vec;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use microbit::hal::prelude::*;

#[cfg(feature = "v1")]
use microbit::{
    hal::prelude::*,
    hal::uart,
    hal::uart::{Baudrate, Parity},
};
#[cfg(feature = "v1")]
use microbit::{hal::twi, pac::twi0::frequency::FREQUENCY_A};

#[cfg(feature = "v2")]
use microbit::{
    hal::prelude::*,
    hal::uarte,
    hal::uarte::{Baudrate, Parity},
};
#[cfg(feature = "v2")]
use microbit::{hal::twim, pac::twim0::frequency::FREQUENCY_A};
#[cfg(feature = "v2")]
mod serial_setup;
#[cfg(feature = "v2")]
use serial_setup::UartePort;

use lsm303agr::{AccelOutputDataRate, Lsm303agr, MagOutputDataRate};

/*
const ACCELEROMETER_ADDR: u8 = 0b0011001;
const MAGNETOMETER_ADDR: u8 = 0b0011110;

const ACCELEROMETER_ID_REG: u8 = 0x0f;
const MAGNETOMETER_ID_REG: u8 = 0x4f;
*/

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    #[cfg(feature = "v1")]
    let i2c = { twi::Twi::new(board.TWI0, board.i2c.into(), FREQUENCY_A::K100) };

    #[cfg(feature = "v1")]
    let mut serial = {
        uart::Uart::new(
            board.UART0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        )
    };

    #[cfg(feature = "v2")]
    let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };

    #[cfg(feature = "v2")]
    let mut serial = {
        let serial = uarte::Uarte::new(
            board.UARTE0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );
        UartePort::new(serial)
    };

    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz50).unwrap();
    sensor.set_mag_odr(MagOutputDataRate::Hz50).unwrap();
    let mut sensor = sensor.into_mag_continuous().ok().unwrap();

    let mut buffer: Vec<u8, 32> = Vec::new();
    loop {
        buffer.clear();

        loop {
            let byte = nb::block!(serial.read()).unwrap();

            nb::block!(serial.write(byte)).unwrap();
            nb::block!(serial.flush()).unwrap();

            if buffer.push(byte).is_err() {
                write!(serial, "error: buffer full\r\n").unwrap();
                break;
            }
            // Send back the reversed string
            if byte as char == '\n' || byte as char == '\r' {
                let input_string = core::str::from_utf8(&buffer).unwrap();
                if input_string == "magnetometer\r" || input_string == "accelerometer\r" {
                    loop {
                        if input_string == "accelerometer\r" {
                            if sensor.accel_status().unwrap().xyz_new_data {
                                let data = sensor.accel_data().unwrap();
                                write!(
                                    serial,
                                    "Acceleration: x {} y {} z {}\r\n",
                                    data.x, data.y, data.z
                                )
                                .unwrap();
                                nb::block!(serial.flush()).unwrap();
                                break;
                            }
                        } else {
                            if sensor.mag_status().unwrap().xyz_new_data {
                                let data = sensor.mag_data().unwrap();
                                write!(
                                    serial,
                                    "Magnetometer: {:>4} {:>4} {:>4}\r\n",
                                    data.x, data.y, data.z
                                )
                                .unwrap();
                                nb::block!(serial.flush()).unwrap();
                                break;
                            }
                        }
                    }
                }
                break;
            }
        }
    }
}

/*
    let mut acc = [0];
    let mut mag = [0];

    // First write the address + register onto the bus, then read the chip's responses
    i2c.write_read(ACCELEROMETER_ADDR, &[ACCELEROMETER_ID_REG], &mut acc).unwrap();
    i2c.write_read(MAGNETOMETER_ADDR, &[MAGNETOMETER_ID_REG], &mut mag).unwrap();

    rprintln!("The accelerometer chip's id is: {:#b}", acc[0]);
    rprintln!("The magnetometer chip's id is: {:#b}", mag[0]);

    loop {}
*/
