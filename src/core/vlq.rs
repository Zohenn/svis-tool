use anyhow::{anyhow, Result};

const ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/***
* This function works only for sourcemap VLQ values (probably).
*/
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
    let mut current_vlq: Vec<i8> = vec![];

    for raw_value in base64_decoded.iter() {
        current_vlq.push(*raw_value as i8);
        if (raw_value & 0b100000) == 0 {
            vlqs.push(current_vlq.clone());
            current_vlq.clear();
        }
    }

    if !current_vlq.is_empty() {
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
                negative = (vlq_value & 1) == 1;
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

#[cfg(test)]
mod tests {
    use crate::core::vlq::vlq_decode;

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
