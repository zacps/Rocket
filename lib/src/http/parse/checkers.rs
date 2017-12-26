#[inline(always)]
pub fn is_whitespace(byte: char) -> bool {
    byte == ' ' || byte == '\t'
}

#[inline(always)]
pub fn is_digit(byte: char) -> bool {
    match byte {
        '0'...'9' => true,
        _ => false
    }
}

#[inline]
pub fn is_valid_token(c: char) -> bool {
    match c {
        '0'...'9' | 'A'...'Z' | '^'...'~' | '#'...'\''
            | '!' | '*' | '+' | '-' | '.'  => true,
        _ => false
    }
}
