# Clippy Policy (Rust Rewrite)

Goal: keep `cargo clippy --all-targets --all-features -D warnings` green while balancing ergonomics for TIC-80 style APIs and DSP-heavy code.

Defaults
- Enabled lints at crate root: `clippy::all`, `clippy::pedantic`, `clippy::nursery`, `clippy::cargo`, `rust_2018_idioms`.
- Treat warnings as errors in CI and local development.

Curated allows (project-wide)
- TIC API shape: `many_single_char_names`, `too_many_arguments`, `similar_names`.
- DSP and platform crates: `multiple_crate_versions` (transitive deps), select numeric casts where hot paths need clarity: `cast_possible_wrap`, `cast_sign_loss`, `cast_precision_loss`.

Localized allows (module/function scope)
- Long installers/glue (e.g., Lua API setup, CLI `main`): `too_many_lines`, `items_after_statements`, `missing_errors_doc`, `needless_pass_by_value`, `redundant_clone`, `option_if_let_else`, `uninlined_format_args`, `significant_drop_tightening`.
- VQT/FFT inner loops: `suboptimal_flops`, `imprecise_flops`, `cast_possible_truncation` where using `mul_add`, `hypot`, or explicit casts are already applied.
- `missing_const_for_fn`: prefer not to commit to `const` unless there’s a clear use/benefit and the API contract is stable.

Preferences
- Prefer lossless conversions via `From` over `as` where feasible.
- Prefer `map_or/map_or_else` over `if let`/`else` on `Option` when it improves clarity.
- Use `mul_add`, `hypot`, and `exp_m1` in numeric code where it improves precision/readability.
- Add `#[must_use]` to getters/utilities that return values callers should not ignore.

Process
- Fix high-signal lints; allow low-signal ones locally with rationale.
- Keep changes surgical; avoid broad global allows unless they’re a deliberate policy.

See also
- `AGENTS.md` Worklog entries for recent lint policy changes.
- `docs/specs/implementation_status.md` Hygiene section for current status.

