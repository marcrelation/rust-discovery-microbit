#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use microbit::board::Board;
use microbit::display::blocking::Display;
use microbit::hal::Timer;
/*
use microbit::hal::prelude::*;
*/
//use panic_halt as _;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);
    let mut led_matrix = [
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ];
    let led_positions = [
        (0, 0),
        (0, 1),
        (0, 2),
        (0, 3),
        (0, 4),
        (1, 4),
        (2, 4),
        (3, 4),
        (4, 4),
        (4, 3),
        (4, 2),
        (4, 1),
        (4, 0),
        (3, 0),
        (2, 0),
        (1, 0),
    ];

    let mut last_led = (0, 0);
    loop {
        for pos in led_positions.iter() {
            led_matrix[last_led.0][last_led.1] = 0;
            led_matrix[pos.0][pos.1] = 1;
            // rprintln!("Lighting: [{}][{}]", pos.0, pos.1);
            display.show(&mut timer, led_matrix, 30);
            //display.clear();
            //timer.delay_ms(100_u32);
            last_led = *pos;
        }
    }
}
