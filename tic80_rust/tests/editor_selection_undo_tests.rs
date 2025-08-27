use tic80_rust::editor::code::CodeBuffer;

#[test]
fn replace_selection_and_undo_redo() {
    let mut cb = CodeBuffer::from_text("abc\ndef");
    eprintln!("R0: {:?}", cb.as_string());
    // Select "bc" on first line
    cb.caret_line = 0;
    cb.caret_col = 1;
    cb.start_selection();
    cb.caret_col = 3; // selection [1,3)
                      // Cut, then paste "Z"
    let cut = cb.cut_selection_text().unwrap();
    eprintln!("R1: cut={:?} now={:?}", cut, cb.as_string());
    assert_eq!(cut, "bc");
    assert_eq!(cb.as_string(), "a\ndef");
    cb.paste_text("Z");
    eprintln!("R2: {:?}", cb.as_string());
    assert_eq!(cb.as_string(), "aZ\ndef");
    // Undo paste+cut (batch), redo
    cb.undo();
    eprintln!("R3: {:?}", cb.as_string());
    assert_eq!(cb.as_string(), "a\ndef");
    cb.redo();
    eprintln!("R4: {:?}", cb.as_string());
    assert_eq!(cb.as_string(), "aZ\ndef");
}

#[test]
fn undo_redo_simple_backspace() {
    let mut cb = CodeBuffer::from_text("ab");
    eprintln!("T0: {:?}", cb.as_string());
    cb.caret_line = 0;
    cb.caret_col = 2; // end
    cb.backspace();
    eprintln!("T1: {:?}", cb.as_string());
    assert_eq!(cb.as_string(), "a");
    cb.undo();
    eprintln!("T2: {:?}", cb.as_string());
    assert_eq!(cb.as_string(), "ab");
    cb.redo();
    eprintln!("T3: {:?}", cb.as_string());
    assert_eq!(cb.as_string(), "a");
}

#[test]
fn debug_backspace_print() {
    let mut cb = CodeBuffer::from_text("ab");
    eprintln!("BEFORE: {:?}", cb.as_string());
    cb.caret_line = 0;
    cb.caret_col = 2;
    cb.backspace();
    eprintln!("AFTER: {:?}", cb.as_string());
}

#[test]
fn select_all_and_cut_to_empty() {
    let mut cb = CodeBuffer::from_text("hello\nworld");
    cb.select_all();
    let cut = cb.cut_selection_text().unwrap();
    assert_eq!(cut, "hello\nworld");
    assert_eq!(cb.as_string(), "");
}

#[test]
fn paste_simple_newline_context() {
    let mut cb = CodeBuffer::from_text("a\nb");
    cb.caret_line = 0;
    cb.caret_col = 1;
    cb.paste_text("Z");
    assert_eq!(cb.as_string(), "aZ\nb");
}
