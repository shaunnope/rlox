
/// Checks if the given char is valid as an identifier's start character.
#[inline]
pub fn is_valid_identifier_start(c: char) -> bool {
  c.is_ascii_alphabetic() || c == '_'
}

/// Checks if the given char can belong to an identifier's tail.
#[inline]
pub fn is_valid_identifier_tail(c: char) -> bool {
  c.is_ascii_digit() || is_valid_identifier_start(c)
}
