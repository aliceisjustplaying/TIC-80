# ADR 0003: Lua 5.3 With 5.1/5.2 Compatibility

Status: Accepted (supersedes ADR 0002)
Date: 2025-08-26

Context
- TIC-80 cartridges historically target Lua 5.3 semantics with selected 5.1/5.2 compatibility. Matching those semantics reduces drift across carts and the reference C build.

Decision
- Use `mlua` with vendored Lua 5.3: `mlua = { version = "0.9", features = ["lua53", "vendored"] }`.
- Enable 5.1/5.2 compatibility macros during Lua build (LUA_COMPAT_5_2/5_1) where feasible. If the vendored build lacks direct flags, emulate required behaviors at the API boundary and cover with tests.

Consequences
- Closer behavioral parity with TIC-80 vs. Lua 5.4.
- Some compatibility shims may still be required; capture deltas with tests.

Notes
- ADR 0002 (Lua 5.4) is superseded by this ADR. Keep it for historical context.

