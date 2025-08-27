use tic80_rust::editor::code::CodeBuffer;

#[test]
fn insert_chars_and_newlines() {
    let mut cb = CodeBuffer::from_text("");
    cb.insert_char('a');
    cb.insert_char('b');
    cb.insert_newline();
    cb.insert_char('c');
    cb.insert_char('d');
    assert_eq!(cb.as_string(), "ab\ncd");
}

#[test]
fn backspace_within_line_and_join_previous() {
    let mut cb = CodeBuffer::from_text("hello\nworld");
    // Place caret after 'l' in first line
    cb.caret_line = 0;
    cb.caret_col = 3; // hel|lo
    cb.backspace(); // remove 'l'
    assert_eq!(cb.as_string(), "helo\nworld");
    // Move to start of second line and backspace -> join
    cb.caret_line = 1;
    cb.caret_col = 0;
    cb.backspace();
    assert_eq!(cb.as_string(), "heloworld");
    assert_eq!(cb.caret_line, 0);
}

#[test]
fn delete_forward_and_join_next_line() {
    let mut cb = CodeBuffer::from_text("xy\nzz");
    // place at end of first line and delete -> join lines
    cb.caret_line = 0;
    cb.caret_col = 2; // xy|\nzz
    cb.delete_forward();
    assert_eq!(cb.as_string(), "xyzz");
    // delete in middle
    cb.caret_line = 0;
    cb.caret_col = 1; // x|yzz
    cb.delete_forward();
    assert_eq!(cb.as_string(), "xzz");
}

#[test]
fn home_and_end() {
    let mut cb = CodeBuffer::from_text("abc\ndef");
    cb.caret_line = 1;
    cb.caret_col = 1;
    cb.home();
    assert_eq!(cb.caret_col, 0);
    cb.end();
    assert_eq!(cb.caret_col, 3);
}

#[test]
fn insert_tab_inserts_two_spaces() {
    let mut cb = CodeBuffer::from_text("");
    cb.insert_tab();
    cb.insert_char('x');
    assert_eq!(cb.as_string(), " x");
}
