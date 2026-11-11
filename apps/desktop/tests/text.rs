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

#[test]
fn mixed_unicode_labels_fit_without_losing_boundaries_or_reusing_a_narrow_pass() {
    use unicode_segmentation::UnicodeSegmentation;
    let measure = |value: &str| value.graphemes(true).count() as f32;
    let text = "AB远程控制器🙂正式版";
    assert_eq!(fit_label(text, 6., true, measure), "AB远…式版");
    assert_eq!(fit_label(text, 6., false, measure), "AB远程控…");
    assert_eq!(fit_label(text, 1., true, measure), "…");
    assert_eq!(fit_label(text, 100., true, measure), text);
    assert_eq!(fit_label(text, 0., true, measure), "");
    assert_eq!(
        fit_label("A👩‍💻B e\u{301}C", 5., true, measure),
        "A👩‍💻…e\u{301}C"
    );
}
