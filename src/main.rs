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

mod error;

#[macro_use]
mod util;

mod mpkbank;
mod mpkmidi;
mod u14;
mod operations;

// fn cmd_dump_bank_yaml(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
//     let bank = matches.value_of("bank").unwrap().parse::<u8>()?;
//     dump_bank_yaml(bank)?;
//     Ok(())
// }

// fn cmd_dump_yaml(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
//     match matches.subcommand_name() {
//         Some("bank") => cmd_dump_bank_yaml(matches.subcommand_matches("bank").unwrap()),
//         Some("ram") => dump_bank_yaml(0),
//         _ => Err(Box::new(RuntimeError::new("please provide a valid command."))),
//     }
// }

// fn cmd_read_yaml(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
//     let filename = matches.value_of("filename").unwrap().parse::<String>()?;
//     let bank_desc: MpkBankDescriptor = serde_yaml::from_reader(File::open(&filename)?)?;
//     println!("{}", bank_desc);
//     debug!("{:?}", bank_desc.into_bytes());
//     Ok(())
// }

// fn cmd_send_yaml(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
//     let filename = matches.value_of("filename").unwrap().parse::<String>()?;
//     let bank_desc: MpkBankDescriptor = serde_yaml::from_reader(File::open(&filename)?)?;
//     let bank = matches.value_of("destination").unwrap().parse::<u8>()?;
//     set_bank_from_desc(bank, bank_desc)?;
//     Ok(())
// }

// fn app() -> Result<(), Box<dyn Error>> {
//     let matches = App::new(env!("CARGO_PKG_NAME"))
//         .version(env!("CARGO_PKG_VERSION"))
//         .author(env!("CARGO_PKG_AUTHORS"))
//         .about(env!("CARGO_PKG_DESCRIPTION"))
//         .subcommand(
//             SubCommand::with_name("dump")
//                 .about("Dump settings")
//                 .subcommand(
//                     SubCommand::with_name("bank")
//                         .about("Dump bank settings as yaml")
//                         .arg(Arg::with_name("bank").index(1).required(true)),
//                 )
//                 .subcommand(SubCommand::with_name("ram").about("Dump current active settings (RAM) as yaml")),
//         )
//         .subcommand(SubCommand::with_name("passthrough").about("Passthrough (while snooping) MIDI messages"))
//         .subcommand(
//             SubCommand::with_name("read")
//                 .about("Read yaml bank descriptor from file and display it")
//                 .arg(Arg::with_name("filename").index(1).required(true)),
//         )
//         .subcommand(
//             SubCommand::with_name("send")
//                 .about("Read yaml bank descriptor from file and send it to the device")
//                 .arg(Arg::with_name("filename").index(1).required(true))
//                 .arg(
//                     Arg::with_name("destination")
//                         .index(2)
//                         .required(true)
//                         .help("0 for RAM, 1-4 for banks"),
//                 ),
//         )

//
//     match matches.subcommand_name() {
//         Some("dump") => cmd_dump_yaml(matches.subcommand_matches("dump").unwrap()),
//         Some("passthrough") => passthrough(),
//         Some("read") => cmd_read_yaml(matches.subcommand_matches("read").unwrap()),
//         Some("send") => cmd_send_yaml(matches.subcommand_matches("send").unwrap()),
//     }
// }

use clap::{Parser, Subcommand};

/// AKAI MPK Mini mkII Control Tool
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
   /// Prints debugging information
   #[arg(long)]
   debug: bool,

   #[command(subcommand)]
   command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Snoop MIDI messages
    Snoop,

    /// Show bank settings
    ShowBank { bank: u8 },

    /// Show current active settings (RAM)
    ShowRAM,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let log_level = match args.debug {
        false => simplelog::LevelFilter::Info,
        true => simplelog::LevelFilter::Debug,
    };

    simplelog::CombinedLogger::init(vec![simplelog::TermLogger::new(
        log_level,
        simplelog::Config::default(),
        simplelog::TerminalMode::Stderr,
        simplelog::ColorChoice::Auto,
    )])?;

    match args.command {
        Command::Snoop => operations::snoop(),
        Command::ShowBank { bank } => operations::show_bank(bank),
        Command::ShowRAM => operations::show_bank(0),
    }?;

    Ok(())
}
