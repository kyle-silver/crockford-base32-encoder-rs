use num::{PrimInt, Unsigned};

mod decoder;
mod encoder;

pub const CROCKFORD_ALPHABET: [char; 32] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J',
    'K', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'V', 'W', 'X', 'Y', 'Z',
];

pub(crate) fn crockford_index(c: char) -> Option<u8> {
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

#[cfg(test)]
mod tests {
    use crate::encoder::IntoCrockfordEncoder;
    use crate::{decode, encode, CrockfordDecodeError};

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
        let data = [0b11111000, 0b00111110, 0b00001111, 0b10000011, 0b11100000];
        let encoded: String = data.iter().cloned().crockford_encoded().collect();
        assert_eq!("Z0Z0Z0Z0", encoded);
    }
}
