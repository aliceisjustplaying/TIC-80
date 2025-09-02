
# Editor Syntax Highlighting — Plan & TODOs (CODE View)

Status: planning (tests-first, incremental)

This document is the implementation plan and checklist for adding syntax highlighting to the CODE view with TIC‑80 parity in spirit and practical constraints of the 240×136 framebuffer. We will implement Lua first (TIC‑80 default), use CODE_THEME colors, and integrate with the existing renderer, selection, and caret.

## Goals

- Lua (5.3) highlighting with a pragmatic tokenizer:
  - Keywords, identifiers, numbers, strings (short/long), comments (line/block), signs/punctuation, API names.
  - Multiline constructs (long strings/comments) carry state across lines.
- Theme‑driven colors from CODE_THEME:
  - `BG`, `FG`, `STRING`, `NUMBER`, `KEYWORD`, `API`, `COMMENT`, `SIGN`, `SELECT`, `CURSOR`.
- Low overhead: tokenize only visible lines (plus minimal context) with per‑line state cache and dirty invalidation on edits.
- Selection/caret precedence: keep current selection overlay and caret behavior; highlighting is suppressed when selection inverts glyphs.

## Token Kinds and Color Map

- `Whitespace` (no color change; use previous).
- `Identifier` → `FG` (unless API or keyword).
- `Keyword` (Lua 5.3 set): `and, break, do, else, elseif, end, false, for, function, goto, if, in, local, nil, not, or, repeat, return, then, true, until, while` → `KEYWORD`.
- `Number` → `NUMBER` (support decimal, hex `0x..`, floats with exponent; pragmatic subset).
- `StringShort` (single/double quoted with escapes) → `STRING`.
- `StringLong` (Lua long brackets `[[ .. ]]`, `[=[ .. ]=]` nesting level) → `STRING`.
- `CommentLine` (`-- ...`) → `COMMENT`.
- `CommentBlock` (`--[[ .. ]]`, equal‑sign variants) → `COMMENT`.
- `API` (known TIC‑80 API names: `cls, pix, line, rect, rectb, circ, circb, elli, ellib, tri, trib, clip, print, peek, poke, memcpy, memset, fft*, vqt*` …) → `API`.
- `Sign` (operators, punctuation) → `SIGN`.

## Architecture

- Module: `editor/highlight.rs`
  - `struct LineTok { runs: Vec<(col_start, col_end, Kind)>; state_out: State }`
  - `enum State { Normal, InLongString { level: u32 }, InBlockComment { level: u32 } }`
  - `fn lex_line(text: &str, state_in: State) -> LineTok`
  - Color resolver: `fn color_for(kind: Kind, theme: &Theme) -> u8`

- Cache and invalidation
  - Store per‑line `State` and compact token runs in `CodeBuffer` (or sibling `HighlighterCache`).
  - On edit: mark changed line..end as dirty; recompute forward until state stabilizes (no change) or visible limit.

- Rendering hooks (in `CodeBuffer::draw`)
  - Compute visible line range; request `LineTok` for each visible line.
  - Before drawing monospace glyphs, set color per token run.
  - Selection overlay: if selected, use current selection path (shadow + fill + dark glyph) and skip token color.
  - Caret overlay unchanged.

- Theme load
  - Parse CODE_THEME from TIC-80 `config.tic` already embedded; map palette indices directly (Sweetie16 with white=12, greys 13/14/15).

## Tests (TDD)

- Tokenization unit tests (`highlight_tests.rs`)
  - Keywords vs identifiers; numbers (int/float/hex); strings short/long across lines; comments line/block with bracket levels.
  - State propagation across lines for long strings/comments.

- Rendering tests (framebuffer)
  - Draw a small snippet and assert pixel colors at representative positions:
    - keyword, number, string, comment, API function call, sign.
  - Verify selection overlay precedence (colored tokens become dark under selection).

- Performance sanity
  - Tokenize visible range only; tests ensure caches invalidate on edits and recompute affected lines (without exhaustively re‑lexing entire file).

## TODO Checklist

- [ ] Skeleton module `editor/highlight.rs` (tokens, state, API).
- [ ] Theme plumbing: expose CODE_THEME colors (BG/FG/STRING/NUMBER/KEYWORD/API/COMMENT/SIGN).
- [ ] Lua tokenizer (short strings, numbers, keywords, comments, signs, identifiers).
- [ ] Long strings / block comments with `[[` and `[=[` nesting; cross‑line state.
- [ ] API name set and classification.
- [ ] Highlighter cache in editor: dirty range strategy + forward recompute.
- [ ] Integrate with `CodeBuffer::draw` (colors for tokens; selection/caret precedence).
- [ ] Unit tests: tokenization coverage.
- [ ] Framebuffer tests: colored pixels for representative tokens; selection precedence.
- [ ] Config flag to disable highlighting (optional; default on).
- [ ] Documentation updates (implementation status, testing catalog).

## Out of Scope (later phases)

- Language modes beyond Lua (Wren, Squirrel, MoonScript, JS/Python): pluggable lexers.
- Identifier‑based semantic highlighting (locals/upvalues vs globals).
- Multi‑threaded/background lexing (current size/perf doesn’t require it).

---

Implementation will follow this plan and check off items as they land with tests.
