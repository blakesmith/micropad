#![no_std]

pub enum Message {
    Ping,
    SetLedBrightness(u8),
    GetLedBrightness,
    GetModeInfo,
    Unknown,
}

impl Message {
    fn code(&self) -> u8 {
        match self {
            Message::Ping => 0x01,
            Message::SetLedBrightness(_) => 0x02,
            Message::GetLedBrightness => 0x03,
            Message::GetModeInfo => 0x04,
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
    pub fn raw(&self) -> u8 {
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

#[derive(Debug)]
pub enum ResponsePayload {
    None,
    LedBrightness(u8),
    ModeInfo {
        built_in_mode_count: u8,
        user_mode_count: u8,
        current_mode_index: u8,
    },
}

impl ResponsePayload {
    pub fn fill(&self, frame: &mut MessageFrame) {
        match self {
            ResponsePayload::None => {
                for i in 1..frame.frame_size() {
                    frame.buf[i] = 0x00;
                }
            }
            ResponsePayload::LedBrightness(brightness) => {
                frame.buf[1] = *brightness;
                for i in 2..frame.frame_size() {
                    frame.buf[i] = 0x00;
                }
            }
            ResponsePayload::ModeInfo {
                built_in_mode_count,
                user_mode_count,
                current_mode_index,
            } => {
                frame.buf[1] = *built_in_mode_count;
                frame.buf[2] = *user_mode_count;
                frame.buf[3] = *current_mode_index;
                for i in 4..frame.frame_size() {
                    frame.buf[i] = 0x00;
                }
            }
        }
    }

    fn from_message(message: &Message, response_frame: &MessageFrame) -> ResponsePayload {
        match message {
            Message::Ping | Message::SetLedBrightness(_) | Message::Unknown => {
                ResponsePayload::None
            }
            Message::GetLedBrightness => ResponsePayload::LedBrightness(response_frame.buf[1]),
            Message::GetModeInfo => ResponsePayload::ModeInfo {
                built_in_mode_count: response_frame.buf[1],
                user_mode_count: response_frame.buf[2],
                current_mode_index: response_frame.buf[3],
            },
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

    pub fn into_code_and_payload(self, message: &Message) -> (ResponseCode, ResponsePayload) {
        let code = ResponseCode::from(self.buf[0]);
        let payload = ResponsePayload::from_message(message, &self);
        (code, payload)
    }
}

impl From<&MessageFrame> for Message {
    fn from(frame: &MessageFrame) -> Message {
        match frame.buf[0] {
            0x01 => Message::Ping,
            0x02 => Message::SetLedBrightness(frame.buf[1]),
            0x03 => Message::GetLedBrightness,
            0x04 => Message::GetModeInfo,
            _ => Message::Unknown,
        }
    }
}

impl From<&Message> for MessageFrame {
    fn from(message: &Message) -> Self {
        let mut message_frame = MessageFrame::new();
        match message {
            Message::Ping | Message::GetLedBrightness | Message::GetModeInfo => {
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
