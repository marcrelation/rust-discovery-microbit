#![no_main]
#![no_std]

use core::fmt::Write;
use cortex_m_rt::entry;
use heapless::Vec;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

#[cfg(feature = "v1")]
use microbit::{
    hal::prelude::*,
    hal::uart,
    hal::uart::{Baudrate, Parity},
};

#[cfg(feature = "v2")]
use microbit::{
    hal::prelude::*,
    hal::uarte,
    hal::uarte::{Baudrate, Parity},
};

#[cfg(feature = "v2")]
mod serial_setup;
#[cfg(feature = "v2")]
use serial_setup::UartePort;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

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
    let mut serial = {
        let serial = uarte::Uarte::new(
            board.UARTE0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );
        UartePort::new(serial)
    };

    //let string_to_send = "The quick brown fox jumps over the lazy dog.\r\n";

    //nb::block!(serial.write(b'X')).unwrap();
    /*
    for character in string_to_send.as_bytes().iter() {
        nb::block!(serial.write(*character)).unwrap();
    }
    */
    //write!(serial, "{}", string_to_send).unwrap();
    //nb::block!(serial.flush()).unwrap();

    /*
    loop {
        let byte = nb::block!(serial.read()).unwrap();
        //rprintln!("{}", byte as char);
        nb::block!(serial.write('[' as u8)).unwrap();
        nb::block!(serial.write(byte)).unwrap();
        nb::block!(serial.write(']' as u8)).unwrap();
        nb::block!(serial.flush()).unwrap();
    }
    */
    // A buffer with 32 bytes of capacity
    let mut buffer: Vec<u8, 32> = Vec::new();

    loop {
        buffer.clear();

        loop {
            let byte = nb::block!(serial.read()).unwrap();
            // Receive a user request. Each user request ends with ENTER
            // NOTE `buffer.push` returns a `Result`. Handle the error by responding
            // with an error message.
            if buffer.push(byte).is_err() {
                write!(serial, "error: buffer full\r\n").unwrap();
                break;
            }
            // Send back the reversed string
            if byte as char == '\n' || byte as char == '\r' {
                for byte in buffer.iter().rev().chain(&[b'\n', b'\r']) {
                    nb::block!(serial.write(*byte)).unwrap();
                }
                break;
            }
        }
        nb::block!(serial.flush()).unwrap();
    }
}
