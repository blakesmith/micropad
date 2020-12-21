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
    pub buf: [u8; 4],
}

impl MessageFrame {
    pub fn new() -> MessageFrame {
        MessageFrame {
            buf: Default::default(),
        }
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
