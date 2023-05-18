/*
 * Copyright 2023 Hugo Trippaers
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

mod protocol;

extern crate core;

use clap::Parser;
use serialport::{DataBits, Parity, SerialPort, StopBits};
use std::fs::File;
use std::io;
use std::io::{ErrorKind, Read, Write};
use std::time::Duration;
use crc::{Crc, CRC_16_XMODEM};
use crate::protocol::OpCodes::SRSP;

pub const CRC_16: Crc<u16> = Crc::<u16>::new(&CRC_16_XMODEM);

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Serial port
    #[arg(short, long)]
    port: String,

    /// The firmware file to upload
    #[arg(short, long)]
    firmware: std::path::PathBuf,

    /// Verify all uploads
    #[arg(long, action)]
    verify: bool
}


fn main() {
    let args = Args::parse();

    let portname = args.port;
    let filename = args.firmware.to_str().unwrap();

    let mut port = match open_serial_port(portname.to_owned()) {
        Err(why) => panic!("couldn't open port {}: {}", portname, why),
        Ok(file) => file,
    };
    port.set_timeout(Duration::from_millis(500))
        .expect("set timeout failed");

    let mut file = match File::open(&args.firmware) {
        Err(why) => panic!("couldn't open {}: {}", filename, why),
        Ok(file) => file,
    };

    let metadata = match file.metadata() {
        Err(why) => panic!("unable to get metadata for {}: {}", filename, why),
        Ok(metadata) => metadata
    };

    let blocks = metadata.len() / 512;
    if metadata.len() % 512 > 0 {
        println!("warning: {} doesn't end on a 512 byte block boundary", filename);
    }

    println!("Uploading");
    println!("-------------------------------");
    println!("  {}", args.firmware.file_name().unwrap().to_str().unwrap());
    println!("  {} blocks", blocks);
    println!();

    println!("Connecting to programmer");
    protocol::send_sbegin(port.as_mut(), args.verify)
        .expect("Failed to send SBEGIN");

    println!("Waiting for SRSP");
    let response = protocol::read_response(port.as_mut())
        .expect("Failed to read response");

    if response != SRSP {
        println!("Unexpected response for SBEGIN: {}", response as u8);
        return;
    }

    println!("Received SRSP, starting upload");

    // increase the timeout on responses as the programmer needs to write the data
    let timeout = port.timeout();
    port.set_timeout(Duration::from_millis(2000))
        .expect("set timeout failed");

    let mut block_count = 0;
    let upload_result = loop {
        // Read the next 512 bytes from the firmware file
        let mut buffer = vec![0; 512];
        let result = file.read(buffer.as_mut_slice());
        match result {
            Ok(count) => {
                if count == 0 {
                    // Nothing more to read, we are done
                    break Ok(())
                }
                if count != 512 {
                    // We can't deal with incomplete blocks yet
                    break Err(io::Error::new(ErrorKind::Other, "Incomplete read expected 512 bytes"))
                }
            }
            Err(_err) => break Err(_err)
        }

        protocol::send_sdata(port.as_mut(), buffer.as_slice())
            .expect("Send failed");

        match protocol::read_response(&mut port) {
            Ok(response) => {
                if response != SRSP  {
                    break Err(io::Error::new(ErrorKind::InvalidData, "programmer returned error"))
                }

                block_count += 1;
                print!("Block {} of {} uploaded\r", block_count, blocks);
                io::stdout().flush().expect("flush issue on stdout");
            },
            Err(e) => {
                break Err(e);
            }
        }

    };
    println!();

    if upload_result.is_err() {
        println!("Upload failed: {}", upload_result.err().unwrap())
    }

    // restore timeout
    port.set_timeout(timeout)
        .expect("set_timeout failed");

    protocol::send_send(port.as_mut())
        .expect("failed to send SEND");

    println!("Upload completed");
}

fn open_serial_port(portname: String) -> Result<Box<dyn SerialPort>, serialport::Error> {
    let mut port = serialport::new(portname, 115_200)
        .timeout(Duration::from_millis(100))
        .open()?;

    // 8N1
    port.set_data_bits(DataBits::Eight)?;
    port.set_parity(Parity::None)?;
    port.set_stop_bits(StopBits::One)?;

    // Arduino Leonardo need DTR to be high
    port.write_data_terminal_ready(true)?;

    return Ok(port);
}