use std::io::{BufRead, Read, Write};
use clap::Parser;
use std::time::Duration;
use serialport::{DataBits, Parity, StopBits};
use retry::{retry};
use retry::delay::Fixed;
use crate::OpCodes::{ERRO, SBEGIN};

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
enum OpCodes {
 SBEGIN = 0x01,
 SDATA  = 0x02,
 SRSP  =  0x03,
 SEND =   0x04,
 ERRO  =  0x05 ,
}

fn main() {
    let args = Args::parse();

    let mut port = serialport::new(args.port, 115_200)
        .timeout(Duration::from_millis(100))
        .open()
        .expect("Failed to open port");

    port.set_data_bits(DataBits::Eight)
        .expect("Failed to set databits");

    port.set_parity(Parity::None)
        .expect("Failed to set parity");

    port.set_stop_bits(StopBits::One)
        .expect("Failed to set stopbits");

    let mut serial_buf: Vec<u8> = vec![0; 1000];

    serial_buf[0] = SBEGIN as u8;
    serial_buf[1] = 0;
    let bytes_written = port.write(&serial_buf[0..2])
        .expect("Write failed");
    println!("Bytes written {bytes_written}");

    let result = retry(Fixed::from_millis(1000).take(15), || {
        let result = port.read(serial_buf.as_mut_slice());
        return match result {
            Ok(n) => {
                if n == 0 {
                    return Err("Serial connection closed".to_owned());
                }
                if serial_buf[0] == ERRO as u8 {
                    return Err("Programmer returned error".to_owned());
                }
                result.unwrap()
            },
            Err(err) => Err(err)
        }
    });

    if result.is_ok() {
        let bytes_read = result.unwrap();
        println!("Bytes read: {bytes_read}")
    } else {
        println!("Timeout reading from serial")
    }

}
