use anyhow::{anyhow, Result};

const ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

// This function works only for sourcemap VLQ values.
pub fn vlq_decode(base64_str: &str) -> Result<[i32; 4]> {
    if base64_str.is_empty() {
        return Ok([0; 4]);
    }

    let base64_decoded = {
        let mut result: Vec<u8> = vec![];
        for byte in base64_str.chars() {
            result.push(ALPHABET.find(byte).unwrap() as u8);
        }

        result
    };

    let mut vlqs: Vec<Vec<i8>> = vec![];
    let mut current_vlq: usize = 0;
    let mut vlq_sequence_ended = true;

    for raw_value in base64_decoded.iter() {
        if vlq_sequence_ended {
            vlqs.push(vec![]);
            vlq_sequence_ended = false;
        }

        vlqs[current_vlq].push(*raw_value as i8);

        if (raw_value & 0b100000) == 0 {
            // MSB decides whether this octet is the last octet of this number.
            current_vlq += 1;
            vlq_sequence_ended = true;
        }
    }

    if !vlq_sequence_ended {
        return Err(anyhow!("Last VLQ sequence never ended."));
    }

    if vlqs.len() != 4 && vlqs.len() != 5 {
        return Err(anyhow!(
            "Either 4 or 5 VLQ values should be present, {} values found. Base64 value: {base64_str}",
            vlqs.len()
        ));
    }

    let mut result = [0i32; 4];

    for (index, vlq) in vlqs.iter().take(4).enumerate() {
        let mut value = 0i32;
        let mut negative = false;

        for (index, vlq_val) in vlq.into_iter().enumerate().rev() {
            let mut vlq_value = *vlq_val as i32;
            if index == 0 {
                // First value in VLQ sequence decides whether end number is positive or negative.
                negative = (vlq_value & 1) == 1; // Number is negative if LSB is 1.
                vlq_value >>= 1;
                value <<= 4;
                value |= vlq_value & 0b1111;
            } else {
                value <<= 5;
                value |= vlq_value & 0b11111;
            }
        }

        result[index] = if negative { -value } else { value };
    }

    Ok(result)
}

#[cfg(any(test, rust_analyzer))]
mod tests {
    use crate::vlq::vlq_decode;

    #[test]
    fn example() {
        #[rustfmt::skip]
        let values = [
            "AA2CA",
            "MAAK",
            "YAAa",
            "gBAAa",
            "AAAA",
            "EAC3B",
            "MAAM",
            "AAAA",
            "EACN",
            "YAAY",
            "EAAE",
            "iBAAiB",
            "aAAc",
            "AAAA",
            "EAC7C",
            "OAAO",
        ];

        let expected = [
            [0i32, 0, 43, 0],
            [6, 0, 0, 5],
            [12, 0, 0, 13],
            [16, 0, 0, 13],
            [0, 0, 0, 0],
            [2, 0, 1, -27],
            [6, 0, 0, 6],
            [0, 0, 0, 0],
            [2, 0, 1, -6],
            [12, 0, 0, 12],
            [2, 0, 0, 2],
            [17, 0, 0, 17],
            [13, 0, 0, 14],
            [0, 0, 0, 0],
            [2, 0, 1, -45],
            [7, 0, 0, 7],
        ];

        for (index, value) in values.iter().enumerate() {
            assert_eq!(vlq_decode(value).unwrap(), expected[index]);
        }
    }
}
