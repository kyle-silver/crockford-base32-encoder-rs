use crate::crockford_index;

pub trait IntoCrockfordDecoder<I>
where
    I: Iterator<Item = char>,
{
    fn crockford_decoded(self) -> CrockfordDecoder<I>;
}

impl<I> IntoCrockfordDecoder<I> for I
where
    I: Iterator<Item = char>,
{
    fn crockford_decoded(self) -> CrockfordDecoder<I> {
        CrockfordDecoder {
            chars: self,
            cycle_position: 0,
            buffer: None,
            finished: false,
        }
    }
}

pub struct CrockfordDecoder<I>
where
    I: Iterator<Item = char>,
{
    chars: I,
    cycle_position: usize,
    buffer: Option<u8>,
    finished: bool,
}

impl<I> CrockfordDecoder<I>
where
    I: Iterator<Item = char>,
{
    fn get_next(&mut self) -> u8 {
        if let Some(next) = self.chars.next() {
            crockford_index(next).unwrap()
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

impl<I> Iterator for CrockfordDecoder<I>
where
    I: Iterator<Item = char>,
{
    type Item = u8;

    #[allow(clippy::let_and_return)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        // 00000_00011_11111_10000_00001_11111_11000_00000
        let decoded = match self.cycle_position {
            0 => {
                let c1 = if let Some(next) = self.chars.next() {
                    crockford_index(next).unwrap()
                } else {
                    self.finished = true;
                    return None;
                };
                let c1_shifted = c1 << 3;
                let c2 = self.get_next();
                let c2_shifted = (c2 & 0b00011100) >> 2;
                let data = c1_shifted | c2_shifted;
                self.push_buffer(c2);
                data
            }
            1 => {
                let c1 = self.pop_buffer();
                let c2 = self.get_next();
                let c3 = self.get_next();
                let c1 = (c1 & 0b00011) << 6;
                let c2 = c2 << 1;
                self.push_buffer(c3);
                let c3 = (c3 & 0b10000) >> 4;
                let data = c1 | c2 | c3;
                data
            }
            2 => {
                let c1 = self.pop_buffer();
                let c2 = self.get_next();
                self.push_buffer(c2);
                let c1 = (c1 & 0b01111) << 4;
                let c2 = (c2 & 0b11110) >> 1;
                let data = c1 | c2;
                data
            }
            3 => {
                let c1 = self.pop_buffer();
                let c2 = self.get_next();
                let c3 = self.get_next();
                self.push_buffer(c3);
                let c1 = (c1 & 0b00001) << 7;
                let c2 = c2 << 2;
                let c3 = (c3 & 0b11000) >> 3;
                let data = c1 | c2 | c3;
                data
            }
            4 => {
                let c1 = self.pop_buffer();
                let c2 = self.get_next();
                let c1 = (c1 & 0b00111) << 5;
                let data = c1 | c2;
                data
            }
            _ => unreachable!(),
        };
        self.cycle_position = (self.cycle_position + 1) % 5;
        Some(decoded)
    }
}

#[cfg(test)]
mod tests {
    use crate::decoder::IntoCrockfordDecoder;
    use crate::encoder::IntoCrockfordEncoder;

    #[test]
    fn simple_decode() {
        let encoded: String = (0u8..=u8::MAX).crockford_encoded().collect();
        println!("{encoded}");
        let decoded: Vec<u8> = encoded.chars().crockford_decoded().collect();
        println!("{decoded:?}");
        let data: Vec<_> = (0u8..=u8::MAX)
            .crockford_encoded()
            .crockford_decoded()
            .collect();
        for x in data.iter().take(256) {
            println!("{x}");
        }
    }
}
