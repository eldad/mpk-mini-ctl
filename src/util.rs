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

use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use regex::Regex;

use crate::error::AppError;

macro_rules! append_array {
    ($vec:expr, $arr:expr) => {
        $vec.append(&mut $arr.iter().map(|&x| x).collect());
    };
}

const DEVICE_NAME: &str = "MPKmini2";

pub fn midi_out_connect() -> Result<MidiOutputConnection, AppError> {
    let midi_output = MidiOutput::new(env!("CARGO_PKG_NAME"))?;
    let name = env!("CARGO_PKG_NAME");
    let re = Regex::new(&format!("{DEVICE_NAME} [0-9]+:[0-9]")).unwrap();
    for port in midi_output.ports() {
        let port_name = midi_output.port_name(&port)?;
        if re.is_match(port_name.as_str()) {
            return Ok(midi_output.connect(&port, name)?);
        }
    }
    Err(AppError::MidiOutputPortNotFound(name.to_owned()))
}

pub fn midi_in_connect<F, T: Send>(callback: F, data: T) -> Result<MidiInputConnection<T>, AppError>
where
    F: FnMut(u64, &[u8], &mut T) + Send + 'static,
{
    let mut midi_input = MidiInput::new(env!("CARGO_PKG_NAME"))?;
    midi_input.ignore(Ignore::None);
    let name = env!("CARGO_PKG_NAME");
    let re = Regex::new(&format!("{DEVICE_NAME} [0-9]+:[0-9]")).unwrap();
    for port in midi_input.ports() {
        let port_name = midi_input.port_name(&port)?;
        if re.is_match(port_name.as_str()) {
            return Ok(midi_input.connect(&port, name, callback, data)?);
        }
    }
    Err(AppError::MidiInputPortNotFound(name.to_owned()))
}
