#![no_std]

pub enum Message {
    Ping,
    SetLedBrightness(u8),
    GetLedBrightness,
    Unknown,
}

impl Message {
    fn code(&self) -> u8 {
        match self {
            Message::Ping => 0x01,
            Message::SetLedBrightness(_) => 0x02,
            Message::GetLedBrightness => 0x03,
            Message::Unknown => 0xFF,
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ResponseCode {
    Ok = 0x00,
    UnknownMessage = 0x01,
    Unknown = 0xFF,
}

impl ResponseCode {
    pub fn code(&self) -> u8 {
        *self as u8
    }
}

impl From<u8> for ResponseCode {
    fn from(code: u8) -> ResponseCode {
        match code {
            0x00 => ResponseCode::Ok,
            0x01 => ResponseCode::UnknownMessage,
            _ => ResponseCode::Unknown,
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
            0x03 => Message::GetLedBrightness,
            _ => Message::Unknown,
        }
    }
}

impl From<&Message> for MessageFrame {
    fn from(message: &Message) -> Self {
        let mut message_frame = MessageFrame::new();
        match message {
            Message::Ping | Message::GetLedBrightness => {
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
            Message::Unknown => {
                for i in 0..message_frame.frame_size() {
                    message_frame.buf[i] = 0x00;
                }
            }
        }
        message_frame
    }
}
