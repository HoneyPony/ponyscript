pub fn is_whitespace(byte: Option<u8>) -> bool {
    let byte = byte.unwrap_or(b'0');
    return byte == b' ' || byte == b'\n' || byte == b'\t' || byte == b'\r';
}

pub fn is_alpha(byte: Option<u8>) -> bool {
    let byte = byte.unwrap_or(b'/');

    let lower = byte >= b'a' && byte <= b'z';
    let upper = byte >= b'A' && byte <= b'Z';

    return lower || upper;
}

pub fn is_num(byte: Option<u8>) -> bool {
    let byte = byte.unwrap_or(b'/');
    return byte >= b'0' && byte <= b'9';
}

pub fn is_alphanum(byte: Option<u8>) -> bool {
    let byte = byte.unwrap_or(b'/');

    let lower = byte >= b'a' && byte <= b'z';
    let upper = byte >= b'A' && byte <= b'Z';
    let num   = byte >= b'0' && byte <= b'9';

    return lower || upper || num;
}