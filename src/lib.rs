use num::{PrimInt, Unsigned};

pub const CROCKFORD_ALPHABET: [char; 32] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J',
    'K', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'V', 'W', 'X', 'Y', 'Z',
];

fn crockford_index(c: char) -> Option<u8> {
    match c {
        '0' => Some(0),
        '1' => Some(1),
        '2' => Some(2),
        '3' => Some(3),
        '4' => Some(4),
        '5' => Some(5),
        '6' => Some(6),
        '7' => Some(7),
        '8' => Some(8),
        '9' => Some(9),
        'A' => Some(10),
        'B' => Some(11),
        'C' => Some(12),
        'D' => Some(13),
        'E' => Some(14),
        'F' => Some(15),
        'G' => Some(16),
        'H' => Some(17),
        'J' => Some(18),
        'K' => Some(19),
        'M' => Some(20),
        'N' => Some(21),
        'P' => Some(22),
        'Q' => Some(23),
        'R' => Some(24),
        'S' => Some(25),
        'T' => Some(26),
        'V' => Some(27),
        'W' => Some(28),
        'X' => Some(29),
        'Y' => Some(30),
        'Z' => Some(31),
        _ => None,
    }
}

pub fn encode<N>(mut x: N, buffer: &mut [char])
where
    N: Unsigned + PrimInt,
    u8: Into<N>,
{
    for c in buffer.iter_mut().rev() {
        // grab the lowest five bits of x
        let masked = x
            .bitand(0b00011111.into())
            .to_u8()
            .expect("operation will always succeed as 0b00011111 is less than u8::MAX");
        // insert the corresponding character into the buffer
        *c = CROCKFORD_ALPHABET[masked as usize];
        // shift so we can look at the next five bits
        x = x.unsigned_shr(5);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CrockfordDecodeError {
    character: char,
    slice_position: usize,
}

pub fn decode<N>(buffer: &[char]) -> Result<N, CrockfordDecodeError>
where
    N: Unsigned + PrimInt,
    u8: Into<N>,
{
    let mut acc = 0u8.into();
    for (i, &c) in buffer.iter().rev().enumerate() {
        if let Some(bits) = crockford_index(c) {
            let n: N = bits.into();
            let shifted = n.unsigned_shl(5 * i as u32);
            acc = acc.bitor(shifted);
        } else {
            return Err(CrockfordDecodeError {
                character: c,
                slice_position: buffer.len() - i - 1,
            });
        }
    }
    Ok(acc)
}

struct CrockfordEncoder<I>
where
    I: Iterator<Item = u8>,
{
    byte_stream: I,
}

impl<I> CrockfordEncoder<I>
where
    I: Iterator<Item = u8>,
{
    fn new(byte_stream: I) -> Self {
        CrockfordEncoder { byte_stream }
    }
}

impl<I> IntoIterator for CrockfordEncoder<I>
where
    I: Iterator<Item = u8>,
{
    type Item = char;
    type IntoIter = CrockfordEncoderIterator<I>;

    fn into_iter(self) -> Self::IntoIter {
        CrockfordEncoderIterator {
            byte_stream: self.byte_stream,
            cycle_position: 0,
            buffer: None,
            finished: false,
        }
    }
}

pub trait IntoCrockfordEncoder<I>
where
    I: Iterator<Item = u8>,
{
    fn crockford_encoded(self) -> CrockfordEncoderIterator<I>;
}

impl<I> IntoCrockfordEncoder<I> for I
where
    I: Iterator<Item = u8>,
{
    fn crockford_encoded(self) -> CrockfordEncoderIterator<I> {
        CrockfordEncoder::new(self).into_iter()
    }
}

pub struct CrockfordEncoderIterator<I>
where
    I: Iterator<Item = u8>,
{
    byte_stream: I,
    cycle_position: usize,
    buffer: Option<u8>,
    finished: bool,
}

impl<I> CrockfordEncoderIterator<I>
where
    I: Iterator<Item = u8>,
{
    fn get_next(&mut self) -> u8 {
        if let Some(next) = self.byte_stream.next() {
            next
        } else {
            self.finished = true;
            0
        }
    }

    fn pop_buffer(&mut self) -> u8 {
        // this is a dangerous method that we can only call because we're being
        // *very* responsible about keeping the deque populated
        self.buffer.expect("value should always be populated")
    }

    fn push_buffer(&mut self, value: u8) {
        self.buffer = Some(value);
    }
}

impl<I> Iterator for CrockfordEncoderIterator<I>
where
    I: Iterator<Item = u8>,
{
    type Item = char;

    #[allow(clippy::let_and_return)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let value_to_encode = match self.cycle_position {
            0 => {
                let next = if let Some(next) = self.byte_stream.next() {
                    next
                } else {
                    return None;
                };
                let data = (next & 0b11111000) >> 3;
                self.push_buffer(next);
                data
            }
            1 => {
                let prev = self.pop_buffer();
                let next = self.get_next();
                let prev_masked = (prev & 0b00000111) << 2;
                let next_masked = (next & 0b11000000) >> 6;
                let data = prev_masked | next_masked;
                self.push_buffer(next);
                data
            }
            2 => {
                let prev = self.pop_buffer();
                let data = (prev & 0b00111110) >> 1;
                self.push_buffer(prev);
                data
            }
            3 => {
                let prev = self.pop_buffer();
                let next = self.get_next();
                let prev_masked = (prev & 0b00000001) << 4;
                let next_masked = (next & 0b11110000) >> 4;
                let data = prev_masked | next_masked;
                self.push_buffer(next);
                data
            }
            4 => {
                let prev = self.pop_buffer();
                let next = self.get_next();
                let prev_masked = (prev & 0b00001111) << 1;
                let next_masked = (next & 0b10000000) >> 7;
                let data = prev_masked | next_masked;
                self.push_buffer(next);
                data
            }
            5 => {
                let prev = self.pop_buffer();
                let data = (prev & 0b01111100) >> 2;
                self.push_buffer(prev);
                data
            }
            6 => {
                let prev = self.pop_buffer();
                let next = self.get_next();
                let prev_masked = (prev & 0b00000011) << 3;
                let next_masked = (next & 0b11100000) >> 5;
                let data = prev_masked | next_masked;
                self.push_buffer(next);
                data
            }
            7 => {
                let prev = self.pop_buffer();
                let data = prev & 0b000011111;
                data
            }
            _ => unreachable!("Cycle is always modulo 8"),
        };
        let crockford_char = CROCKFORD_ALPHABET[value_to_encode as usize];
        self.cycle_position = (self.cycle_position + 1) % 8;
        Some(crockford_char)
    }
}

#[cfg(test)]
mod tests {
    use crate::{decode, encode, CrockfordDecodeError, CrockfordEncoder};

    #[test]
    fn test_encode() {
        let mut buf = ['?'; 5];
        encode(1000 as u32, &mut buf[1..]);
        assert_eq!(['?', '0', '0', 'Z', '8'], buf);
    }

    #[test]
    fn test_decode() {
        assert_eq!(1000 as u32, decode(&['0', 'Z', '8']).unwrap());
    }

    #[test]
    fn test_invalid_character() {
        assert_eq!(
            CrockfordDecodeError {
                character: '?',
                slice_position: 1
            },
            decode::<u32>(&['0', '?', 'Z', '8']).unwrap_err()
        )
    }

    #[test]
    fn basic_streaming_encoder() {
        let data = [
            0b11111000, 0b00111110, 0b00001111, 0b10000011, 0b11100000, 0b01010000,
        ];
        let mut byte_stream = data.into_iter();
        let encoded: String = CrockfordEncoder::new(&mut byte_stream)
            .into_iter()
            .collect();
        println!("{encoded}");
    }
}
