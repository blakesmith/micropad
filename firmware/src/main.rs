#![no_main]
#![no_std]

pub mod encoder;
pub mod hid;

use apa102_spi::{Apa102, PixelOrder};
use encoder::RotaryEncoder;
use smart_leds::{gamma, SmartLedsWrite};
use smart_leds_trait::RGB8;

use panic_halt as _;

use stm32f0xx_hal as hal;

use hal::{
    gpio::{
        gpioa::{PA0, PA1, PA2, PA5, PA6, PA7, PA8, PA9},
        Alternate, Floating, Input, PullDown, AF0,
    },
    pac,
    pac::{interrupt, Interrupt},
    prelude::*,
    spi,
    usb::UsbBus,
};

use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;

use cortex_m::{interrupt::free as disable_interrupts, peripheral::NVIC};
use cortex_m_rt::entry;

use crate::hid::{Key, KeyboardHidClass, MediaCode};

static mut USB_BUS_ALLOC: Option<UsbBusAllocator<UsbBus<hal::usb::Peripheral>>> = None;
static mut USB_DEV: Option<UsbDevice<UsbBus<hal::usb::Peripheral>>> = None;
static mut USB_KEYBOARD: Option<KeyboardHidClass<UsbBus<hal::usb::Peripheral>>> = None;

struct Devices {
    play_pause: PA0<Input<Floating>>,
    next: PA1<Input<PullDown>>,
    prev: PA2<Input<PullDown>>,
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
            next,
            prev,
            _enc_btn,
            enc_cw,
            enc_ccw,
            mut ok_led,
            sck,
            miso,
            mosi,
            usb_dm,
            usb_dp,
        ) = (
            gpioa.pa0.into_floating_input(cs), // Play pause button, has a 10k pull down resistor on the board
            gpioa.pa1.into_pull_down_input(cs), // Next button
            gpioa.pa2.into_pull_down_input(cs), // Prev button
            gpioa.pa3.into_pull_up_input(cs),  // Encoder button
            gpioa.pa8.into_floating_input(cs), // Encoder A, has a 10k pull up resistor on the board
            gpioa.pa9.into_floating_input(cs), // Encoder B, has a 10k pull up resistor on the board
            gpioa.pa10.into_push_pull_output(cs), // LED usr
            gpioa.pa5.into_alternate_af0(cs),  // APA102 SPI SCK
            gpioa.pa6.into_alternate_af0(cs),  // APA102 SPI MISO
            gpioa.pa7.into_alternate_af0(cs),  // APA102 SPI MOSI
            gpioa.pa11,                        // USB dm
            gpioa.pa12,                        // USB dp
        );
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
            USB_KEYBOARD = Some(KeyboardHidClass::new(&bus_allocator));
            USB_DEV = Some(
                UsbDeviceBuilder::new(&bus_allocator, UsbVidPid(0xb38, 0x0003))
                    .manufacturer("micropad")
                    .product("micropad")
                    .serial_number("DS")
                    .max_packet_size_0(64)
                    .build(),
            );
            core.NVIC.set_priority(Interrupt::USB, 1);
            NVIC::unmask(Interrupt::USB);
        }

        ok_led.set_high().ok();
        Devices {
            play_pause,
            next,
            prev,
            apa102,
            encoder,
        }
    })
}

#[entry]
fn main() -> ! {
    let mut devices = setup();

    let led_color_reset: [RGB8; 2] = [RGB8 { r: 0, g: 0, b: 0 }, RGB8 { r: 0, g: 0, b: 0 }];
    let led_color_play_pause: [RGB8; 2] = [RGB8 { r: 0, g: 64, b: 0 }, RGB8 { r: 64, g: 0, b: 0 }];
    let led_color_next: [RGB8; 2] = [RGB8 { r: 0, g: 0, b: 64 }, RGB8 { r: 0, g: 64, b: 0 }];
    let led_color_prev: [RGB8; 2] = [RGB8 { r: 64, g: 0, b: 0 }, RGB8 { r: 0, g: 0, b: 64 }];

    devices
        .apa102
        .write(led_color_reset.iter().cloned())
        .unwrap();

    let mut current_encoder_count = 0;

    loop {
        // Sample encoder
        let encoder_sample = devices.encoder.read_count();
        let encoder_diff = encoder_sample - current_encoder_count;
        current_encoder_count = encoder_sample;

        disable_interrupts(|_| unsafe {
            USB_KEYBOARD.as_mut().map(|keyboard| {
                // Encoder volume controls
                for _ in 0..encoder_diff.abs() {
                    if encoder_diff > 0 {
                        keyboard.add_key(Key::Media(MediaCode::VolumeUp));
                    } else {
                        keyboard.add_key(Key::Media(MediaCode::VolumeDown));
                    }
                }
                // Buttons
                if devices.play_pause.is_high().unwrap() {
                    devices
                        .apa102
                        .write(gamma(led_color_play_pause.iter().cloned()))
                        .unwrap();
                    keyboard.add_key(Key::Media(MediaCode::PlayPause));
                } else if devices.next.is_high().unwrap() {
                    devices
                        .apa102
                        .write(gamma(led_color_next.iter().cloned()))
                        .unwrap();
                    keyboard.add_key(Key::Media(MediaCode::ScanNext));
                } else if devices.prev.is_high().unwrap() {
                    devices
                        .apa102
                        .write(gamma(led_color_prev.iter().cloned()))
                        .unwrap();
                    keyboard.add_key(Key::Media(MediaCode::ScanPrev));
                } else {
                    keyboard.reset_report();
                }

                if keyboard.report_has_changed() {
                    keyboard.send_media_report();
                }
            });
        });
    }
}

fn poll_usb() {
    unsafe {
        disable_interrupts(|_| {
            USB_DEV.as_mut().map(|device| {
                USB_KEYBOARD.as_mut().map(|keyboard| {
                    device.poll(&mut [keyboard]);
                });
            });
        })
    }
}

#[interrupt]
fn USB() {
    poll_usb();
}
