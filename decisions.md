# Decisions to Review (Rust migration)

This file tracks choices that affect multiple future “Rust islands”. None of these need to block the current low-risk pilots, but we should decide them before scaling beyond a few components.

## Build / integration

1. **How should Rust crates be selected/linked?**
   - Current: `ffbuild/library.mak` always adds Rust libs when `HAVE_FFMPEG_RUST`, even if a specific feature flag isn’t enabled.
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
