// Some parts of this code picked up from:
// https://github.com/emreyaren/zero85/

pub fn encode(input: &[u8]) -> Result<Vec<u8>, String> {
    if input.is_empty() {
        return Err("No input provided.".to_string());
    }
    let mut input = input.to_vec();

    let mut input_len = input.len();
    let mut bytes_added = 0;
    while input_len % 4 != 0 {
        input.push(b'\0');
        input_len = input.len();
        bytes_added += 1;
    }
    let mut out_vec: Vec<u8> = Vec::new();
    for in_chunk in input.chunks(4) {
        let mut block_num: u32 = 0;
        for byte in in_chunk {
            block_num = block_num * 256 + u32::from(*byte);
        }
        if block_num == 0 {
            out_vec.push(b'z');
        } else {
            let mut out_chunk = [0_u8; 5];
            for c in &mut out_chunk[..] {
                *c = (block_num % 85) as u8 + 33;
                block_num /= 85;
            }
            out_chunk.reverse();
            out_vec.extend_from_slice(&out_chunk);
        }
    }
    out_vec.truncate(out_vec.len() - bytes_added);
    Ok(out_vec)
}

#[allow(dead_code)]
pub fn decode(bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    let mut len = bytes.len();
    let mut bytes = bytes.to_vec();
    let mut bytes_added = 0;
    while len % 5 != 0 {
        bytes.push(b'u'); // pad last batch of chars with u's to 5 bytes
        len = bytes.len();
        bytes_added += 1;
    }
    let mut out_vec: Vec<u8> = Vec::new();
    let mut pos: usize = 0;
    while pos < len {
        let mut block_num: u32 = 0;
        for c in &bytes[pos..(pos + 5)] {
            if b'!' <= *c && *c <= b'u' {
                let ch = u32::from(*c) - 33;
                block_num = block_num * 85 + ch;
            } else if *c == b'z' {
                block_num = 0;
                break;
            } else {
                return Err(format!("Error at pos: {} of {} (char '{}')", pos, len, *c));
            }
        }
        let mut out_chunk = [0_u8; 4];
        for b in &mut out_chunk[..] {
            *b = (block_num % 256) as u8;
            block_num /= 256;
        }
        out_chunk.reverse();
        out_vec.extend_from_slice(&out_chunk);
        pos += 5;
    }
    let len = out_vec.len();
    out_vec.truncate(len - bytes_added); // remove bytes added from output
    Ok(out_vec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_encode() {
        let mut bytes = Vec::new();
        let input = "Man is distinguished, not only by his reason, but by this singular passion from other animals, which is a lust of the mind, that by a perseverance of delight in the continued and indefatigable generation of knowledge, exceeds the short vehemence of any carnal pleasure.";
        write!(bytes, "{}", input).unwrap();
        let result = encode(&bytes[..]).unwrap();
        let mut expected_vec = Vec::new();
        let expected = r#"9jqo^BlbD-BleB1DJ+*+F(f,q/0JhKF<GL>Cj@.4Gp$d7F!,L7@<6@)/0JDEF<G%<+EV:2F!,O<DJ+*.@<*K0@<6L(Df-\0Ec5e;DffZ(EZee.Bl.9pF"AGXBPCsi+DGm>@3BB/F*&OCAfu2/AKYi(DIb:@FD,*)+C]U=@3BN#EcYf8ATD3s@q?d$AftVqCh[NqF<G:8+EV:.+Cf>-FD5W8ARlolDIal(DId<j@<?3r@:F%a+D58'ATD4$Bl@l3De:,-DJs`8ARoFb/0JMK@qB4^F!,R<AKZ&-DfTqBG%G>uD.RTpAKYo'+CT/5+Cei#DII?(E,9)oF*2M7/c"#;
        write!(expected_vec, "{}", expected).unwrap();
        assert_eq!(result, expected_vec);
    }

    #[test]
    fn test_string_encode() {
        let bytes = String::from("Tämä on testi.").into_bytes();
        let result = encode(&bytes[..]).unwrap();
        let expected: Vec<u8> = Vec::from(r#"<5YMK_k\DnDBO%4F*)+K"#);
        assert_eq!(result, expected)
    }

    #[test]
    fn test_decode() {
        let mut bytes = Vec::new();
        let raw = r#"9jqo^BlbD-BleB1DJ+*+F(f,q/0JhKF<GL>Cj@.4Gp$d7F!,L7@<6@)/0JDEF<G%<+EV:2F!,O<DJ+*.@<*K0@<6L(Df-\0Ec5e;DffZ(EZee.Bl.9pF"AGXBPCsi+DGm>@3BB/F*&OCAfu2/AKYi(DIb:@FD,*)+C]U=@3BN#EcYf8ATD3s@q?d$AftVqCh[NqF<G:8+EV:.+Cf>-FD5W8ARlolDIal(DId<j@<?3r@:F%a+D58'ATD4$Bl@l3De:,-DJs`8ARoFb/0JMK@qB4^F!,R<AKZ&-DfTqBG%G>uD.RTpAKYo'+CT/5+Cei#DII?(E,9)oF*2M7/c"#;
        write!(bytes, "{}", raw).unwrap();
        let result = decode(bytes).unwrap();
        assert_eq!(
            String::from_utf8(result).unwrap(),
            String::from("Man is distinguished, not only by his reason, but by this singular passion from other animals, which is a lust of the mind, that by a perseverance of delight in the continued and indefatigable generation of knowledge, exceeds the short vehemence of any carnal pleasure.")
        );
    }

    #[test]
    fn test_string_decode() {
        let bytes: Vec<u8> = Vec::from(r#"<5YMK_k\DnDBO%4F*)+K"#);
        let result = decode(bytes).unwrap();
        let string_bytes = String::from("Tämä on testi.").into_bytes();
        assert_eq!(result, string_bytes)
    }
}
