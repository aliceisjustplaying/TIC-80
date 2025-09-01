use tic80_rust::editor::code::CodeBuffer;

#[test]
fn block_indent_and_outdent_selection() {
    let text = "one\ntwo\nthree\n";
    let mut cb = CodeBuffer::from_text(text);
    // Select from start of line 0 to end of line 2 (inclusive)
    cb.caret_line = 0; cb.caret_col = 0; cb.ensure_selection_anchor();
    cb.caret_line = 2; cb.caret_col = cb.line_len(2);

    // Indent selection
    cb.block_indent();
    assert!(cb.rope().line(0).to_string().starts_with(" "));
    assert!(cb.rope().line(1).to_string().starts_with(" "));
    assert!(cb.rope().line(2).to_string().starts_with(" "));

    // Outdent selection
    cb.block_outdent();
    assert!(!cb.rope().line(0).to_string().starts_with(" "));
    assert!(!cb.rope().line(1).to_string().starts_with(" "));
    assert!(!cb.rope().line(2).to_string().starts_with(" "));
}
