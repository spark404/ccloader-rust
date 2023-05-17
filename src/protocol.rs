use std::io;
use std::io::{ErrorKind, Read, Write};
use crate::CRC_16;
use crate::protocol::OpCodes::{ERRO, SBEGIN, SDATA, SEND, SRSP, UNKN};

#[repr(u8)]
#[derive(PartialEq, PartialOrd)]
pub enum OpCodes {
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


pub(crate) fn send_sbegin(connection: &mut (impl Write + ?Sized), verify: bool) -> Result<(), io::Error> {
    let mut serial_buf: Vec<u8> = vec![0; 1000];

    serial_buf[0] = SBEGIN as u8;
    serial_buf[1] = verify as u8;
    let bytes_written = connection.write(&serial_buf[0..2])?;
    if bytes_written != 2 {
        return Err(io::Error::new(ErrorKind::Other, "Incomplete write to device"));
    }
    Ok(())
}

pub(crate) fn send_send(connection: &mut (impl Write + ?Sized)) -> Result<(), io::Error> {
    let mut serial_buf: Vec<u8> = vec![0; 1000];

    serial_buf[0] = SEND as u8;
    let bytes_written = connection.write(&serial_buf[0..1])?;
    if bytes_written != 2 {
        return Err(io::Error::new(ErrorKind::Other, "Incomplete write to device"));
    }
    Ok(())
}

pub(crate) fn send_sdata(connection: &mut (impl Write + ?Sized), data: &[u8]) -> Result<(), io::Error> {
    let mut serial_buf: Vec<u8> = vec!();

    serial_buf.write(&[SDATA as u8])?;
    serial_buf.write(data)?;

    let crc = CRC_16.checksum(&serial_buf);
    serial_buf.write( &[((crc >> 8) & 0xFF) as u8, (crc & 0xFF) as u8])?;

    let bytes_written = connection.write(&serial_buf)?;
    if bytes_written != serial_buf.len() {
        return Err(io::Error::new(ErrorKind::Other, "Incomplete write to device"));
    }
    Ok(())
}

pub(crate) fn read_response(connection: &mut (impl Read + ?Sized)) -> Result<OpCodes, io::Error> {
    let mut serial_buf: Vec<u8> = vec![0; 1000];
    let result = connection.read(serial_buf.as_mut_slice());
    return match result {
        Ok(n) => {
            if n == 0 {
                return Err(io::Error::new(ErrorKind::Other,"Serial connection closed".to_owned()));
            }
            return Ok(OpCodes::from(serial_buf[0]))
        }
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_sbegin_with_verify() {
        let mut buffer: Vec<u8> = vec!();
        let result = send_sbegin(&mut buffer, true);

        assert!(result.is_ok());
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer[0], 0x01);
        assert_eq!(buffer[1], 0x01);
    }

    #[test]
    fn test_send_sbegin_no_verify() {
        let mut buffer: Vec<u8> = vec!();
        let result = send_sbegin(&mut buffer, false);

        assert!(result.is_ok());
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer[0], 0x01);
        assert_eq!(buffer[1], 0x00);
    }
}