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

use std::fmt;

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
pub fn sysex_get_bank(id: u8) -> Vec<u8> {
    vec!(MIDI_SYSEX, SYSEX_AKAI, 0x00, 0x26, 0x66, 0x00, 0x01, id, MIDI_SYSEX_END)
}

// u14, big endian
struct U14BE {
    host: u16,
}

impl U14BE {
    fn from_device(bytes: [u8; 2]) -> U14BE {
        U14BE { host: ((bytes[0] as u16) << 7) | bytes[1] as u16 }
    }

    fn to_device(&self) -> [u8; 2] {
        if self.host & 0xc000 != 0 {
            panic!("value too large to convert into u14: {}", self.host)
        }
        [
            ((self.host & 0x3f80) >> 7) as u8,
            (self.host & 0x007f) as u8,
        ]
    }
}

impl fmt::Display for U14BE {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.host)
    }
}

impl fmt::Debug for U14BE {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({:?})", self.host, self.to_device())
    }
}

// u14, little endian, only needed for snoop.
macro_rules! u14le_to_u16 {
    ($x:expr, $offset:expr) => {
        $x[$offset] as u16 + (($x[$offset + 1] as u16) << 7)
    };
}

// Note
#[derive(Copy, Clone, Default)]
struct Note {
    value: u8,
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let octave: i8 = (self.value / 12) as i8 - 1;
        let note = match self.value % 12 {
            0 => "C",
            1 => "C#/Db",
            2 => "D",
            3 => "D#/Eb",
            4 => "E",
            5 => "F",
            6 => "F#/Gb",
            7 => "G",
            8 => "G#/Ab",
            9 => "A",
            10 => "A#/Bb",
            11 => "B",
            _ => panic!("internal error"),
        };
        f.pad(&format!("{} {} ({})", note, octave, self.value))
    }
}

// Toggle
#[derive(Copy, Clone, Debug)]
enum Toggle {
    Off,
    On,
}

impl Toggle {
    fn from(value: u8) -> Toggle {
        match value {
            0 => Toggle::Off,
            1 => Toggle::On,
            _ => panic!("Unknown value for toggle {}", value),
        }
    }
}

// Knob
#[derive(Copy, Clone, Default)]
struct Knob {
    control: u8,
    min: u8,
    max: u8,
}

impl fmt::Debug for Knob {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Control: {:3}, Min: {:3}, Max: {:3}", self.control, self.min, self.max)
    }
}

impl Knob {
    fn from(raw: [u8; 3]) -> Knob {
        Knob {
            control: raw[0],
            min: raw[1],
            max: raw[2],
        }
    }
}

// PadMode
#[derive(Copy, Clone, Debug)]
enum PadMode {
    Toggle,
    Momentary,
}

impl PadMode {
    fn from(value: u8) -> PadMode {
        match value {
            0 => PadMode::Momentary,
            1 => PadMode::Toggle,
            _ => panic!("Unknown padmode value {}", value),
        }
    }
}

impl Default for PadMode {
    fn default() -> PadMode {
        PadMode::Momentary
    }
}

// Pad
#[derive(Copy, Clone, Default)]
struct Pad {
    note: Note,
    control: u8,
    program: u8,
    mode: PadMode,
}

impl fmt::Debug for Pad {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Note: {:13}, Control: {:3}, Program: {:3}, Mode: {:?}", self.note, self.control, self.program, self.mode)
    }
}

impl Pad {
    fn from(value: [u8; 4]) -> Pad {
        Pad {
            note: Note { value: value[0] },
            program: value[1],
            control: value[2],
            mode: PadMode::from(value[3]),
        }
    }
}

// ClockSource
#[derive(Copy, Clone, Debug)]
enum ClockSource {
    Internal,
    External,
}

impl ClockSource {
    fn from(value: u8) -> ClockSource {
        match value {
            0 => ClockSource::Internal,
            1 => ClockSource::External,
            _ => panic!("Unknown clock source value {}", value),
        }
    }
}

// ArpeggiatorTimeDivision
#[derive(Copy, Clone, Debug)]
enum ArpeggiatorTimeDivision {
    _4,
    _4T,
    _8,
    _8T,
    _16,
    _16T,
    _32,
    _32T,
}

impl ArpeggiatorTimeDivision {
    fn from(value: u8) -> ArpeggiatorTimeDivision {
        match value {
            0 => ArpeggiatorTimeDivision::_4,
            1 => ArpeggiatorTimeDivision::_4T,
            2 => ArpeggiatorTimeDivision::_8,
            3 => ArpeggiatorTimeDivision::_8T,
            4 => ArpeggiatorTimeDivision::_16,
            5 => ArpeggiatorTimeDivision::_16T,
            6 => ArpeggiatorTimeDivision::_32,
            7 => ArpeggiatorTimeDivision::_32T,
            _ => panic!("Invalid arpeggiator time division {}", value),
        }
    }
}

impl fmt::Display for ArpeggiatorTimeDivision {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let enumrepr = format!("{:?}", self);
        write!(f, "1/{}", &enumrepr[1..])
    }
}

// ArpeggiatorMode
#[derive(Copy, Clone, Debug)]
enum ArpeggiatorMode {
    Up,
    Down,
    Exclusive,
    Inclusive,
    Order,
    Random,
}

impl ArpeggiatorMode {
    fn from(value: u8) -> ArpeggiatorMode {
        match value {
            0 => ArpeggiatorMode::Up,
            1 => ArpeggiatorMode::Down,
            2 => ArpeggiatorMode::Exclusive,
            3 => ArpeggiatorMode::Inclusive,
            4 => ArpeggiatorMode::Order,
            5 => ArpeggiatorMode::Random,
            _ => panic!("Invalid arpeggiator mode {}", value),
        }
    }
}

// Swing
#[derive(Copy, Clone, Debug)]
enum Swing {
    _50,
    _55,
    _57,
    _59,
    _61,
    _64,
}

impl Swing {
    fn from(value: u8) -> Swing {
        match value {
            0 => Swing::_50,
            1 => Swing::_55,
            2 => Swing::_57,
            3 => Swing::_59,
            4 => Swing::_61,
            5 => Swing::_64,
            _ => panic!("Invalid swing value {}", value),
        }
    }
}

impl fmt::Display for Swing {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let enumrepr = format!("{:?}", self);
        write!(f, "{}%", &enumrepr[1..])
    }
}

// Joystick
#[derive(Debug)]
enum Joystick {
    Pitchbend,
    ControlChannel(u8),
    SplitControlChannels(u8, u8), // X: Left, Right, Y: Up, Down
}

impl Joystick {
    fn from(bytes: [u8; 3]) -> Joystick{
        match bytes[0] {
            0 => Joystick::Pitchbend,
            1 => Joystick::ControlChannel(bytes[1]),
            2 => Joystick::SplitControlChannels(bytes[1], bytes[2]),
            _ => panic!("Invalid joystick mode {}", bytes[1]),
        }
    }
}

// MpkBankDescriptor
pub struct MpkBankDescriptor {
    arpeggiator: Toggle,
    octave: u8,
    clock_source: ClockSource,
    arpeggiator_mode: ArpeggiatorMode,
    arpeggiator_time_division: ArpeggiatorTimeDivision,
    arpeggiator_octave: u8, // 0..3
    swing: Swing,
    latch: Toggle,
    pad_midi_channel: u8,
    keybed_channel: u8,
    tempo_taps: u8,
    tempo: U14BE,
    joystick_x: Joystick,
    joystick_y: Joystick,
    knobs: [Knob; 8],
    pads: [Pad; 16],
    transpose: u8, // -12 (0) .. +12 (24)
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
    // System
    Reset,
    // MPKmini2-specific
    Bank(u8, MpkBankDescriptor),
    Unknown(Vec<u8>),
}

impl fmt::Debug for MpkBankDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut sb = String::new();
        sb.push_str(&format!("PAD Channel: {}\n", self.pad_midi_channel + 1));
        sb.push_str(&format!("Keybed Channel: {}\n", self.keybed_channel + 1));
        sb.push_str(&format!("Octave: {}\n", self.octave as i8 - 4));
        sb.push_str(&format!("Transpose: {}\n", self.transpose as i8 - 12));
        sb.push_str(&format!("Arpeggiator: {:?}\n", self.arpeggiator));
        sb.push_str(&format!("Arpeggiator Mode: {:?}\n", self.arpeggiator_mode));
        sb.push_str(&format!("Arpeggiator Time Division: {}\n", self.arpeggiator_time_division));
        sb.push_str(&format!("Arpeggiator Tempo: {}\n", self.tempo));
        sb.push_str(&format!("Arpeggiator Octave: {}\n", self.arpeggiator_octave + 1));
        sb.push_str(&format!("Swing: {}\n", self.swing));
        sb.push_str(&format!("Clock source: {:?}\n", self.clock_source));
        sb.push_str(&format!("Latch: {:?}\n", self.latch));
        sb.push_str(&format!("Tempo taps: {}\n", self.tempo_taps));
        sb.push_str(&format!("Joystick X: {:?}\n", self.joystick_x));
        sb.push_str(&format!("Joystick Y: {:?}\n", self.joystick_y));

        for (i, knob) in self.knobs.iter().enumerate() {
            sb.push_str(&format!("Knob {}: {:?}\n", i + 1, knob));
        }

        for (i, pad) in self.pads.iter().enumerate() {
            let padbank = if i < 8 { "A" } else { "B" };
            sb.push_str(&format!("Pad {}{}: {:?}\n", padbank, i % 8 + 1, pad));
        }
        write!(f, "{}", sb)
    }
}

impl MpkBankDescriptor {
    fn parse_knobs(bytes: &[u8]) -> [Knob; 8] {
        if bytes.len() != 8 * 3 {
            panic!("trying to parse knobs with unexpected length {} (expected {})", bytes.len(), 8 * 3);
        }
        let mut knobs: [Knob; 8] = [Knob::default(); 8];
        for i in 0..8 {
            knobs[i] = Knob::from([bytes[i * 3], bytes[i * 3 + 1], bytes[i * 3 + 2]]);
        }
        knobs
    }

    fn parse_pads(bytes: &[u8]) -> [Pad; 16] {
        if bytes.len() != 16 * 4 {
            panic!("trying to parse pads with unexpected length {} (expected {})", bytes.len(), 16 * 4);
        }
        let mut pads: [Pad; 16] = [Pad::default(); 16];
        for i in 0..16 {
            pads[i] = Pad::from([bytes[i * 4], bytes[i * 4 + 1], bytes[i * 4 + 2], bytes[i * 4 + 3]]);
        }
        pads
    }

    fn from(bytes: &[u8]) -> MpkBankDescriptor {
        if bytes.len() != 108 {
            panic!("Unexpected length for bank descriptor ({}, expected 108)", bytes.len());
        }
        MpkBankDescriptor {
            pad_midi_channel: bytes[0],
            keybed_channel: bytes[1],
            octave: bytes[2],
            arpeggiator: Toggle::from(bytes[3]),
            arpeggiator_mode: ArpeggiatorMode::from(bytes[4]),
            arpeggiator_time_division: ArpeggiatorTimeDivision::from(bytes[5]),
            clock_source: ClockSource::from(bytes[6]),
            latch: Toggle::from(bytes[7]),
            swing: Swing::from(bytes[8]),
            tempo_taps: bytes[9],
            tempo: U14BE::from_device([bytes[10], bytes[11]]),
            arpeggiator_octave: bytes[12],
            joystick_x: Joystick::from([bytes[13], bytes[14], bytes[15]]),
            joystick_y: Joystick::from([bytes[16], bytes[17], bytes[18]]),
            pads: MpkBankDescriptor::parse_pads(&bytes[19..83]),
            knobs: MpkBankDescriptor::parse_knobs(&bytes[83..107]),
            transpose: bytes[107],
        }
    }
}

pub fn parse_sysex(bytes: &[u8]) -> Option<MpkMidiMessage> {
    if bytes.len() < 3 {
        println!("SysEx rx error: runt: {:?}", bytes);
        return None;
    }
    if *bytes.last().unwrap() != MIDI_SYSEX_END {
        println!("SysEx rx error: malformed: {:?}", bytes);
        return None;
    }
    if bytes[1] != SYSEX_AKAI {
        println!("SysEx rx error: non-AKAI: (manufacturer={:x}, expected={:x}) {:?}", bytes[1], SYSEX_AKAI, bytes);
        return None;
    }

    let payload = &bytes[2..bytes.len()-1];
    if payload.starts_with(&SYSEX_MPK_BANK) {
        Some(MpkMidiMessage::Bank(payload[SYSEX_MPK_BANK.len()], MpkBankDescriptor::from(&payload[SYSEX_MPK_BANK.len()+1..])))
    } else {
        println!("WARNING: unknown AKAI sysex message {:?}", payload);
        None
    }
}

fn parse_channel_msg(bytes: &[u8]) -> Option<MpkMidiMessage> {
    let channel = bytes[0] & 0x0f;
    match bytes[0] & 0xf0 {
        MIDI_NOTE_OFF => Some(MpkMidiMessage::NoteOff(channel, bytes[1], bytes[2])),
        MIDI_NOTE_ON => Some(MpkMidiMessage::NoteOn(channel, bytes[1], bytes[2])),
        MIDI_POLYPHONIC_PRESSURE => None,
        MIDI_CONTROL_CHANGE => Some(MpkMidiMessage::ControlChange(channel, bytes[1], bytes[2])),
        MIDI_PROGRAM_CHANGE => Some(MpkMidiMessage::ProgramChange(channel, bytes[1])),
        MIDI_CHANNEL_PRESSURE => None,
        MIDI_PITCH_BEND => Some(MpkMidiMessage::PitchBend(channel, u14le_to_u16!(bytes, 1))),
        _ => None,
    }
}

pub fn parse_msg(bytes: &[u8]) -> Option<MpkMidiMessage> {
    if bytes.len() == 0 {
        panic!("ERROR: received message with length 0");
    }

    if bytes[0] < 127 {
        panic!("ERROR: received message with MSB unset (<127)");
    }

    if bytes[0] & 0xf0 != 0xf0 {
        return parse_channel_msg(bytes);
    }

    match bytes[0] {
        MIDI_SYSEX => parse_sysex(bytes),
        MIDI_RESET => Some(MpkMidiMessage::Reset),
        _ => Some(MpkMidiMessage::Unknown(Vec::from(bytes))),
    }
}