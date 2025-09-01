use tic80_rust::editor::code::CodeBuffer;

fn make_lines(n: usize) -> String {
    let mut s = String::new();
    for i in 0..n { let _ = i; s.push_str("x\n"); }
    s
}

#[test]
fn page_down_moves_by_visible_lines() {
    let text = make_lines(50);
    let mut cb = CodeBuffer::from_text(&text);
    cb.caret_line = 0;
    let vis = 10usize;
    cb.page_down(vis);
    assert_eq!(cb.caret_line, 10);
    cb.page_down(vis);
    assert_eq!(cb.caret_line, 20);
}

#[test]
fn page_up_clamps_at_top() {
    let text = make_lines(5);
    let mut cb = CodeBuffer::from_text(&text);
    cb.caret_line = 2;
    cb.page_up(10);
    assert_eq!(cb.caret_line, 0);
}

#[test]
fn shift_page_extends_selection() {
    let text = make_lines(40);
    let mut cb = CodeBuffer::from_text(&text);
    cb.caret_line = 5;
    cb.caret_col = 1;
    cb.ensure_selection_anchor();
    cb.page_down(10);
    // Expect selection spanning from original caret line to new caret line
    let (s, e) = cb.selection_range_idx().expect("selection after shift+page");
    let s_line = cb.rope().char_to_line(s);
    let e_line = cb.rope().char_to_line(e - 1);
    assert_eq!(s_line, 5);
    assert_eq!(e_line, 15);
}

