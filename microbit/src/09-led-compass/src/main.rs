#![deny(unsafe_code)]
#![no_main]
#![no_std]

use core::f32::consts::PI;

use cortex_m_rt::entry;
use libm::atan2f;
use libm::sqrtf;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

mod calibration;
use crate::calibration::calc_calibration;
use crate::calibration::calibrated_measurement;
use crate::calibration::Calibration;
use crate::led::direction_to_led;

mod led;
use led::Direction;

use microbit::{display::blocking::Display, hal::Timer};

#[cfg(feature = "v1")]
use microbit::{hal::twi, pac::twi0::frequency::FREQUENCY_A};

#[cfg(feature = "v2")]
use microbit::{hal::twim, pac::twim0::frequency::FREQUENCY_A};

use lsm303agr::{AccelOutputDataRate, Lsm303agr, MagOutputDataRate, Measurement};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    #[cfg(feature = "v1")]
    let i2c = { twi::Twi::new(board.TWI0, board.i2c.into(), FREQUENCY_A::K100) };

    #[cfg(feature = "v2")]
    let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };

    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);

    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor.set_mag_odr(MagOutputDataRate::Hz10).unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz10).unwrap();
    let mut sensor = sensor.into_mag_continuous().ok().unwrap();

    let calibration = calc_calibration(&mut sensor, &mut display, &mut timer);
    /*
    let calibration: Calibration = Calibration::new(
        Measurement {
            x: -3680,
            y: 9722,
            z: 51620,
        },
        Measurement {
            x: 2462,
            y: 2920,
            z: 2551,
        },
        60568,
    );
    */
    rprintln!("Calibration: {:?}", calibration);
    rprintln!("Calibration done, entering busy loop");

    loop {
        while !sensor.mag_status().unwrap().xyz_new_data {}
        let mut data = sensor.mag_data().unwrap();
        data = calibrated_measurement(data, &calibration);
        /*
        rprintln!("x: {}, y: {}, z: {}", data.x, data.y, data.z);
        let dir = match (data.x > 0, data.y > 0) {
            // Quadrant ???
            (true, true) => Direction::NorthEast,
            // Quadrant ???
            (false, true) => Direction::NorthWest,
            // Quadrant ???
            (false, false) => Direction::SouthWest,
            // Quadrant ???
            (true, false) => Direction::SouthEast,
        };
        */

        // use libm's atan2f since this isn't in core yet
        let theta = atan2f(data.y as f32, data.x as f32);

        let x = data.x as f32;
        let y = data.y as f32;
        let z = data.z as f32;
        let magnitude = sqrtf(x * x + y * y + z * z);
        rprintln!("{} nT, {} mG", magnitude, magnitude / 100.0);

        // Figure out the direction based on theta
        let dir = if theta < -7. * PI / 8. {
            Direction::West
        } else if theta < -5. * PI / 8. {
            Direction::SouthWest
        } else if theta < -3. * PI / 8. {
            Direction::South
        } else if theta < -PI / 8. {
            Direction::SouthEast
        } else if theta < PI / 8. {
            Direction::East
        } else if theta < 3. * PI / 8. {
            Direction::NorthEast
        } else if theta < 5. * PI / 8. {
            Direction::North
        } else if theta < 7. * PI / 8. {
            Direction::NorthWest
        } else {
            Direction::West
        };

        display.show(&mut timer, direction_to_led(dir), 250);
    }
}
