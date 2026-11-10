use purge::text::*;
#[test]
fn composition_selection_is_relative_to_inserted_text_not_the_unicode_prefix() {
    assert_eq!(composition_range("中🙂ab尾", 7..9, 1..1), 8..8);
    assert_eq!(composition_range("中🙂a🙂b尾", 7..13, 1..3), 8..12);
}
#[test]
fn utf16_and_byte_offsets_round_trip_at_character_boundaries() {
    let text = "A中🙂B";
    for (bytes, units) in [(0, 0), (1, 1), (4, 2), (8, 4), (9, 5)] {
        assert_eq!(byte_offset(text, units), bytes);
        assert_eq!(utf16_offset(text, bytes), units);
    }
    assert_eq!(byte_offset(text, 999), text.len());
    assert_eq!(utf16_offset(text, 999), 5);
}
