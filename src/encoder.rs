use crate::CROCKFORD_ALPHABET;

pub trait IntoCrockfordEncoder<'data, I>
where
    I: Iterator<Item = &'data u8>,
{
    fn crockford_encoded(self) -> CrockfordEncoder<'data, I>;
}

impl<'data, I> IntoCrockfordEncoder<'data, I> for I
where
    I: Iterator<Item = &'data u8>,
{
    fn crockford_encoded(self) -> CrockfordEncoder<'data, I> {
        CrockfordEncoder {
            byte_stream: self,
            cycle_position: 0,
            buffer: None,
            finished: false,
        }
    }
}

pub struct CrockfordEncoder<'data, I>
where
    I: Iterator<Item = &'data u8>,
{
    byte_stream: I,
    cycle_position: usize,
    buffer: Option<u8>,
    finished: bool,
}

impl<'data, I> CrockfordEncoder<'data, I>
where
    I: Iterator<Item = &'data u8>,
{
    fn get_next(&mut self) -> u8 {
        if let Some(&next) = self.byte_stream.next() {
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

impl<'data, I> Iterator for CrockfordEncoder<'data, I>
where
    I: Iterator<Item = &'data u8>,
{
    type Item = char;

    #[allow(clippy::let_and_return)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let value_to_encode = match self.cycle_position {
            0 => {
                let next = if let Some(&next) = self.byte_stream.next() {
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
