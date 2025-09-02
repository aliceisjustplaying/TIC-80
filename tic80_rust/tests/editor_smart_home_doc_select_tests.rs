use tic80_rust::editor::code::CodeBuffer;

#[test]
fn smart_home_toggle_and_shift() {
    let mut cb = CodeBuffer::from_text("    indented\n");
    cb.caret_line = 0; cb.caret_col = 8;
    cb.smart_home(false);
    assert_eq!(cb.caret_col, 4);
    cb.smart_home(false);
    assert_eq!(cb.caret_col, 0);
    cb.caret_col = 6;
    cb.ensure_selection_anchor();
    cb.smart_home(true);
    assert_eq!(cb.caret_col, 4);
    assert!(cb.has_selection());
}

#[test]
fn doc_select_to_bounds() {
    let mut cb = CodeBuffer::from_text("a\nb\nc\n");
    cb.caret_line = 1; cb.caret_col = 1;
    cb.ensure_selection_anchor();
    cb.doc_home();
    let (s, _e) = cb.selection_range_idx().unwrap();
    assert_eq!(cb.rope().char_to_line(s), 0);

    let mut cb2 = CodeBuffer::from_text("a\nbc\n");
    cb2.caret_line = 0; cb2.caret_col = 1;
    cb2.ensure_selection_anchor();
    cb2.doc_end();
    let (_s, e) = cb2.selection_range_idx().unwrap();
    assert_eq!(cb2.rope().char_to_line(e), cb2.line_count() - 1);
}
