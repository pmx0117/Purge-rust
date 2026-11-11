use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

/// Fit one immutable label using complete graphemes. Every measurement starts
/// from the original text; a narrow pass cannot corrupt a later wider pass.
pub fn fit_label(
    text: &str,
    width: f32,
    middle: bool,
    mut measure: impl FnMut(&str) -> f32,
) -> String {
    if width <= 0. || width.is_nan() {
        return String::new();
    }
    if measure(text) <= width {
        return text.to_owned();
    }
    if measure("…") > width {
        return String::new();
    }
    let boundaries: Vec<_> = text
        .grapheme_indices(true)
        .map(|(index, _)| index)
        .chain(std::iter::once(text.len()))
        .collect();
    let count = boundaries.len().saturating_sub(1);
    let candidate = |keep: usize| {
        if middle {
            format!(
                "{}…{}",
                &text[..boundaries[keep.div_ceil(2)]],
                &text[boundaries[count - keep / 2]..]
            )
        } else {
            format!("{}…", &text[..boundaries[keep]])
        }
    };
    let (mut low, mut high) = (0, count.saturating_sub(1));
    while low < high {
        let keep = (low + high).div_ceil(2);
        if measure(&candidate(keep)) <= width {
            low = keep;
        } else {
            high = keep - 1;
        }
    }
    candidate(low)
}
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
