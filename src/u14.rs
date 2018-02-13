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
use error::ParseError;

/* 14 bits unsigned, big endian */
#[derive(Serialize, Deserialize)]
pub struct U14BE {
    host: u16,
}

impl U14BE {
    pub fn from_device(bytes: [u8; 2]) -> Result<U14BE, ParseError> {
        if ((bytes[0] | bytes[1]) & 0x80) == 0x80 {
            Err(ParseError::new(&format!("ERROR: MSB set on U14 type from device: {:?}", bytes)))
        } else {
            Ok(U14BE { host: ((bytes[0] as u16) << 7) | bytes[1] as u16 })
        }
    }

    pub fn to_device(&self) -> Result<[u8; 2], ParseError> {
        if self.host & 0xc000 != 0 {
            Err(ParseError::new(&format!("value too large to convert into u14: {}", self.host)))
        } else {
            Ok([
                ((self.host & 0x3f80) >> 7) as u8,
                (self.host & 0x007f) as u8,
            ])
        }
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