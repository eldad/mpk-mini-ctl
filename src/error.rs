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

use thiserror::Error;

use crate::mpkbank::MPK_BANK_DESCRIPTOR_LENGTH;

/* Runtime Error */

#[derive(Debug, Error)]
pub enum AppError {
    // Parsing

    #[error("Cannot parse note/octave string {0} (expected string with exactly one space)")]
    NoteOctaveParse(String),
    #[error("cannot parse note {0} (from {1})")]
    NoteParse(String, String),
    #[error("Unknown value for toggle: {0}")]
    ToggleUnknown(u8),
    #[error("Unknown padmode value: {0}")]
    PadmodeUnknown(u8),
    #[error("Unknown clock source value: {0}")]
    ClockSourceUnknown(u8),
    #[error("Arpeggiator time division invalid value: {0}")]
    ArpeggiatorTimeDivisionInvalid(u8),
    #[error("Arpeggiator mode invalid: {0}")]
    ArpeggiatorModeInvalid(u8),
    #[error("Swing value invalid: {0}")]
    SwingInvalid(u8),
    #[error("Joystick mode invalid: {0}/{1}/{2}")]
    JoystickModeInvalid(u8, u8, u8),
    #[error("trying to parse knobs with unexpected length {0} (expected 24)")]
    BankKnobsUnexpectedLength(usize),
    #[error("trying to parse pads with unexpected length {0} (expected 64)")]
    BankPadsUnexpectedLength(usize),
    #[error("Unexpected length for bank descriptor ({0}, expected {})", MPK_BANK_DESCRIPTOR_LENGTH)]
    BankDescriptionUnexpectedLength(usize),

    // MIDI
    #[error("SysEx error: {0}")]
    SysEx(String),
    #[error("received empty message")]
    SysExEmptyMessage,
    #[error("received message with MSB unset (<127)")]
    SysExMsbUnset,

    // U14BE
    #[error("U14BE error: MSB set on U14 type from device {0}/{1}")]
    U14BEMsbSet(u8, u8),
    #[error("U14BE error: value too large to convert into u14: {0}")]
    U14BEValueTooLarge(u16),

    // Other
    #[error("Bank value must be between 0 and 4 (0 = RAM), got {0}")]
    BankIndexOutOfBounds(u8),
    #[error("MIDI output port '{0}' not found")]
    MidiOutputPortNotFound(String),
    #[error("MIDI input port '{0}' not found")]
    MidiInputPortNotFound(String),

    // midir
    #[error("Midir InitError: {0}")]
    MidirInitError(#[from] midir::InitError),
    #[error("Midir SendError: {0}")]
    MidirSendError(#[from] midir::SendError),
    #[error("Midir MidirPortInfoError: {0}")]
    MidirPortInfoError(#[from] midir::PortInfoError),
    #[error("Midir connect error {0}")]
    MidirConnectError(String),
    // mpsc
    #[error("mpsc RecvTimeoutError: {0}")]
    MpscRecvTimeoutError(#[from] std::sync::mpsc::RecvTimeoutError)
}

// Special implementation due to thread unsafety types inside of error enum
impl<T> From<midir::ConnectError<T>> for AppError {
    fn from(err: midir::ConnectError<T>) -> Self {
        Self::MidirConnectError(format!("{err}"))
    }
}
