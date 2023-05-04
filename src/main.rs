extern crate core;

use crate::OpCodes::{ERRO, SBEGIN, SDATA, SEND, SRSP, UNKN};
use clap::Parser;
use retry::delay::Fixed;
use retry::{OperationResult, retry};
use serialport::{DataBits, Parity, StopBits};
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Write};
use std::time::Duration;
use crc::{Crc, CRC_16_USB};

pub const CRC_16: Crc<u16> = Crc::<u16>::new(&CRC_16_USB);

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Serial port
    #[arg(short, long)]
    port: String,

    /// The firmware file to upload
    #[arg(short, long)]
    firmware: std::path::PathBuf,
}

#[repr(u8)]
#[derive(PartialEq, PartialOrd)]
enum OpCodes {
    SBEGIN = 0x01,
    SDATA = 0x02,
    SRSP = 0x03,
    SEND = 0x04,
    ERRO = 0x05,
    UNKN = 0xFF,
}

impl From<u8> for OpCodes {
    fn from(orig: u8) -> Self {
        match orig {
            0x01 => return SBEGIN,
            0x02 => return SDATA,
            0x03 => return SRSP,
            0x04 => return SEND,
            0x05 => return ERRO,
            _ => return UNKN,
        };
    }
}

fn main() {
    let args = Args::parse();

    let mut port = serialport::new(args.port, 115_200)
        .timeout(Duration::from_millis(100))
        .open()
        .expect("Failed to open port");

    port.set_data_bits(DataBits::Eight)
        .expect("Failed to set databits");

    port.set_parity(Parity::None).expect("Failed to set parity");

    port.set_stop_bits(StopBits::One)
        .expect("Failed to set stopbits");

    port.write_data_terminal_ready(true)
        .expect("DTR high failed");

    let mut file = match File::open(&args.firmware) {
        Err(why) => panic!("couldn't open {}: {}", args.firmware.to_str().unwrap(), why),
        Ok(file) => file,
    };

    let mut serial_buf: Vec<u8> = vec![0; 1000];

    println!("Sending SBEGIN");
    serial_buf[0] = SBEGIN as u8;
    serial_buf[1] = 0;
    let _bytes_written = port.write(&serial_buf[0..2]).expect("Write failed");

    println!("Waiting for SRSP");

    let result = retry(Fixed::from_millis(1000).take(15), || {
        let result = port.read(serial_buf.as_mut_slice());
        return match result {
            Ok(n) => {
                if n == 0 {
                    return OperationResult::Err("Serial connection closed".to_owned());
                }
                if serial_buf[0] == ERRO as u8 {
                    return OperationResult::Err("Programmer returned error".to_owned());
                }
                OperationResult::Ok(result.unwrap())
            }
            Err(err) => OperationResult::Retry(err.to_string()),
        };
    });

    if !result.is_ok() {
        println!("Timeout reading from serial");
        return;
    }

    // We should have the received byte in the serial buffer now
    let response: OpCodes = serial_buf[0].into();
    if response != SRSP {
        println!("Invalid reponse {:?}", response as u8);
    }

    println!("Received SRSP, starting upload");

    let mut data_buf: Vec<u8> = vec![0; 515];
    data_buf[0] = SDATA as u8;

    // increase the timeout on responses as the programmer needs to write the data
    let timeout = port.timeout();
    port.set_timeout(Duration::from_millis(2000))
        .expect("set timeout failed");

    let upload_result = loop {
        // Read the next 512 bytes from the firmware file
        let result = file.read(&mut data_buf[1..513]);
        match result {
            Ok(count) => {
                if count == 0 {
                    // Nothing more to read, we are done
                    break Ok(())
                }
            }
            Err(_err) => break Err(_err)
        }

        let crc = CRC_16.checksum(&data_buf[1..513]);
        data_buf[513] = ((crc >> 8) & 0xFF) as u8;
        data_buf[514] = (crc & 0xFF) as u8;

        match port.write(data_buf.as_slice()) {
            Ok(n) => {
                if n != 515 {
                    break Err(Error::new(ErrorKind::InvalidData, "not all data sent"))
                }
            }

            Err(e) => break Err(e)
        };

        match port.read(serial_buf.as_mut_slice()) {
            Ok(n) => {
                // EOF
                if n == 0 {
                    break Err(Error::new(ErrorKind::InvalidData, "connection closed"))
                }
                if serial_buf[0] == ERRO as u8 {
                    break Err(Error::new(ErrorKind::InvalidData, "programmer returned error"))
                }
                println!("Block sent OK")
            },
            Err(e) => println!("{}", e)
        }

    };

    if upload_result.is_err() {
        println!("Upload failed")
    }

    // restore timeout
    port.set_timeout(timeout).expect("set_timeout failed");

    println!("Sending SEND");
    serial_buf[0] = SEND as u8;
    let _bytes_written = port.write(&serial_buf[0..1]).expect("Write failed");

}