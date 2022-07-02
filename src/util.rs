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

use crate::error::RuntimeError;
use std::error::Error;

use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use regex::Regex;

macro_rules! check_bank_value {
    ($bank:expr) => {
        if $bank > 4 {
            return Err(Box::new(RuntimeError::new(
                "Bank value must be between 0 and 4 (0 = RAM)",
            )));
        }
    };
}

macro_rules! append_array {
    ($vec:expr, $arr:expr) => {
        $vec.append(&mut $arr.iter().map(|&x| x).collect());
    };
}

const DEVICE_NAME: &str = "MPKmini2";

pub fn midi_out_connect() -> Result<MidiOutputConnection, Box<dyn Error>> {
    let midi_output = MidiOutput::new(env!("CARGO_PKG_NAME"))?;
    let name = env!("CARGO_PKG_NAME");
    let re = Regex::new(&format!("{} [0-9]+:[0-9]", DEVICE_NAME)).unwrap();
    for port in midi_output.ports() {
        let port_name = midi_output.port_name(&port)?;
        if re.is_match(port_name.as_str()) {
            return match midi_output.connect(&port, name) {
                Ok(ret) => Ok(ret),
                Err(e) => Err(Box::new(e)),
            };
        }
    }
    Err(Box::new(RuntimeError::new(&format!(
        "MIDI Out port '{}' not found.",
        name
    ))))
}

pub fn midi_in_connect<F, T: Send>(callback: F, data: T) -> Result<MidiInputConnection<T>, Box<dyn Error>>
where
    F: FnMut(u64, &[u8], &mut T) + Send + 'static,
{
    let mut midi_input = MidiInput::new(env!("CARGO_PKG_NAME"))?;
    midi_input.ignore(Ignore::None);
    let name = env!("CARGO_PKG_NAME");
    let re = Regex::new(&format!("{} [0-9]+:[0-9]", DEVICE_NAME)).unwrap();
    for port in midi_input.ports() {
        let port_name = midi_input.port_name(&port)?;
        if re.is_match(port_name.as_str()) {
            return match midi_input.connect(&port, name, callback, data) {
                Ok(ret) => Ok(ret),
                Err(e) => Err(Box::new(e)),
            };
        }
    }
    Err(Box::new(RuntimeError::new(&format!(
        "MIDI In port '{}' not found.",
        name
    ))))
}
