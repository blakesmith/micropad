#![no_std]

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
    Unknown = 0xFF,
}

impl Response {
    pub fn code(&self) -> u8 {
        *self as u8
    }
}

impl From<u8> for Response {
    fn from(code: u8) -> Response {
        match code {
            0x00 => Response::Ok,
            0x01 => Response::LedDisabled,
            0x02 => Response::UnknownMessage,
            _ => Response::Unknown,
        }
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

impl From<&Message> for MessageFrame {
    fn from(message: &Message) -> Self {
        let mut message_frame = MessageFrame::new();
        match message {
            Message::Ping => {
                message_frame.buf[0] = 0x01;
                for i in 1..4 {
                    message_frame.buf[i] = 0x00;
                }
            },
            _ => {
                for i in 0..4 {
                    message_frame.buf[i] = 0x00;
                }
            }
        }
        message_frame
    }
}
