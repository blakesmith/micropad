#![no_main]
#![no_std]

pub mod hid;

use panic_halt as _;

use stm32f0xx_hal as hal;

use hal::usb::UsbBus;
use hal::{
    pac,
    pac::{interrupt, Interrupt},
    prelude::*,
};

use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;

use cortex_m::{interrupt::free as disable_interrupts, peripheral::NVIC};
use cortex_m_rt::entry;

use crate::hid::{Key, KeyboardHidClass, MediaCode};

static mut USB_BUS_ALLOC: Option<UsbBusAllocator<UsbBus<hal::usb::Peripheral>>> = None;
static mut USB_DEV: Option<UsbDevice<UsbBus<hal::usb::Peripheral>>> = None;
static mut USB_KEYBOARD: Option<KeyboardHidClass<UsbBus<hal::usb::Peripheral>>> = None;

#[entry]
fn main() -> ! {
    let mut peripherals = pac::Peripherals::take().unwrap();
    let mut core = pac::CorePeripherals::take().unwrap();
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
        mut ok_led,
        play_pause,
        _next,
        _prev,
        _enc_btn,
        _enc_a,
        _enc_b,
        sck,
        miso,
        mosi,
        usb_dm,
        usb_dp,
    ) = disable_interrupts(move |cs| {
        (
            gpioa.pa10.into_push_pull_output(cs), // LED usr
            gpioa.pa0.into_pull_down_input(cs),   // Play pause button
            gpioa.pa1.into_pull_down_input(cs),   // Next button
            gpioa.pa2.into_pull_down_input(cs),   // Prev button
            gpioa.pa3.into_pull_up_input(cs),     // Encoder button
            gpioa.pa8.into_pull_up_input(cs),     // Encoder A
            gpioa.pa9.into_pull_up_input(cs),     // Encoder B
            gpioa.pa5.into_alternate_af0(cs),
            gpioa.pa6.into_alternate_af0(cs),
            gpioa.pa7.into_alternate_af0(cs),
            gpioa.pa11, // USB dm
            gpioa.pa12, // USB dp
        )
    });
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

    loop {
        disable_interrupts(|_| unsafe {
            USB_KEYBOARD.as_mut().map(|keyboard| {
                if play_pause.is_high().unwrap() {
                    keyboard.add_key(Key::Media(MediaCode::PlayPause));
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
