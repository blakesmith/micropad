use embedded_hal::serial::{Read, Write};

pub enum Message {
    Ping,
    DisableLed,
    EnableLed,
    ChangeLed(u8, u8, u8),
    Unknown,
}

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Response {
    Ok = 0x00,
    LedDisabled = 0x01,
    UnknownMessage = 0x02,
}

impl Response {
    pub fn code(&self) -> u8 {
        *self as u8
    }
}

pub struct MessageFrame {
    buf: [u8; 4],
}

impl MessageFrame {
    pub fn new() -> MessageFrame {
        MessageFrame {
            buf: Default::default(),
        }
    }

    pub fn read<R>(&mut self, reader: &mut R) -> nb::Result<(), R::Error>
    where
        R: Read<u8>,
    {
        for i in 0..4 {
            match reader.read() {
                Ok(b) => self.buf[i] = b,
                Err(err) => return Err(err),
            }
        }

        Ok(())
    }

    pub fn write_response<W>(
        &mut self,
        writer: &mut W,
        response: Response,
    ) -> nb::Result<(), W::Error>
    where
        W: Write<u8>,
    {
        self.buf[0] = response.code();
        for i in 1..4 {
            self.buf[i] = 0x0; // Zero pad to the frame boundary
        }

        for i in 0..4 {
            writer.write(self.buf[i])?;
        }
        Ok(())
    }
}

impl From<&MessageFrame> for Message {
    fn from(frame: &MessageFrame) -> Message {
        match frame.buf[0] {
            0x01 => Message::Ping,
            0x02 => Message::DisableLed,
            0x03 => Message::EnableLed,
            0x04 => Message::ChangeLed(frame.buf[1], frame.buf[2], frame.buf[3]),
            _ => Message::Unknown,
        }
    }
}
