/*
 * Copyright 2017-2022 Eldad Zack
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

use std::sync::mpsc;
use std::thread::sleep;
use std::time::Duration;

use crate::error::*;

use log::{debug, error, info, warn};

use crate::mpkbank::MpkBankDescriptor;
use crate::mpkmidi::*;
use crate::util::*;

fn snoop() -> Result<(), AppError> {
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

fn passthrough() -> Result<(), AppError> {
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

    info!("Passthrough started: MIDI messages from input will be sent to output. Use CTRL-C to stop.");
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

fn get_bank_desc(bank: u8) -> Result<MpkBankDescriptor, AppError> {
    if bank > 4 {
        return Err(AppError::BankIndexOutOfBounds(bank));
    }

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

fn set_bank_from_desc(bank: u8, bank_desc: MpkBankDescriptor) -> Result<(), AppError> {
    if bank > 4 {
        return Err(AppError::BankIndexOutOfBounds(bank));
    }

    let mut midi_out = midi_out_connect()?;
    midi_out.send(&sysex_set_bank(bank, bank_desc))?;
    midi_out.close();

    Ok(())
}

fn show_bank(bank: u8) -> Result<(), AppError> {
    let bank_desc = get_bank_desc(bank)?;
    println!("Bank {}:\n{}", bank, bank_desc);
    Ok(())
}

fn dump_bank_yaml(bank: u8) -> Result<(), AppError> {
    let bank_desc = get_bank_desc(bank)?;
    let serialized = serde_yaml::to_string(&bank_desc).unwrap();
    println!("{}", serialized);
    Ok(())
}
