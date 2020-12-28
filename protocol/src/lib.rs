#![no_std]

pub enum Message {
    Ping,
    SetLedBrightness(u8),
    ChangeLed(u8, u8, u8),
    Unknown,
}

impl Message {
    fn code(&self) -> u8 {
        match self {
            Message::Ping => 0x01,
            Message::SetLedBrightness(_) => 0x02,
            Message::ChangeLed(_, _, _) => 0x03,
            Message::Unknown => 0xFF,
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Response {
    Ok = 0x00,
    UnknownMessage = 0x01,
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
            0x01 => Response::UnknownMessage,
            _ => Response::Unknown,
        }
    }
}

pub struct MessageFrame {
    pub buf: [u8; 8],
}

impl MessageFrame {
    pub fn new() -> MessageFrame {
        MessageFrame {
            buf: Default::default(),
        }
    }

    pub fn frame_size(&self) -> usize {
        self.buf.len()
    }
}

impl From<&MessageFrame> for Message {
    fn from(frame: &MessageFrame) -> Message {
        match frame.buf[0] {
            0x01 => Message::Ping,
            0x02 => Message::SetLedBrightness(frame.buf[1]),
            0x03 => Message::ChangeLed(frame.buf[1], frame.buf[2], frame.buf[3]),
            _ => Message::Unknown,
        }
    }
}

impl From<&Message> for MessageFrame {
    fn from(message: &Message) -> Self {
        let mut message_frame = MessageFrame::new();
        match message {
            Message::Ping => {
                message_frame.buf[0] = message.code();
                for i in 1..message_frame.frame_size() {
                    message_frame.buf[i] = 0x00;
                }
            }
            Message::SetLedBrightness(brightness) => {
                message_frame.buf[0] = message.code();
                message_frame.buf[1] = *brightness;
                for i in 2..message_frame.frame_size() {
                    message_frame.buf[i] = 0x00;
                }
            }
            _ => {
                for i in 0..message_frame.frame_size() {
                    message_frame.buf[i] = 0x00;
                }
            }
        }
        message_frame
    }
}
