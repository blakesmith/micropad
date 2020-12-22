#![no_main]
#![no_std]

pub mod encoder;
pub mod hid;

use apa102_spi::{Apa102, PixelOrder};
use embedded_hal::serial::{Read, Write};
use encoder::RotaryEncoder;
use micropad_protocol::{Message, MessageFrame, Response};
use smart_leds::{gamma, SmartLedsWrite};
use smart_leds_trait::RGB8;

use panic_halt as _;

use stm32f0xx_hal as hal;

use hal::{
    delay::Delay,
    gpio::{
        gpioa::{PA0, PA1, PA10, PA2, PA3, PA5, PA6, PA7, PA8, PA9},
        Alternate, Floating, Input, Output, PullDown, PullUp, PushPull, AF0,
    },
    pac,
    pac::{interrupt, Interrupt},
    prelude::*,
    spi,
    usb::UsbBus,
};

use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usbd_serial::SerialPort;

use core::{cell::RefCell, ops::DerefMut};
use cortex_m::{interrupt::free as disable_interrupts, interrupt::Mutex, peripheral::NVIC};
use cortex_m_rt::entry;

use crate::hid::{Key, KeyboardHidClass, MediaCode};

static mut USB_BUS_ALLOC: Option<UsbBusAllocator<UsbBus<hal::usb::Peripheral>>> = None;
static USB_DEV: Mutex<RefCell<Option<UsbDevice<UsbBus<hal::usb::Peripheral>>>>> =
    Mutex::new(RefCell::new(None));
static USB_KEYBOARD: Mutex<RefCell<Option<KeyboardHidClass<UsbBus<hal::usb::Peripheral>>>>> =
    Mutex::new(RefCell::new(None));
static USB_SERIAL: Mutex<RefCell<Option<SerialPort<UsbBus<hal::usb::Peripheral>>>>> =
    Mutex::new(RefCell::new(None));

static CONTROL_STATE: Mutex<RefCell<ControlState>> =
    Mutex::new(RefCell::new(ControlState { led_brightness: 127 }));

struct Devices {
    ok_led: PA10<Output<PushPull>>,
    delay: Delay,
    play_pause: PA0<Input<PullDown>>,
    next: PA2<Input<PullDown>>,
    prev: PA1<Input<PullDown>>,
    enc_btn: PA3<Input<PullUp>>,
    apa102: Apa102<
        spi::Spi<
            hal::stm32::SPI1,
            PA5<Alternate<AF0>>,
            PA6<Alternate<AF0>>,
            PA7<Alternate<AF0>>,
            spi::EightBit,
        >,
    >,
    encoder: RotaryEncoder<PA8<Input<Floating>>, PA9<Input<Floating>>>,
}

#[derive(Clone)]
struct ControlState {
    led_brightness: u8,
}

impl ControlState {
    fn set_led_brightness(&mut self, brightness: u8) {
        self.led_brightness = brightness;
    }

    fn get_led_brightness(&self) -> u8 {
        self.led_brightness
    }
}

struct LEDIndicatorState {
    color: RGB8,
    phase: u16,
}

impl LEDIndicatorState {
    fn new() -> Self {
        Self {
            color: RGB8 { r: 0, g: 0, b: 0 },
            phase: 0,
        }
    }

    fn pulse_color(&mut self, color: RGB8, brightness: u8) {
        self.color = color;
        self.phase = (brightness as u16) << 8;
    }

    fn write_if_blinking(
        &mut self,
        apa102: &mut Apa102<
            spi::Spi<
                hal::stm32::SPI1,
                PA5<Alternate<AF0>>,
                PA6<Alternate<AF0>>,
                PA7<Alternate<AF0>>,
                spi::EightBit,
            >,
        >,
    ) {
        if self.phase > 0 {
            self.phase -= 10;
            if self.color.r != 0 {
                self.color.r = (self.phase >> 8) as u8;
            };
            if self.color.g != 0 {
                self.color.g = (self.phase >> 8) as u8;
            };
            if self.color.b != 0 {
                self.color.b = (self.phase >> 8) as u8;
            };
            apa102.write(gamma([self.color].iter().cloned())).unwrap();
        }
    }
}

fn setup() -> Devices {
    let mut peripherals = pac::Peripherals::take().unwrap();
    let mut core = pac::CorePeripherals::take().unwrap();
    disable_interrupts(|cs| {
        let mut rcc = peripherals
            .RCC
            .configure()
            .hsi48()
            .enable_crs(peripherals.CRS)
            .sysclk(48.mhz())
            .pclk(24.mhz())
            .freeze(&mut peripherals.FLASH);

        let gpioa = peripherals.GPIOA.split(&mut rcc);
        let (
            play_pause,
            prev,
            next,
            enc_btn,
            enc_cw,
            enc_ccw,
            mut ok_led,
            sck,
            miso,
            mosi,
            usb_dm,
            usb_dp,
        ) = (
            gpioa.pa0.into_pull_down_input(cs), // Play pause button, has a 10k pull down resistor on the board
            gpioa.pa1.into_pull_down_input(cs), // Next button
            gpioa.pa2.into_pull_down_input(cs), // Prev button
            gpioa.pa3.into_pull_up_input(cs),   // Encoder button
            gpioa.pa8.into_floating_input(cs), // Encoder A, has a 10k pull up resistor on the board
            gpioa.pa9.into_floating_input(cs), // Encoder B, has a 10k pull up resistor on the board
            gpioa.pa10.into_push_pull_output(cs), // LED usr
            gpioa.pa5.into_alternate_af0(cs),  // APA102 SPI SCK
            gpioa.pa6.into_alternate_af0(cs),  // APA102 SPI MISO
            gpioa.pa7.into_alternate_af0(cs),  // APA102 SPI MOSI
            gpioa.pa11,                        // USB dm
            gpioa.pa12,                        // USB dp
        );
        let delay = Delay::new(core.SYST, &rcc);
        let spi = spi::Spi::spi1(
            peripherals.SPI1,
            (sck, miso, mosi),
            spi::Mode {
                polarity: spi::Polarity::IdleLow,
                phase: spi::Phase::CaptureOnFirstTransition,
            },
            1.mhz(),
            &mut rcc,
        );
        let apa102 = Apa102::new_with_options(spi, 4, true, PixelOrder::RBG);
        let encoder = RotaryEncoder::new(enc_cw, enc_ccw);
        let usb = hal::usb::Peripheral {
            usb: peripherals.USB,
            pin_dm: usb_dm,
            pin_dp: usb_dp,
        };

        unsafe {
            let bus_allocator = {
                USB_BUS_ALLOC = Some(UsbBus::new(usb));
                USB_BUS_ALLOC.as_ref().unwrap()
            };
            *USB_KEYBOARD.borrow(cs).borrow_mut() = Some(KeyboardHidClass::new(&bus_allocator));
            *USB_SERIAL.borrow(cs).borrow_mut() = Some(SerialPort::new(&bus_allocator));
            *USB_DEV.borrow(cs).borrow_mut() = Some(
                UsbDeviceBuilder::new(&bus_allocator, UsbVidPid(0xb38, 0x0003))
                    .manufacturer("micropad")
                    .product("micropad")
                    .serial_number("MP00X")
                    .max_packet_size_0(64)
                    .build(),
            );

            core.NVIC.set_priority(Interrupt::USB, 1);
            NVIC::unmask(Interrupt::USB);
        }

        ok_led.set_high().ok();
        Devices {
            ok_led,
            delay,
            play_pause,
            next,
            prev,
            enc_btn,
            apa102,
            encoder,
        }
    })
}

#[entry]
fn main() -> ! {
    let mut devices = setup();

    let led_color_reset: [RGB8; 1] = [RGB8 { r: 0, g: 0, b: 0 }];
    devices
        .apa102
        .write(led_color_reset.iter().cloned())
        .unwrap();

    let mut led_indicator = LEDIndicatorState::new();
    let mut current_encoder_count = 0;
    let mut key: Option<Key> = None;

    loop {
        let control_state = disable_interrupts(|cs| CONTROL_STATE.borrow(cs).borrow().clone());

        // Sample encoder
        let encoder_sample: i32 = devices.encoder.read_count();
        let encoder_diff: i32 = encoder_sample - current_encoder_count;
        current_encoder_count = encoder_sample;

        // Encoder
        if encoder_diff > 0 {
            led_indicator.pulse_color(RGB8 {
                r: 0,
                b: 255,
                g: 255,
            }, control_state.get_led_brightness());
            key = Some(Key::Media(MediaCode::VolumeUp));
        } else if encoder_diff < 0 {
            led_indicator.pulse_color(RGB8 {
                r: 255,
                b: 255,
                g: 0,
            }, control_state.get_led_brightness());
            key = Some(Key::Media(MediaCode::VolumeDown));
        }
        // Buttons
        else if devices.play_pause.is_high().unwrap() {
            led_indicator.pulse_color(RGB8 { r: 0, g: 0, b: 255 }, control_state.get_led_brightness());
            key = Some(Key::Media(MediaCode::PlayPause));
        } else if devices.next.is_high().unwrap() {
            led_indicator.pulse_color(RGB8 { r: 0, g: 255, b: 0 }, control_state.get_led_brightness());
            key = Some(Key::Media(MediaCode::ScanNext));
        } else if devices.prev.is_high().unwrap() {
            led_indicator.pulse_color(RGB8 { r: 255, g: 0, b: 0 }, control_state.get_led_brightness());
            key = Some(Key::Media(MediaCode::ScanPrev));
        } else if devices.enc_btn.is_low().unwrap() {
            led_indicator.pulse_color(RGB8 {
                r: 255,
                g: 255,
                b: 0,
            }, control_state.get_led_brightness());
            key = Some(Key::Media(MediaCode::Mute));
        } else {
            // Encoder diff is zero, and no buttons currently pressed. Reset report.
            key = None;
        }

        disable_interrupts(|cs| {
            if let &mut Some(ref mut keyboard) = USB_KEYBOARD.borrow(cs).borrow_mut().deref_mut() {
                match key {
                    Some(k) => keyboard.add_key(k),
                    None => keyboard.reset_report(),
                };

                if keyboard.report_has_changed() {
                    keyboard.send_media_report();
                }
            }
        });

        led_indicator.write_if_blinking(&mut devices.apa102);

        // Make sure we delay outside of our 'disable_interrupts' block
        if key.is_some() {
            devices.delay.delay_ms(10u32);
        }
    }
}

fn read_into_frame<R>(frame: &mut MessageFrame, reader: &mut R) -> nb::Result<(), R::Error>
where
    R: Read<u8>,
{
    for i in 0..4 {
        match reader.read() {
            Ok(b) => frame.buf[i] = b,
            Err(err) => return Err(err),
        }
    }

    Ok(())
}

fn write_response<W>(
    frame: &mut MessageFrame,
    writer: &mut W,
    response: Response,
) -> nb::Result<(), W::Error>
where
    W: Write<u8>,
{
    frame.buf[0] = response.code();
    for i in 1..4 {
        frame.buf[i] = 0x0; // Zero pad to the frame boundary
    }

    for i in 0..4 {
        writer.write(frame.buf[i])?;
    }
    Ok(())
}

fn poll_usb() {
    disable_interrupts(|cs| {
        if let (&mut Some(ref mut device), &mut Some(ref mut keyboard), &mut Some(ref mut serial)) = (
            USB_DEV.borrow(cs).borrow_mut().deref_mut(),
            USB_KEYBOARD.borrow(cs).borrow_mut().deref_mut(),
            USB_SERIAL.borrow(cs).borrow_mut().deref_mut(),
        ) {
            device.poll(&mut [keyboard, serial]);
            let mut message_frame = MessageFrame::new();

            if let Ok(_) = read_into_frame(&mut message_frame, serial) {
                let message = Message::from(&message_frame);
                match message {
                    Message::Ping => {
                        let _ = write_response(&mut message_frame, serial, Response::Ok);
                    }
                    Message::SetLedBrightness(brightness) => {
                        CONTROL_STATE.borrow(cs).borrow_mut().set_led_brightness(brightness);
                        let _ = write_response(&mut message_frame, serial, Response::Ok);
                    }
                    _ => {
                        let _ =
                            write_response(&mut message_frame, serial, Response::UnknownMessage);
                    }
                };
            };
        }
    });
}

#[interrupt]
fn USB() {
    poll_usb();
}
