use embedded_hal::digital::v2::InputPin;

#[derive(Copy, Clone)]
pub enum RotationAction {
    None,
    Clockwise,
    CounterClockwise,
}

/**
 * Computed from the quadrature output table. Algorithm is:
 * 1. Start with pin state 0b00 (both pins low)
 * 2. Moving through the table clockwise, shift the previous two bits twice. Append next bit state to the lower 2 bits of the 4-bit integer. These get assigned ENC_ACTION_ROTATE_CLOCKWISE
 * 3. Moving through the table counter clockwise, shift the previous two bits twice. Append next bit state to the lower 2 bits of the 4-bit integer. These get assigned ENC_ACTION_ROTATE_COUNTER_CLOCKWISE
 * 4. Go through each state in the table again, and repeat each state for the upper and lower bits of the 4-bit integer. These get assigned ENC_ACTION_NONE.
 * 5. Each bit state is equal to an integer between 0 - 15 (The array length of the table below), the value is the direction of the direction the state should move.
 * 6. Any gaps in the table get assigned a value of ENC_ACTION_NONE.
 */
static ENCODER_ACTIONS: &'static [RotationAction] = &[
    RotationAction::None,
    RotationAction::CounterClockwise,
    RotationAction::Clockwise,
    RotationAction::None,
    RotationAction::Clockwise,
    RotationAction::None,
    RotationAction::None,
    RotationAction::CounterClockwise,
    RotationAction::CounterClockwise,
    RotationAction::None,
    RotationAction::None,
    RotationAction::Clockwise,
    RotationAction::None,
    RotationAction::Clockwise,
    RotationAction::CounterClockwise,
    RotationAction::None,
];

pub struct RotaryEncoder<CwPin, CcwPin>
where
    CwPin: InputPin,
    CcwPin: InputPin,
{
    /// The current state of the encoder table
    rotation_state: u8,
    /// The rotation count of the encoder
    count: i32,
    clockwise_pin: CwPin,
    counter_clockwise_pin: CcwPin,
}

impl<CwPin, CcwPin> RotaryEncoder<CwPin, CcwPin>
where
    CwPin: InputPin,
    CcwPin: InputPin,
{
    pub fn new(clockwise_pin: CwPin, counter_clockwise_pin: CcwPin) -> Self {
        Self {
            rotation_state: 0,
            count: 0,
            clockwise_pin,
            counter_clockwise_pin,
        }
    }

    pub fn read_count(&mut self) -> i32 {
        self.count += match self.rotation_action() {
            RotationAction::None => 0,
            RotationAction::Clockwise => 1,
            RotationAction::CounterClockwise => -1,
        };
        self.count
    }

    fn rotation_action(&mut self) -> RotationAction {
        let cw_pin = self.clockwise_pin.is_high().map_or(0, |b| b as u8);
        let ccw_pin = self.counter_clockwise_pin.is_high().map_or(0, |b| b as u8);
        self.rotation_state <<= 2; // Retain the previous pin state as the upper two bits
        self.rotation_state |= ((cw_pin << 1) | ccw_pin) & 0x03; // Shift the current state onte the lower 2 bits
        let lookup_index = self.rotation_state & 0x0F; // Only keep the bottom 4 bits, throw away the upper 4, and lookup in our rotation table
        ENCODER_ACTIONS[lookup_index as usize]
    }
}
