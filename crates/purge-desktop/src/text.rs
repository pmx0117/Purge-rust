use std::ops::Range;
pub fn byte_offset(text: &str, utf16: usize) -> usize {
    convert_offset(text, utf16, char::len_utf16, char::len_utf8)
}
pub fn utf16_offset(text: &str, byte: usize) -> usize {
    convert_offset(text, byte, char::len_utf8, char::len_utf16)
}
fn convert_offset(
    text: &str,
    offset: usize,
    source: fn(char) -> usize,
    target: fn(char) -> usize,
) -> usize {
    let (mut consumed, mut result) = (0, 0);
    for character in text.chars() {
        if consumed >= offset {
            break;
        }
        consumed += source(character);
        result += target(character);
    }
    result
}
pub fn composition_range(
    content: &str,
    inserted: Range<usize>,
    relative_utf16: Range<usize>,
) -> Range<usize> {
    byte_offset(&content[inserted.clone()], relative_utf16.start) + inserted.start
        ..byte_offset(&content[inserted.clone()], relative_utf16.end) + inserted.start
}
