pub fn unescape(s: String) -> String {
    let mut chars = s.into_bytes();
    let mut len = chars.len();
    let mut i = 0;
    let mut j = 0;

    while i < len {
        if chars[j] == b'\\' {
            chars[i] = match chars[j + 1] {
                b'n' => b'\n',
                b'r' => b'\r',
                b't' => b'\t',
                c => c,
            };
            j += 2;
            len -= 1;
        } else {
            if i != j {
                chars[i] = chars[j];
            }
            j += 1;
        }

        i += 1;
    }

    chars.truncate(len);
    String::from_utf8(chars).unwrap()
}
