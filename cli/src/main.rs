use clap::{App, Arg, SubCommand};
use log;
use log::LevelFilter;
use simple_logger::SimpleLogger;

#[derive(Debug)]
enum IfError {
    Serial(serialport::Error),
}

impl From<serialport::Error> for IfError {
    fn from(err: serialport::Error) -> IfError {
        IfError::Serial(err)
    }
}

fn ping() -> Result<(), IfError> {
    let available_ports = serialport::available_ports()?;
    log::info!("Got {} available serial ports", available_ports.len());
    for port_info in available_ports {
        log::info!(
            "Got port: {}, type: {:?}",
            port_info.port_name,
            port_info.port_type
        );
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
        }
        (unknown, _) => {
            if unknown.is_empty() {
                log::error!("No command provided");
            } else {
                log::error!("Unknown command: {}", unknown);
            }
        }
    }
}
