use tic80_rust::editor::code::CodeBuffer;

#[test]
fn word_left_right_basic() {
    let mut cb = CodeBuffer::from_text("foo bar_baz  qux\n");
    cb.caret_line = 0; cb.caret_col = 11; // before two spaces
    cb.word_left();
    assert_eq!((cb.caret_line, cb.caret_col), (0, 4)); // start of bar_baz
    cb.word_left();
    assert_eq!((cb.caret_line, cb.caret_col), (0, 0)); // start of foo

    cb.caret_col = 0;
    cb.word_right();
    assert_eq!(cb.caret_col, 3); // after foo
    cb.word_right();
    assert_eq!(cb.caret_col, 4 + 7); // at end of bar_baz
}

#[test]
fn delete_word_left_right() {
    let mut cb = CodeBuffer::from_text("  foo, bar\n");
    cb.caret_line = 0; cb.caret_col = 10; // end
    cb.delete_word_left(); // delete 'bar'
    assert_eq!(cb.as_string(), "  foo, \n");
    cb.delete_word_left(); // delete punctuation and space
    assert_eq!(cb.as_string(), "  foo\n");

    let mut cb2 = CodeBuffer::from_text("foo  bar\n");
    cb2.caret_line = 0; cb2.caret_col = 0;
    cb2.delete_word_right(); // delete 'foo'
    assert_eq!(cb2.as_string(), "  bar\n");
}
