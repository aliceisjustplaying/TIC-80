# ADR 0002: Lua Engine and Compatibility

Status: Superseded by ADR 0003
Date: 2025-08-26

Context
- TIC-80 carts rely on Lua semantics aligned with 5.2 behavior; portability and deterministic behavior are priorities.

Decision
- Use `mlua` with vendored Lua 5.4, enabling behavior that preserves expected 5.2 semantics where applicable (see tests).

Consequences
- Portable across supported platforms; avoids LuaJIT portability trade-offs initially.
- Add tests around numeric semantics and iteration order to guard against drift.
