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
use std::sync::mpsc;
use std::time::Duration;

mod error;
use error::*;

mod u14;
mod mpkbank;
use mpkbank::MpkBankDescriptor;
mod mpkmidi;
use mpkmidi::*;

extern crate clap;
use clap::{App, SubCommand, Arg, ArgMatches};

extern crate midir;
use midir::{MidiInput, MidiOutput, MidiOutputConnection, MidiInputConnection};

extern crate regex;
use regex::Regex;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_yaml;

const DEVICE_NAME: &str = "MPKmini2";

fn midi_out_connect_by_name(port: MidiOutput, name: &str) -> Result<MidiOutputConnection, Box<Error>> {
    let re = Regex::new(&format!("{} [0-9]+:[0-9]", DEVICE_NAME)).unwrap();
    for i in 0..port.port_count() {
        let port_name = port.port_name(i)?;
        //println!("* OUT: {}", port_name);
        if re.is_match(port_name.as_str()) {
            return match port.connect(i, name) {
                Ok(ret) => Ok(ret),
                Err(e) => Err(Box::new(e)),
            }
        }
    }
    Err(Box::new(RuntimeError::new(&format!("MIDI Out port '{}' not found.", name))))
}

fn midi_in_connect_by_name <F, T: Send> (port: MidiInput, name: &str, callback: F, data: T) -> Result<MidiInputConnection<T>, Box<Error>>
where
    F: FnMut(u64, &[u8], &mut T) + Send + 'static
{
    let re = Regex::new(&format!("{} [0-9]+:[0-9]", DEVICE_NAME)).unwrap();
    for i in 0..port.port_count() {
        let port_name = port.port_name(i)?;
        //println!("* IN: {}", port_name);
        if re.is_match(port_name.as_str()) {
            return match port.connect(i, name, callback, data) {
                Ok(ret) => Ok(ret),
                Err(e) => Err(Box::new(e)),
            }
        }
    }
    Err(Box::new(RuntimeError::new(&format!("MIDI In port '{}' not found.", name))))
}

fn snoop() -> Result<(), Box<Error>> {
    let cb = |_, bytes: &[u8], _: &mut _| {
        match parse_msg(bytes) {
            Ok(m) => println!("{:?}", m),
            Err(e) => println!("Unparsed: {}; bytes: {:?}", e, bytes),
        }
    };
    let _midi_in = midi_in_connect_by_name(MidiInput::new(env!("CARGO_PKG_NAME"))?, env!("CARGO_PKG_NAME"), cb, ())?;
    println!("Snoop started. Use CTRL-C to stop.");
    loop {}
}

fn get_bank_desc(bank: u8) -> Result<MpkBankDescriptor, Box<Error>> {
    if bank > 4 {
        return Err(Box::new(RuntimeError::new("Bank value must be between 0 and 4 (0 = RAM)")))
    }

    let (tx, rx) = mpsc::channel();

    let cb = move |_, bytes: &[u8], _: &mut _| {
        if let Ok(m) = parse_msg(bytes) {
            if let MpkMidiMessage::Bank(bank_rx, d) = m {
                if bank != bank_rx {
                    println!("Error: received bank {}, expected {}", bank_rx, bank);
                }
                if let Err(e) = tx.send(d) {
                    println!("Error while sending on channel: {}", e);
                }
            } else {
                println!("Unexpected message (ignored): {:?}", m);
            }
        } else {
            println!("Unparsed: {:?}", bytes);
        }
    };

    let mut midi_out = midi_out_connect_by_name(MidiOutput::new(env!("CARGO_PKG_NAME"))?, env!("CARGO_PKG_NAME"))?;
    let midi_in = midi_in_connect_by_name(MidiInput::new(env!("CARGO_PKG_NAME"))?, env!("CARGO_PKG_NAME"), cb, ())?;

    midi_out.send(sysex_get_bank(bank).as_slice())?;
    let bank_desc = rx.recv_timeout(Duration::new(10, 0))?;

    midi_out.close();
    midi_in.close();

    Ok(bank_desc)
}

fn show_bank(bank: u8) -> Result<(), Box<Error>> {
    let bank_desc = get_bank_desc(bank)?;
    println!("Bank {}:\n{:?}", bank, bank_desc);
    Ok(())
}

fn dump_bank_yaml(bank: u8) -> Result<(), Box<Error>> {
    let bank_desc = get_bank_desc(bank)?;
    let serialized = serde_yaml::to_string(&bank_desc).unwrap();
    println!("{}", serialized);
    Ok(())
}

fn cmd_show_bank(matches: &ArgMatches) -> Result<(), Box<Error>> {
    let bank = matches.value_of("bank").unwrap().parse::<u8>()?;
    show_bank(bank)?;
    Ok(())
}

fn cmd_dump_bank_yaml(matches: &ArgMatches) -> Result<(), Box<Error>> {
    let bank = matches.value_of("bank").unwrap().parse::<u8>()?;
    dump_bank_yaml(bank)?;
    Ok(())
}

fn cmd_show(matches: &ArgMatches) -> Result<(), Box<Error>> {
    match matches.subcommand_name() {
        Some("bank") => cmd_show_bank(matches.subcommand_matches("bank").unwrap()),
        Some("ram") => show_bank(0),
        _ => Err(Box::new(RuntimeError::new("please provide a valid command."))),
    }
}

fn cmd_dump_yaml(matches: &ArgMatches) -> Result<(), Box<Error>> {
    match matches.subcommand_name() {
        Some("bank") => cmd_dump_bank_yaml(matches.subcommand_matches("bank").unwrap()),
        Some("ram") => dump_bank_yaml(0),
        _ => Err(Box::new(RuntimeError::new("please provide a valid command."))),
    }
}

fn app() -> Result<(), Box<Error>> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(SubCommand::with_name("show")
            .about("show commands")
            .subcommand(SubCommand::with_name("bank")
                .about("show bank settings")
                .arg(Arg::with_name("bank")
                    .index(1)
                    .required(true)
                )
            )
            .subcommand(SubCommand::with_name("ram")
                .about("show current active settings (RAM)"))
        )
        .subcommand(SubCommand::with_name("dump")
            .about("dump settings")
            .subcommand(SubCommand::with_name("bank")
                .about("dump bank settings as yaml")
                .arg(Arg::with_name("bank")
                    .index(1)
                    .required(true)
                )
            )
            .subcommand(SubCommand::with_name("ram")
                .about("dump current active settings (RAM) as yaml"))
        )
        .subcommand(SubCommand::with_name("snoop")
            .about("snoop MIDI messages")
        )
        .get_matches();

    match matches.subcommand_name() {
        Some("show") => cmd_show(matches.subcommand_matches("show").unwrap()),
        Some("dump") => cmd_dump_yaml(matches.subcommand_matches("dump").unwrap()),
        Some("snoop") => snoop(),
        _ => Err(Box::new(RuntimeError::new("please provide a valid command (use 'help' for information)"))),
    }
}

fn main() {
    match app() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err.description())
    }
}