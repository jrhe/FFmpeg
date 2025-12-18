# Repository Guidelines

## Project Structure

- `libav*/*`: Core FFmpeg libraries (C). Most changes land here.
- `rust/*`: Optional “Rust islands” compiled as `staticlib` crates and linked into FFmpeg.
- `tools/*`: Developer tools, microbenches, and fuzz harness entrypoints.
- `tests/*`: Test harness and FATE definitions; reference outputs live under `tests/ref/fate/`.
- `fate-suite/`: External media samples checkout used by FATE (not part of upstream FFmpeg).

## Build, Test, and Development Commands

- Configure/build: `./configure [flags] && make -j$(sysctl -n hw.ncpu 2>/dev/null || echo 4)`
- Run broad tests (includes FATE): `FATE_SAMPLES=./fate-suite make check`
- Run FATE only: `FATE_SAMPLES=./fate-suite make fate`
- Source policy check (license headers/guards): `make fate-source`

Rust paths are opt-in via configure flags, e.g.:
- `./configure --disable-programs --enable-rust-webvtt`
- `./configure --disable-programs --enable-rust-subrip`

Microbenches (after building):
- `make tools/bench_webvtt && ./tools/bench_webvtt 2000000`
- `make tools/bench_subrip && ./tools/bench_subrip 2000000`

## Coding Style & Naming Conventions

- Follow existing FFmpeg C style in the touched file (brace placement, spacing, macro usage).
- Prefer small, focused diffs; keep C↔Rust boundaries explicit and `extern "C"` only.
- Rust code lives under `rust/<crate>/`; keep dependencies minimal and avoid unwinding across FFI.

## Testing Guidelines

- Add/adjust FATE tests when behavior or outputs change; update refs under `tests/ref/fate/`.
- Keep unit tests close to code when feasible (Rust: `cargo test` from the crate directory).

## Commit & Pull Request Guidelines

- Use short, imperative commit subjects; common prefixes in this repo include `rust:`, `hls:`, `spec:`, `tests:`.
- Include rationale and test commands in the commit body when changing parsing behavior.
- PRs should describe scope, flags used (e.g. `--enable-rust-*`), and relevant FATE targets.
