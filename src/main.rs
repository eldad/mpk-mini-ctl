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
mod operations;
mod u14;

use crate::mpkbank::MpkBankDescriptor;

use clap::{CommandFactory, Parser, Subcommand};
use log::debug;
use std::{fs::File, io::Write};

/// AKAI MPK Mini mkII Control Tool
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, help_template = "
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}")]
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

    /// Passthrough (while snooping) MIDI messages
    Passthrough,

    /// Show bank settings
    ShowBank { bank: u8 },

    /// Show current active settings (RAM)
    ShowRAM,

    /// Read yaml bank descriptor from file and display it
    ReadFile { filename: String },

    /// Dump bank settings as yaml
    DumpBankSettings { bank: u8 },

    /// Dump current active settings (RAM) as yaml
    DumpRAMSettings,

    /// Read yaml bank descriptor from file and apply it on a bank
    LoadBank { filename: String, bank: u8 },

    /// Read yaml bank descriptor from file and apply it to active settings (RAM)
    LoadRAM { filename: String },

    /// Install local bash auto-completion
    Autocompletion {
        #[arg(value_enum)]
        shell: clap_complete::Shell,

        #[arg(long)]
        install: bool,
    },
}

fn read_yaml(filename: &str) -> anyhow::Result<()> {
    let bank_desc: MpkBankDescriptor = serde_yaml::from_reader(File::open(filename)?)?;
    println!("{bank_desc}");
    debug!("{:?}", bank_desc.into_bytes());
    Ok(())
}

fn load_yaml(filename: &str, bank: u8) -> anyhow::Result<()> {
    let bank_desc: MpkBankDescriptor = serde_yaml::from_reader(File::open(filename)?)?;
    operations::set_bank_from_desc(bank, bank_desc)?;
    Ok(())
}

fn autocompletion(shell: clap_complete::Shell, install: bool) -> anyhow::Result<()> {
    let mut output: Box<dyn Write> = match (install, &shell) {
        (false, _) => Box::new(std::io::stdout()),
        (true, clap_complete::Shell::Bash) => {
            let homedir = home::home_dir().ok_or_else(|| anyhow::anyhow!("cannot get homedir"))?;
            let basedir = homedir.join(".local/share/bash-completion/completions");
            std::fs::create_dir_all(&basedir)?;

            let target = basedir.join("mpk-mini-ctl");

            Box::new(File::options().read(false).write(true).create_new(true).open(target)?)
        }
        _ => Err(anyhow::anyhow!(
            "installing autocompletion for this shell is not implemented yet"
        ))?,
    };

    clap_complete::generate(shell, &mut Args::command(), "mpk-mini-ctl", &mut output);
    Ok(())
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
        Command::Snoop => operations::snoop()?,
        Command::ShowBank { bank } => operations::show_bank(bank)?,
        Command::ShowRAM => operations::show_bank(0)?,
        Command::Passthrough => operations::passthrough()?,
        Command::ReadFile { filename } => read_yaml(&filename)?,
        Command::DumpBankSettings { bank } => operations::dump_bank_yaml(bank)?,
        Command::DumpRAMSettings => operations::dump_bank_yaml(0)?,
        Command::LoadBank { filename, bank } => load_yaml(&filename, bank)?,
        Command::LoadRAM { filename } => load_yaml(&filename, 0)?,
        Command::Autocompletion { shell, install } => autocompletion(shell, install)?,
    };

    Ok(())
}
