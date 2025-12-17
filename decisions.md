# Decisions to Review (Rust migration)

This file tracks choices that affect multiple future “Rust islands”. None of these need to block the current low-risk pilots, but we should decide them before scaling beyond a few components.

## Build / integration

1. **How should Rust crates be selected/linked?**
   - Current: `ffbuild/library.mak` links only the Rust libs needed for enabled components (e.g. `CONFIG_RUST_HLSPARSER`, `CONFIG_RUST_MICRODVD`, …) and builds `rust-libs` as a prerequisite when any `CONFIG_RUST_*` is enabled.
   - Options:
     - Link only the Rust libs needed for enabled components (`CONFIG_RUST_HLSWRITER`, `CONFIG_RUST_HLSPARSER`, …).
     - Always link all Rust libs when `--enable-rust`, but gate code usage behind `CONFIG_RUST_*`.

2. **Where should built Rust artifacts live?**
   - Current: `rust/<crate>/target/...` (Cargo default).
   - Options:
     - Keep Cargo defaults.
     - Use `CARGO_TARGET_DIR` under `ffbuild/` to centralize and simplify cleaning.

3. **Cross compilation story**
   - How do we map FFmpeg configure target (and sysroot) to a Rust target triple and linker?
   - Do we require `cargo -Z build-std` for some targets or keep `no_std` crates only?

4. **Per-component configure flags naming**
   - Current: `--enable-rust-hlswriter`, `--enable-rust-hlsparser`.
   - Alternative: group under `--enable-rust-parsers`, `--enable-rust-protocols`, etc.

## FFI policy

5. **Error code conventions**
   - Decide the canonical mapping: Rust parser returns `0/-EINVAL/-ENOMEM/...` vs custom negative codes remapped by C.

6. **String / buffer return patterns**
   - Current HLS parser returns URL slices as offsets/lengths into input buffer.
   - Alternative: “size query then fill” with an output arena allocated by C.

7. **Panic policy**
   - Current: `panic=abort` and `no_std` with a tiny `#[panic_handler]`.
   - Alternative: `catch_unwind` in a `std` crate (but that increases deps and needs an allocator).

## Testing and rollout

8. **What does “tested Rust versions rather than default ones” mean in CI?**
   - Always run a CI job with `--enable-rust-*` flags for new components.
   - For each migrated component, add a deterministic unit test and a fuzzer harness.

9. **Fuzzing integration**
   - Should Rust fuzz targets be:
     - C fuzzer harnesses calling Rust (current for HLS), or
     - Rust `cargo-fuzz` targets, or
     - both?

10. **Promotion criteria**
   - When does a Rust component become default?
   - After N fuzz-hours + FATE stable + performance parity.

## Benchmarking TODOs (deferred)

1. **Fuzz throughput harness**
   - Add a simple, reproducible way to measure executions/sec for each fuzzer target.
   - Options: libFuzzer-based run in CI, or a local “tight loop” harness that calls the fuzzer entrypoint over corpus inputs.

## Blocking/feedback needed (captured while continuing)

1. **P1 scope realism**
   - P1 includes substantial features (full HLS demuxer playlist parsing + DASH MPD + VTT/SRT). Implementing all of these robustly is non-trivial and will require agreeing on coverage targets, dependency policy (XML parsing), and which exact tags/features must be supported for parity.

2. **HLS demuxer parser boundary**
   - HLS demuxer `parse_playlist()` handles many tags (KEY, MAP, MEDIA renditions, byterange, playlist type, etc.). We need to decide whether the Rust parser replaces the whole function or only a subset initially (and keeps some tag parsing in C).

3. **DASH MPD parsing dependencies**
   - Decide whether Rust MPD parsing can use an XML crate (license audit + build/cross-compile implications) or must be a minimal, internal parser.

## HLS demuxer migration approach (recommended)

Goal: avoid a “big bang” rewrite of `libavformat/hls.c:parse_playlist()` while still moving the risky text parsing surface into Rust.

1. **Rust “parse” layer (token/event stream)**
   - Rust consumes the playlist bytes and emits a compact array of POD events (tag kind + value spans into the original buffer + parsed numeric fields where useful).
   - Unknown tags are preserved as “unknown” events rather than triggering failure.
   - No allocations in Rust; caller provides the output buffer and can size-query.

2. **C “apply” layer (initially)**
   - C iterates the event stream and applies semantics to existing `HLSContext`/`playlist`/`segment` structs using existing helpers.
   - Fallback to the legacy C line parser on Rust parse failure, truncation, or unsupported events while coverage is grown.

3. **Future: Rust “apply” layer behind a separate flag**
   - A Rust apply implementation (building the same semantic model) should be gated behind a separate configure flag so it can be enabled independently of the parse/token layer.
   - This keeps the low-risk “Rust parsing + C semantics” path available even if the Rust apply path is still maturing.

### Rollout plan (tests + fuzzing)

- Add/maintain a libFuzzer entrypoint for the Rust event parser (`tools/target_hlsdemux_events_fuzzer.c`).
- Add a “differential” debug mode in C (non-fate) that runs both C-line parsing and Rust-events parsing on the same input and compares a normalized summary (segment count/durations, variants, media sequence, endlist) to catch semantic drift.
- Grow coverage by implementing additional event kinds (KEY/MAP/MEDIA/BYTERANGE/PROGRAM-DATE-TIME/PLAYLIST-TYPE/START/…) while keeping C as the apply layer.
- Once event coverage is high and FATE is stable, prototype a Rust apply layer behind `--enable-rust-hlsdemux-apply` and fuzz it separately (apply fuzz target should construct a minimal `HLSContext`/`playlist` and apply the event stream).

## Next targets

- MicroDVD: keep C demuxer semantics but move per-line parsing into Rust behind `--enable-rust-microdvd`, then add a full-file event parser only if needed for performance.
- TTML: treat the first step as hardening parsing of `ttmlenc` extradata (NUL-separated strings) in Rust; defer full TTML XML parsing until we decide on XML dependencies.

## P1 status log

- 2025-12-17: Added benches + fuzz harnesses for Rust WebVTT/SubRip and ran `make fate` with `FATE_SAMPLES=./fate-suite`; no remaining P1 blockers identified.
- 2025-12-17: Completed Rust JACOsub timestamp/shift helpers (`--enable-rust-jacosub`) and ran `fate-sub-jacosub*`; updated linking so subtitle Rust crates are added to `ffbuild/library.mak` when enabled.
- 2025-12-17: Added Rust helpers for ffconcat/concat demuxer token parsing (`--enable-rust-concat`) with fuzzer `tools/target_concat_token_fuzzer.c`; behavior remains C-compatible and falls back on Rust errors.
- 2025-12-17: Added Rust `data:` URI protocol parsing helper (`--enable-rust-data-uri`) plus a deterministic FATE test (`fate-data-uri-wav`) and fuzzer `tools/target_data_uri_fuzzer.c`.
