
use tic80_rust::editor::highlight as hl;

#[test]
fn lex_keywords_ident_numbers() {
    let line = "local x = 42";
    let tok = hl::lex_line(line, hl::State::Normal);
    assert!(tok.runs.iter().any(|r| matches!(r, (0, 5, hl::Kind::Keyword))));
    assert!(tok.runs.iter().any(|r| matches!(r, (6, 7, hl::Kind::Identifier))));
    assert!(tok.runs.iter().any(|r| matches!(r, (10, 12, hl::Kind::Number))));
}

#[test]
fn lex_strings_and_comments() {
    let line = "print(\"hi\") -- ok";
    let tok = hl::lex_line(line, hl::State::Normal);
    assert!(tok.runs.iter().any(|(_,_,k)| matches!(k, hl::Kind::StringShort)));
    assert!(tok.runs.iter().any(|(_,_,k)| matches!(k, hl::Kind::CommentLine)));
}
