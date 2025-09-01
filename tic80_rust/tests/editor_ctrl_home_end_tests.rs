use tic80_rust::editor::code::CodeBuffer;

#[test]
fn ctrl_home_end_move_to_bounds() {
    let text = "abc\ndef\nlast";
    let mut cb = CodeBuffer::from_text(text);
    cb.caret_line = 1;
    cb.caret_col = 2;
    cb.doc_home();
    assert_eq!((cb.caret_line, cb.caret_col), (0, 0));
    cb.doc_end();
    assert_eq!(cb.caret_line, cb.line_count() - 1);
    assert_eq!(cb.caret_col, cb.line_len(cb.caret_line));
}

