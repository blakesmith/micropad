use clap::{App, Arg, SubCommand};
use log;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use micropad_protocol::{Message, MessageFrame, Response};

use serialport::{SerialPort, SerialPortInfo, SerialPortType};
use std::time::Duration;

#[derive(Debug)]
enum CliError {
    Serial(serialport::Error),
    Io(std::io::Error),
    NotFound,
}

impl From<serialport::Error> for CliError {
    fn from(err: serialport::Error) -> CliError {
        CliError::Serial(err)
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> CliError {
        CliError::Io(err)
    }
}

fn find_micropads() -> Result<Box<dyn Iterator<Item = SerialPortInfo>>, CliError> {
    let available_ports = serialport::available_ports()?;
    Ok(Box::new(available_ports.into_iter().filter(
        |info| match info.port_type {
            SerialPortType::UsbPort(ref usb_port_info) => match usb_port_info.serial_number {
                Some(ref serial) if serial.starts_with("MP00") => true,
                _ => false,
            },
            _ => false,
        },
    )))
}

fn find_micropad(position: usize) -> Result<SerialPortInfo, CliError> {
    find_micropads()?
        .nth(position)
        .ok_or(CliError::NotFound)
        .map_err(|err| match err {
            CliError::NotFound => {
                log::error!(
                    "Could not find an attached micropad at position: {}, is it plugged in?",
                    position
                );
                CliError::NotFound
            }
            err => err,
        })
}

fn connect_micropad(port_info: &SerialPortInfo) -> Result<Box<dyn SerialPort>, CliError> {
    serialport::new(&port_info.port_name, 11520)
        .timeout(Duration::from_millis(500))
        .open()
        .map_err(|err| err.into())
}

fn send_message(message: &Message) -> Result<Response, CliError> {
    let micropad_info = find_micropad(0)?;
    let mut micropad_port = connect_micropad(&micropad_info)?;

    let request_frame = MessageFrame::from(message);
    micropad_port.write(&request_frame.buf)?;
    let mut response_buf: [u8; 1] = [0x0; 1];
    micropad_port.read(&mut response_buf)?;
    Ok(Response::from(response_buf[0]))
}

fn ping() -> Result<(), CliError> {
    match send_message(&Message::Ping)? {
        Response::Ok => log::info!("Got ping response!"),
        response => log::info!("Got non-ok response: {:?}", response)
    }

    Ok(())
}

fn set_led_brightness(brightness: u8) -> Result<(), CliError> {
    match send_message(&Message::SetLedBrightness(brightness))? {
        Response::Ok => log::info!("LED brightness changed to: {}", brightness),
        response => log::info!("Got non-ok response: {:?}", response)
    }

    Ok(())
}

fn main() {
    let matches = App::new("Micropad cli interface")
        .version("0.1")
        .author("Blake Smith <blakesmith0@gmail.com>")
        .about("Control and configure your USB micropad via the command line")
        .arg(
            Arg::with_name("debug")
                .help("Enable debug logging")
                .short("d"),
        )
        .subcommand(
            SubCommand::with_name("ping").about("Ping the micropad to test for connectivity"),
        )
        .subcommand(
            SubCommand::with_name("set_led_brightness").about("Set the LED brightness to 0-255")
            .arg(Arg::with_name("brightness").short("b").required(true).takes_value(true).help("The LED brightness, 0 - 255")))
        .get_matches();

    if matches.is_present("debug") {
        SimpleLogger::new()
            .with_level(LevelFilter::Debug)
            .init()
            .unwrap();
    } else {
        SimpleLogger::new()
            .with_level(LevelFilter::Info)
            .init()
            .unwrap();
    }

    match matches.subcommand() {
        ("ping", Some(_sub_matches)) => {
            log::info!("Pinging device");
            ping().expect("Failed to ping device");
        },
        ("set_led_brightness", Some(brightness_matches)) => {
            let brightness = brightness_matches.value_of("brightness").map(|v| v.parse::<u8>().expect("Brightness must be a value between 0-255!")).unwrap();
            log::info!("Setting LED brightness to: {}", brightness);
            set_led_brightness(brightness).expect("Failed to set LED brightness");
        },
        (unknown, _) => {
            if unknown.is_empty() {
                log::error!("No command provided");
            } else {
                log::error!("Unknown command: {}", unknown);
            }
        }
    }
}
