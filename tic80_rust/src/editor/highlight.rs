
#![allow(
    clippy::cognitive_complexity,
    clippy::single_match_else,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::derivable_impls,
    clippy::use_self,
    unused_assignments,
    unused_mut
)]

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    Whitespace,
    Identifier,
    Keyword,
    Number,
    StringShort,
    StringLong,
    CommentLine,
    CommentBlock,
    Api,
    Sign,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    Normal,
    InLongString { level: u32 },
    InBlockComment { level: u32 },
}

impl Default for State { fn default() -> Self { Self::Normal } }

#[derive(Clone, Debug, Default)]
pub struct LineTok {
    pub runs: Vec<(usize, usize, Kind)>, // [start,end) in columns
    pub state_out: State,
}

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub bg: u8,
    pub fg: u8,
    pub string: u8,
    pub number: u8,
    pub keyword: u8,
    pub api: u8,
    pub comment: u8,
    pub sign: u8,
}

#[must_use]
pub const fn default_theme() -> Theme {
    Theme { bg: 15, fg: 12, string: 4, number: 11, keyword: 3, api: 5, comment: 14, sign: 13 }
}

const fn is_ident_start(c: char) -> bool { c == '_' || c.is_ascii_alphabetic() }
const fn is_ident_continue(c: char) -> bool { c == '_' || c.is_ascii_alphanumeric() }

fn is_keyword(s: &str) -> bool {
    matches!(s,
        "and"|"break"|"do"|"else"|"elseif"|"end"|"false"|"for"|"function"|
        "goto"|"if"|"in"|"local"|"nil"|"not"|"or"|"repeat"|"return"|
        "then"|"true"|"until"|"while")
}

fn is_api(s: &str) -> bool {
    matches!(s,
        "cls"|"pix"|"line"|"rect"|"rectb"|"circ"|"circb"|"elli"|"ellib"|
        "tri"|"trib"|"clip"|"print"|"peek"|"poke"|"memcpy"|"memset"|
        "fft"|"ffts"|"fftr"|"fftrs"|"vqt"|"vqts"|"vqtr"|"vqtrs")
}

#[must_use]
pub fn lex_line(text: &str, mut state: State) -> LineTok {
    let mut out = LineTok::default();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0usize;
    // Handle continuing states
    match state {
        State::InLongString { level } => {
            // search for ]=...=]
            while i < chars.len() {
                if chars[i] == ']' {
                    // check = level then ]
                    let mut j = 0; while i + 1 + j < chars.len() && j < level as usize && chars[i+1 + j] == '=' { j += 1; }
                    if j == level as usize && i + 1 + j < chars.len() && chars[i+1 + j] == ']' {
                        // include closing delimiter
                        out.runs.push((0, i + 2 + j, Kind::StringLong));
                        i += 2 + j; state = State::Normal; break;
                    }
                }
                i += 1;
            }
            if matches!(state, State::InLongString { .. }) { out.runs.push((0, chars.len(), Kind::StringLong)); out.state_out = state; return out; }
        }
        State::InBlockComment { level } => {
            while i < chars.len() {
                if chars[i] == ']' {
                    let mut j = 0; while i + 1 + j < chars.len() && j < level as usize && chars[i+1 + j] == '=' { j += 1; }
                    if j == level as usize && i + 1 + j < chars.len() && chars[i+1 + j] == ']' { out.runs.push((0, i + 2 + j, Kind::CommentBlock)); i += 2 + j; state = State::Normal; break; }
                }
                i += 1;
            }
            if matches!(state, State::InBlockComment { .. }) { out.runs.push((0, chars.len(), Kind::CommentBlock)); out.state_out = state; return out; }
        }
        State::Normal => {}
    }

    // Normal scanning
    i = 0;
    let push_run = |start: usize, end: usize, kind: Kind, out: &mut LineTok| { if end > start { out.runs.push((start, end, kind)); } };
    while i < chars.len() {
        let c = chars[i];
        // whitespace
        if c.is_whitespace() { let start = i; while i < chars.len() && chars[i].is_whitespace() { i += 1; } push_run(start, i, Kind::Whitespace, &mut out); continue; }
        // comment
        if c == '-' && i + 1 < chars.len() && chars[i+1] == '-' {
            // block comment?
            if i + 3 < chars.len() && chars[i+2] == '[' {
                let mut j = i + 3; let mut level = 0u32; while j < chars.len() && chars[j] == '=' { level += 1; j += 1; }
                if j < chars.len() && chars[j] == '[' {
                    let mut k = j + 1; let mut closed = None;
                    while k < chars.len() { if chars[k] == ']' {
                        let mut eq = 0usize; while k + 1 + eq < chars.len() && eq < level as usize && chars[k+1+eq] == '=' { eq += 1; }
                        if eq == level as usize && k + 1 + eq < chars.len() && chars[k+1+eq] == ']' { closed = Some(k + 2 + eq); break; }
                    } k += 1; }
                    match closed { Some(end) => { push_run(i, end, Kind::CommentBlock, &mut out); i = end; continue; }
                                   None => { push_run(i, chars.len(), Kind::CommentBlock, &mut out); out.state_out = State::InBlockComment { level }; return out; } }
                }
            }
            push_run(i, chars.len(), Kind::CommentLine, &mut out); i = chars.len(); break;
        }
        // short strings
        if c == '"' || c == '\'' {
            let quote = c; let start = i; i += 1;
            while i < chars.len() {
                if chars[i] == '\\' { i += 2; continue; }
                if chars[i] == quote { i += 1; break; }
                i += 1;
            }
            push_run(start, i, Kind::StringShort, &mut out); continue;
        }
        // long string
        if c == '[' { let mut j = i + 1; let mut level = 0u32; while j < chars.len() && chars[j] == '=' { level += 1; j += 1; } if j < chars.len() && chars[j] == '[' {
            let start = i; let mut k = j + 1; let mut closed = None; while k < chars.len() { if chars[k] == ']' {
                let mut eq = 0usize; while k + 1 + eq < chars.len() && eq < level as usize && chars[k+1+eq] == '=' { eq += 1; }
                if eq == level as usize && k + 1 + eq < chars.len() && chars[k+1+eq] == ']' { closed = Some(k + 2 + eq); break; }
            } k += 1; } match closed { Some(end) => { push_run(start, end, Kind::StringLong, &mut out); i = end; continue; } None => { push_run(start, chars.len(), Kind::StringLong, &mut out); out.state_out = State::InLongString { level }; return out; } } } }
        // number
        if c.is_ascii_digit() { let start = i; i += 1; while i < chars.len() && (chars[i].is_ascii_hexdigit() || matches!(chars[i], '.'|'x'|'X'|'e'|'E'|'+'|'-'|'_')) { i += 1; } push_run(start, i, Kind::Number, &mut out); continue; }
        // identifier / keyword / api
        if is_ident_start(c) { let start = i; i += 1; while i < chars.len() && is_ident_continue(chars[i]) { i += 1; } let s: String = chars[start..i].iter().copied().collect(); let kind = if is_keyword(&s) { Kind::Keyword } else if is_api(&s) { Kind::Api } else { Kind::Identifier }; push_run(start, i, kind, &mut out); continue; }
        // sign
        push_run(i, i+1, Kind::Sign, &mut out); i += 1;
    }
    out.state_out = state; out
}

#[must_use]
pub const fn color_for(kind: Kind, th: Theme) -> u8 {
    match kind {
        Kind::Whitespace | Kind::Identifier => th.fg,
        Kind::Keyword => th.keyword,
        Kind::Number => th.number,
        Kind::StringShort | Kind::StringLong => th.string,
        Kind::CommentLine | Kind::CommentBlock => th.comment,
        Kind::Api => th.api,
        Kind::Sign => th.sign,
    }
}
