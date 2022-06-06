/*
 * Copyright 2017 Eldad Zack
 *
 * Permission is hereby granted, free of charge, to any person obtaining a
 * copy of this software and associated documentation files (the "Software"),
 * to deal in the Software without restriction, including without
 * limitation the rights to use, copy, modify, merge, publish, distribute,
 * sublicense, and/or sell copies of the Software, and to permit persons to
 * whom the Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
 * THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 *
 * https://opensource.org/licenses/MIT
 *
 */

use std::error::Error;
use std::fs::File;
use std::sync::mpsc;
use std::thread::sleep;
use std::time::Duration;

mod error;
use crate::error::*;

use log::{debug, error, info, log, warn};

#[macro_use]
mod util;

mod mpkbank;
mod mpkmidi;
mod u14;

use clap::{App, Arg, ArgMatches, SubCommand};

use crate::mpkbank::MpkBankDescriptor;
use crate::mpkmidi::*;
use crate::util::*;

fn snoop() -> Result<(), Box<dyn Error>> {
    let cb = |_, bytes: &[u8], _: &mut _| {
        debug!("rx bytes: {:?}", bytes);
        match MpkMidiMessage::parse_msg(bytes) {
            Ok(m) => println!("{:?}", m),
            Err(e) => warn!("Unparsed: {}; bytes: {:?}", e, bytes),
        }
    };
    let _midi_in = midi_in_connect(cb, ())?;
    info!("Snoop started. Use CTRL-C to stop.");
    loop {
        sleep(Duration::from_millis(250));
    }
}

fn passthrough() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel();

    let cb = move |_, bytes: &[u8], _: &mut _| {
        debug!("rx bytes: {:?}", bytes);
        match MpkMidiMessage::parse_msg(bytes) {
            Ok(m) => println!("{:?}", m),
            Err(e) => warn!("Unparsed: {}; bytes: {:?}", e, bytes),
        }
        if let Err(e) = tx.send(Vec::from(bytes)) {
            error!("Error while sending: {}", e);
        }
    };

    let mut midi_out = midi_out_connect()?;
    let _midi_in = midi_in_connect(cb, ())?;

    info!(
        "Passthrough started: MIDI messages from input will be sent to output. Use CTRL-C to stop."
    );
    loop {
        match rx.recv() {
            Ok(m) => {
                if let Err(e) = midi_out.send(m.as_slice()) {
                    error!("Error while forwarding: {}", e);
                }
            }
            Err(e) => {
                error!("Error while receiving: {}", e);
            }
        }
    }
}

fn get_bank_desc(bank: u8) -> Result<MpkBankDescriptor, Box<dyn Error>> {
    check_bank_value!(bank);

    let (tx, rx) = mpsc::channel();

    let cb = move |_, bytes: &[u8], _: &mut _| {
        if let Ok(m) = MpkMidiMessage::parse_msg(bytes) {
            if let MpkMidiMessage::Bank(bank_rx, d) = m {
                if bank != bank_rx {
                    error!("Error: received bank {}, expected {}", bank_rx, bank);
                }
                if let Err(e) = tx.send(d) {
                    error!("Error while sending on channel: {}", e);
                }
            } else {
                warn!("Unexpected message (ignored): {:?}", m);
            }
        } else {
            warn!("Unparsed: {:?}", bytes);
        }
    };

    let mut midi_out = midi_out_connect()?;
    let midi_in = midi_in_connect(cb, ())?;

    midi_out.send(sysex_get_bank(bank).as_slice())?;
    let bank_desc = rx.recv_timeout(Duration::new(10, 0))?;

    midi_out.close();
    midi_in.close();

    Ok(bank_desc)
}

fn set_bank_from_desc(bank: u8, bank_desc: MpkBankDescriptor) -> Result<(), Box<dyn Error>> {
    check_bank_value!(bank);

    let mut midi_out = midi_out_connect()?;
    midi_out.send(&sysex_set_bank(bank, bank_desc))?;
    midi_out.close();

    Ok(())
}

fn show_bank(bank: u8) -> Result<(), Box<dyn Error>> {
    let bank_desc = get_bank_desc(bank)?;
    println!("Bank {}:\n{}", bank, bank_desc);
    Ok(())
}

fn dump_bank_yaml(bank: u8) -> Result<(), Box<dyn Error>> {
    let bank_desc = get_bank_desc(bank)?;
    let serialized = serde_yaml::to_string(&bank_desc).unwrap();
    println!("{}", serialized);
    Ok(())
}

fn cmd_show_bank(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let bank = matches.value_of("bank").unwrap().parse::<u8>()?;
    show_bank(bank)?;
    Ok(())
}

fn cmd_dump_bank_yaml(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let bank = matches.value_of("bank").unwrap().parse::<u8>()?;
    dump_bank_yaml(bank)?;
    Ok(())
}

fn cmd_show(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    match matches.subcommand_name() {
        Some("bank") => cmd_show_bank(matches.subcommand_matches("bank").unwrap()),
        Some("ram") => show_bank(0),
        _ => Err(Box::new(RuntimeError::new(
            "please provide a valid command.",
        ))),
    }
}

fn cmd_dump_yaml(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    match matches.subcommand_name() {
        Some("bank") => cmd_dump_bank_yaml(matches.subcommand_matches("bank").unwrap()),
        Some("ram") => dump_bank_yaml(0),
        _ => Err(Box::new(RuntimeError::new(
            "please provide a valid command.",
        ))),
    }
}

fn cmd_read_yaml(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let filename = matches.value_of("filename").unwrap().parse::<String>()?;
    let bank_desc: MpkBankDescriptor = serde_yaml::from_reader(File::open(&filename)?)?;
    println!("{}", bank_desc);
    debug!("{:?}", bank_desc.into_bytes());
    Ok(())
}

fn cmd_send_yaml(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let filename = matches.value_of("filename").unwrap().parse::<String>()?;
    let bank_desc: MpkBankDescriptor = serde_yaml::from_reader(File::open(&filename)?)?;
    let bank = matches.value_of("destination").unwrap().parse::<u8>()?;
    set_bank_from_desc(bank, bank_desc)?;
    Ok(())
}

fn app() -> Result<(), Box<dyn Error>> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            SubCommand::with_name("show")
                .about("Show commands")
                .subcommand(
                    SubCommand::with_name("bank")
                        .about("Show bank settings")
                        .arg(Arg::with_name("bank").index(1).required(true)),
                )
                .subcommand(
                    SubCommand::with_name("ram").about("Show current active settings (RAM)"),
                ),
        )
        .subcommand(
            SubCommand::with_name("dump")
                .about("Dump settings")
                .subcommand(
                    SubCommand::with_name("bank")
                        .about("Dump bank settings as yaml")
                        .arg(Arg::with_name("bank").index(1).required(true)),
                )
                .subcommand(
                    SubCommand::with_name("ram")
                        .about("Dump current active settings (RAM) as yaml"),
                ),
        )
        .subcommand(SubCommand::with_name("snoop").about("Snoop MIDI messages"))
        .subcommand(
            SubCommand::with_name("passthrough")
                .about("Passthrough (while snooping) MIDI messages"),
        )
        .subcommand(
            SubCommand::with_name("read")
                .about("Read yaml bank descriptor from file and display it")
                .arg(Arg::with_name("filename").index(1).required(true)),
        )
        .subcommand(
            SubCommand::with_name("send")
                .about("Read yaml bank descriptor from file and send it to the device")
                .arg(Arg::with_name("filename").index(1).required(true))
                .arg(
                    Arg::with_name("destination")
                        .index(2)
                        .required(true)
                        .help("0 for RAM, 1-4 for banks"),
                ),
        )
        .arg(
            Arg::with_name("debug")
                .required(false)
                .long("debug")
                .help("Prints debugging information"),
        )
        .get_matches();

    let log_level = match matches.is_present("debug") {
        false => simplelog::LevelFilter::Info,
        true => simplelog::LevelFilter::Debug,
    };
    simplelog::CombinedLogger::init(vec![simplelog::TermLogger::new(
        log_level,
        simplelog::Config::default(),
    )
    .unwrap()])?;

    match matches.subcommand_name() {
        Some("show") => cmd_show(matches.subcommand_matches("show").unwrap()),
        Some("dump") => cmd_dump_yaml(matches.subcommand_matches("dump").unwrap()),
        Some("snoop") => snoop(),
        Some("passthrough") => passthrough(),
        Some("read") => cmd_read_yaml(matches.subcommand_matches("read").unwrap()),
        Some("send") => cmd_send_yaml(matches.subcommand_matches("send").unwrap()),
        _ => Err(Box::new(RuntimeError::new(
            "please provide a valid command (use 'help' for information)",
        ))),
    }
}

fn main() {
    match app() {
        Ok(_) => (),
        Err(err) => error!("Error: {}", err),
    }
}
