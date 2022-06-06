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

use crate::error::ParseError;
use crate::mpkbank::MpkBankDescriptor;

// https://www.midi.org/specifications/item/table-1-summary-of-midi-message
const MIDI_SYSEX: u8 = 0xf0;
const MIDI_SYSEX_END: u8 = 0xf7;
const SYSEX_AKAI: u8 = 0x47; // See http://www.amei.or.jp/report/System_ID_e.html
const MIDI_RESET: u8 = 0xff;

// Channel messages are in the form 0xMC, where M = message type and C = channel
const MIDI_NOTE_OFF: u8 = 0x80;
const MIDI_NOTE_ON: u8 = 0x90;
const MIDI_POLYPHONIC_PRESSURE: u8 = 0xa0;
const MIDI_CONTROL_CHANGE: u8 = 0xb0;
const MIDI_PROGRAM_CHANGE: u8 = 0xc0;
const MIDI_CHANNEL_PRESSURE: u8 = 0xd0;
const MIDI_PITCH_BEND: u8 = 0xe0;

// MPK-Specific
const SYSEX_MPK_BANK: [u8; 5] = [0x00, 0x26, 0x67, 0x00, 0x6d];
pub fn sysex_get_bank(bank: u8) -> Vec<u8> {
    vec![
        MIDI_SYSEX,
        SYSEX_AKAI,
        0x00,
        0x26,
        0x66,
        0x00,
        0x01,
        bank,
        MIDI_SYSEX_END,
    ]
}

pub fn sysex_set_bank(bank: u8, bank_desc: MpkBankDescriptor) -> Vec<u8> {
    let mut ret = vec![MIDI_SYSEX, SYSEX_AKAI, 0x00, 0x26, 0x64, 0x00, 0x6d, bank];
    append_array!(ret, &bank_desc.into_bytes());
    ret.push(MIDI_SYSEX_END);
    ret
}

// u14, little endian, only needed for snoop.
macro_rules! u14le_to_u16 {
    ($x:expr, $offset:expr) => {
        $x[$offset] as u16 + (($x[$offset + 1] as u16) << 7)
    };
}

#[derive(Debug)]
pub enum MpkMidiMessage {
    // channel, note, velocity
    NoteOff(u8, u8, u8),
    NoteOn(u8, u8, u8),
    // channel, control, value
    ControlChange(u8, u8, u8),
    ProgramChange(u8, u8),
    PitchBend(u8, u16),
    // Unparsed
    Unparsed,
    // System
    Reset,
    // MPKmini2-specific
    Bank(u8, MpkBankDescriptor),
    Unknown(Vec<u8>),
}

impl MpkMidiMessage {
    fn parse_sysex(bytes: &[u8]) -> Result<Self, ParseError> {
        if bytes.len() < 3 {
            return Err(ParseError::new(&format!(
                "SysEx rx error: runt: {:?}",
                bytes
            )));
        }
        if *bytes.last().unwrap() != MIDI_SYSEX_END {
            return Err(ParseError::new(&format!(
                "SysEx rx error: malformed: {:?}",
                bytes
            )));
        }
        if bytes[1] != SYSEX_AKAI {
            return Err(ParseError::new(&format!(
                "SysEx rx error: non-AKAI: (manufacturer={:x}, expected={:x}) {:?}",
                bytes[1], SYSEX_AKAI, bytes
            )));
        }

        let payload = &bytes[2..bytes.len() - 1];
        if payload.starts_with(&SYSEX_MPK_BANK) {
            Ok(MpkMidiMessage::Bank(
                payload[SYSEX_MPK_BANK.len()],
                MpkBankDescriptor::from(&payload[SYSEX_MPK_BANK.len() + 1..])?,
            ))
        } else {
            Err(ParseError::new(&format!(
                "WARNING: unknown AKAI sysex message {:?}",
                payload
            )))
        }
    }

    fn parse_channel_msg(bytes: &[u8]) -> Result<Self, ParseError> {
        let channel = bytes[0] & 0x0f;
        match bytes[0] & 0xf0 {
            MIDI_NOTE_OFF => Ok(MpkMidiMessage::NoteOff(channel, bytes[1], bytes[2])),
            MIDI_NOTE_ON => Ok(MpkMidiMessage::NoteOn(channel, bytes[1], bytes[2])),
            MIDI_POLYPHONIC_PRESSURE => Ok(MpkMidiMessage::Unparsed),
            MIDI_CONTROL_CHANGE => Ok(MpkMidiMessage::ControlChange(channel, bytes[1], bytes[2])),
            MIDI_PROGRAM_CHANGE => Ok(MpkMidiMessage::ProgramChange(channel, bytes[1])),
            MIDI_CHANNEL_PRESSURE => Ok(MpkMidiMessage::Unparsed),
            MIDI_PITCH_BEND => Ok(MpkMidiMessage::PitchBend(channel, u14le_to_u16!(bytes, 1))),
            _ => unreachable!(),
        }
    }

    pub fn parse_msg(bytes: &[u8]) -> Result<Self, ParseError> {
        if bytes.is_empty() {
            return Err(ParseError::new("ERROR: received empty message"));
        }

        if bytes[0] < 127 {
            return Err(ParseError::new(
                "ERROR: received message with MSB unset (<127)",
            ));
        }

        if bytes[0] & 0xf0 != 0xf0 {
            return MpkMidiMessage::parse_channel_msg(bytes);
        }

        match bytes[0] {
            MIDI_SYSEX => MpkMidiMessage::parse_sysex(bytes),
            MIDI_RESET => Ok(MpkMidiMessage::Reset),
            _ => Ok(MpkMidiMessage::Unknown(Vec::from(bytes))),
        }
    }
}
