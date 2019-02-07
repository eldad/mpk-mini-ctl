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
use crate::u14::U14BE;
use crate::error::ParseError;

use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::{Visitor, Unexpected};
use serde::de;

use serde_derive::{Serialize, Deserialize};

// Note
#[derive(Copy, Clone, Default)]
pub struct Note {
    value: u8,
}

impl Serialize for Note {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(&format!("{}",self))
    }
}

struct NoteVisitor;
impl<'de> Visitor<'de> for NoteVisitor {
    type Value = Note;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("note and octave string separated by space (e.g., `B# 4`)")
    }

    fn visit_str<E>(self, value: &str) -> Result <Note, E>
        where E: de::Error
    {
        match Note::from_str(value) {
            Ok(n) => Ok(n),
            Err(_) => Err(de::Error::invalid_value(Unexpected::Str(value), &self)),
        }
    }
}

impl<'de> Deserialize<'de> for Note {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        Ok(deserializer.deserialize_str(NoteVisitor)?)
    }
}

impl Note {
    fn from_str(s: &str) -> Result<Self, ParseError> {
        let note_octave: Vec<&str> = s.split(' ').collect();
        if note_octave.len() != 2 {
            return Err(ParseError::new(&format!("cannot parse note string {} (expected string with exactly one space)", s)));
        }
        let note = match note_octave[0] {
            "C"     => 0,
            "C#/Db" => 1, "C#" => 1, "Db" => 1,
            "D"     => 2,
            "D#/Eb" => 3, "D#" => 3, "Eb" => 3,
            "E"     => 4,
            "F"     => 5,
            "F#/Gb" => 6, "F#" => 6, "Gb" => 6,
            "G"     => 7,
            "G#/Ab" => 8, "G#" => 8, "Ab" => 8,
            "A"     => 9,
            "A#/Bb" => 10, "A#" => 10, "Bb" => 10,
            "B"     => 11,
            _ => return Err(ParseError::new(&format!("cannot parse note {} from {}", note_octave[0], s))),
        };
        let octave: i8 = note_octave[1].parse::<i8>().unwrap(); // TODO: wrap
        Ok(Note { value: ((octave + 1) * 12) as u8 + note })
    }

    pub fn as_str(&self) -> String {
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
            _ => unreachable!(),
        };
        format!("{} {}", note, octave)
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(&self.as_str())
    }
}

impl fmt::Debug for Note {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(&format!("{} ({})", self.as_str(), self.value))
    }
}

#[test]
fn test_note_parse_str() {
    assert_eq!(0, Note::from_str("C -1").unwrap().value);
    assert_eq!(1, Note::from_str("C# -1").unwrap().value);
    assert_eq!(1, Note::from_str("C#/Db -1").unwrap().value);
    assert_eq!(127, Note::from_str("G 9").unwrap().value);
}

// Toggle
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
enum Toggle {
    Off = 0,
    On = 1,
}

impl Toggle {
    fn from(value: u8) -> Result<Self, ParseError> {
        match value {
            0 => Ok(Toggle::Off),
            1 => Ok(Toggle::On),
            _ => Err(ParseError::new(&format!("Unknown value for toggle {}", value))),
        }
    }
}

// Knob
#[derive(Serialize, Deserialize, Copy, Clone, Default)]
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
    fn from(raw: [u8; 3]) -> Self {
        Knob {
            control: raw[0],
            min: raw[1],
            max: raw[2],
        }
    }

    fn to_bytes(&self) -> [u8; 3] {
        [self.control, self.min, self.max]
    }
}

// PadMode
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
enum PadMode {
    Momentary = 0,
    Toggle = 1,
}

impl PadMode {
    fn from(value: u8) -> Result<Self, ParseError> {
        match value {
            0 => Ok(PadMode::Momentary),
            1 => Ok(PadMode::Toggle),
            _ => Err(ParseError::new(&format!("Unknown padmode value {}", value))),
        }
    }
}

impl Default for PadMode {
    fn default() -> Self {
        PadMode::Momentary
    }
}

// Pad
#[derive(Serialize, Deserialize, Copy, Clone, Default)]
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
    fn from(value: [u8; 4]) -> Result<Self, ParseError> {
        Ok(Pad {
            note: Note { value: value[0] },
            program: value[1],
            control: value[2],
            mode: PadMode::from(value[3])?,
        })
    }

    fn to_bytes(&self) -> [u8; 4] {
        [self.note.value, self.control, self.program, self.mode as u8]
    }
}

// ClockSource
#[derive(Serialize, Deserialize, Debug)]
enum ClockSource {
    Internal,
    External,
}

impl ClockSource {
    fn from(value: u8) -> Result<Self, ParseError> {
        match value {
            0 => Ok(ClockSource::Internal),
            1 => Ok(ClockSource::External),
            _ => Err(ParseError::new(&format!("Unknown clock source value {}", value))),
        }
    }
}

// ArpeggiatorTimeDivision
#[derive(Serialize, Deserialize, Debug)]
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
    fn from(value: u8) -> Result<Self, ParseError> {
        match value {
            0 => Ok(ArpeggiatorTimeDivision::_4),
            1 => Ok(ArpeggiatorTimeDivision::_4T),
            2 => Ok(ArpeggiatorTimeDivision::_8),
            3 => Ok(ArpeggiatorTimeDivision::_8T),
            4 => Ok(ArpeggiatorTimeDivision::_16),
            5 => Ok(ArpeggiatorTimeDivision::_16T),
            6 => Ok(ArpeggiatorTimeDivision::_32),
            7 => Ok(ArpeggiatorTimeDivision::_32T),
            _ => Err(ParseError::new(&format!("Invalid arpeggiator time division {}", value))),
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
#[derive(Serialize, Deserialize, Debug)]
enum ArpeggiatorMode {
    Up = 0,
    Down = 1,
    Exclusive = 2,
    Inclusive = 3,
    Order = 4,
    Random = 5,
}

impl ArpeggiatorMode {
    fn from(value: u8) -> Result<Self, ParseError> {
        match value {
            0 => Ok(ArpeggiatorMode::Up),
            1 => Ok(ArpeggiatorMode::Down),
            2 => Ok(ArpeggiatorMode::Exclusive),
            3 => Ok(ArpeggiatorMode::Inclusive),
            4 => Ok(ArpeggiatorMode::Order),
            5 => Ok(ArpeggiatorMode::Random),
            _ => Err(ParseError::new(&format!("Invalid arpeggiator mode {}", value))),
        }
    }
}

// Swing
#[derive(Serialize, Deserialize, Debug)]
enum Swing {
    _50 = 0,
    _55 = 1,
    _57 = 2,
    _59 = 3,
    _61 = 4,
    _64 = 5,
}

impl Swing {
    fn from(value: u8) -> Result<Self, ParseError> {
        match value {
            0 => Ok(Swing::_50),
            1 => Ok(Swing::_55),
            2 => Ok(Swing::_57),
            3 => Ok(Swing::_59),
            4 => Ok(Swing::_61),
            5 => Ok(Swing::_64),
            _ => Err(ParseError::new(&format!("Invalid swing value {}", value))),
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
#[derive(Serialize, Deserialize, Debug)]
enum Joystick {
    Pitchbend,
    ControlChannel(u8),
    SplitControlChannels(u8, u8), // X: Left, Right, Y: Up, Down
}

impl Joystick {
    fn from(bytes: [u8; 3]) -> Result<Self, ParseError> {
        match bytes[0] {
            0 => Ok(Joystick::Pitchbend),
            1 => Ok(Joystick::ControlChannel(bytes[1])),
            2 => Ok(Joystick::SplitControlChannels(bytes[1], bytes[2])),
            _ => Err(ParseError::new(&format!("Invalid joystick mode {}", bytes[1]))),
        }
    }

    fn to_bytes(&self) -> [u8; 3] {
        match *self {
            Joystick::Pitchbend => [0; 3],
            Joystick::ControlChannel(c) => { [1, c, 0] },
            Joystick::SplitControlChannels(c1, c2) => { [2, c1, c2 ]}
        }
    }
}

// MpkBankDescriptor
const MPK_BANK_DESCRIPTOR_LENGTH: usize = 108;

#[derive(Serialize, Deserialize)]
pub struct MpkBankDescriptor {
    octave: u8,
    transpose: u8, // -12 (0) .. +12 (24)
    pad_midi_channel: u8,
    keybed_channel: u8,
    joystick_x: Joystick,
    joystick_y: Joystick,

    arpeggiator: Toggle,
    arpeggiator_mode: ArpeggiatorMode,
    arpeggiator_time_division: ArpeggiatorTimeDivision,
    arpeggiator_octave: u8, // 0..3
    swing: Swing,
    latch: Toggle,
    clock_source: ClockSource,
    tempo_taps: u8,
    tempo: U14BE,

    knobs: [Knob; 8],
    pads: [Pad; 16],
}

impl fmt::Display for MpkBankDescriptor {
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

impl fmt::Debug for MpkBankDescriptor {
    /* TODO: write raw bytes */
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl MpkBankDescriptor {
    fn parse_knobs(bytes: &[u8]) -> Result<[Knob; 8], ParseError> {
        if bytes.len() != 8 * 3 {
            Err(ParseError::new(&format!("trying to parse knobs with unexpected length {} (expected {})", bytes.len(), 8 * 3)))
        } else {
            let mut knobs: [Knob; 8] = [Knob::default(); 8];
            for i in 0..8 {
                knobs[i] = Knob::from([bytes[i * 3], bytes[i * 3 + 1], bytes[i * 3 + 2]]);
            }
            Ok(knobs)
        }
    }

    fn parse_pads(bytes: &[u8]) -> Result<[Pad; 16], ParseError> {
        if bytes.len() != 16 * 4 {
            Err(ParseError::new(&format!("trying to parse pads with unexpected length {} (expected {})", bytes.len(), 16 * 4)))
        } else {
            let mut pads: [Pad; 16] = [Pad::default(); 16];
            for i in 0..16 {
                pads[i] = Pad::from([bytes[i * 4], bytes[i * 4 + 1], bytes[i * 4 + 2], bytes[i * 4 + 3]])?;
            }
            Ok(pads)
        }
    }

    pub fn from(bytes: &[u8]) -> Result<Self, ParseError> {
        if bytes.len() != MPK_BANK_DESCRIPTOR_LENGTH {
            Err(ParseError::new(&format!("Unexpected length for bank descriptor ({}, expected {})", bytes.len(), MPK_BANK_DESCRIPTOR_LENGTH)))
        } else {
            Ok(MpkBankDescriptor {
                pad_midi_channel: bytes[0],
                keybed_channel: bytes[1],
                octave: bytes[2],
                arpeggiator: Toggle::from(bytes[3])?,
                arpeggiator_mode: ArpeggiatorMode::from(bytes[4])?,
                arpeggiator_time_division: ArpeggiatorTimeDivision::from(bytes[5])?,
                clock_source: ClockSource::from(bytes[6])?,
                latch: Toggle::from(bytes[7])?,
                swing: Swing::from(bytes[8])?,
                tempo_taps: bytes[9],
                tempo: U14BE::from_device([bytes[10], bytes[11]])?,
                arpeggiator_octave: bytes[12],
                joystick_x: Joystick::from([bytes[13], bytes[14], bytes[15]])?,
                joystick_y: Joystick::from([bytes[16], bytes[17], bytes[18]])?,
                pads: MpkBankDescriptor::parse_pads(&bytes[19..83])?,
                knobs: MpkBankDescriptor::parse_knobs(&bytes[83..107])?,
                transpose: bytes[107],
            })
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let mut ret: Vec<u8> = vec!(
            self.pad_midi_channel,
            self.keybed_channel,
            self.octave,
            self.arpeggiator as u8,
            self.arpeggiator_mode as u8,
            self.arpeggiator_time_division as u8,
            self.clock_source as u8,
            self.latch as u8,
            self.swing as u8,
            self.tempo_taps,
        );
        append_array!(ret, self.tempo.to_device().unwrap());
        ret.push(self.arpeggiator_octave);
        append_array!(ret, self.joystick_x.to_bytes());
        append_array!(ret, self.joystick_y.to_bytes());
        for pad in &self.pads {
            append_array!(ret, pad.to_bytes());
        }
        for knob in &self.knobs {
            append_array!(ret, knob.to_bytes());
        }
        ret.push(self.transpose);

        assert_eq!(ret.len(), MPK_BANK_DESCRIPTOR_LENGTH);
        ret
    }
}
