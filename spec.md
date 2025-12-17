# Rust Migration Spec (FFmpeg fork)

## Goals

- Improve memory safety and parser robustness in historically bug-dense areas (demuxers, protocol handlers, manifest/playlist parsing).
- Keep FFmpeg’s codec inner loops, SIMD kernels, and performance-critical DSP in C/asm.
- Avoid coupling Rust to internal FFmpeg struct layouts (e.g., do not pass `AVFormatContext*` into Rust as an API surface).
- Migrate incrementally behind stable C ABI boundaries so upstream changes remain manageable.

## Non-goals

- Rewriting codec hot paths (entropy decode/encode inner loops, pixel math, SIMD).
- Introducing Rust as a required dependency for all builds (Rust should remain optional for a long time).
- Exposing Rust types across the boundary.

## Architecture: “Rust Islands”

Rust code is compiled as one or more `staticlib` crates and linked into FFmpeg.

### C ABI rules

- Rust entrypoints are `extern "C"` and `#[no_mangle]`.
- Data is passed as `(ptr, len)` slices or simple POD structs with explicit layout.
- Errors are returned as FFmpeg-style negative errno codes (or an error enum mapped to them).
- Rust must not unwind across FFI; panics are abort-only or caught and translated.

### Ownership rules

- Prefer: C allocates, C frees.
- Rust may allocate internally but must not transfer ownership to C unless using explicit alloc/free APIs.
- No implicit global state; if state is needed, use opaque handles created/destroyed by C.

### Build system rules

- Rust is behind `--enable-rust` (or per-component flags later).
- Cross compilation must be explicit: `RUST_TARGET` mirrors FFmpeg’s target triple selection.
- Keep the “default” build unchanged when Rust is disabled.

## Prioritized Conversion Backlog

## Component Tracker

This section tracks each Rust-backed component (“Rust island”) with its flag, wiring point, tests, fuzz target, and benchmarks.

Legend:
- **Flag**: configure flag that enables the Rust path.
- **Wiring**: the C entrypoint(s)/file(s) that call into Rust.
- **Tests**: minimum tests to run before landing changes.
- **Fuzz**: fuzzer target file(s) (throughput measurement is tracked separately in `decisions.md`).
- **Bench**: A/B microbench + startup-latency script(s) where relevant.
- **Status**: `done`, `in-progress`, or `planned`.

### Protocol / Manifest

| Component | Flag | Wiring | Tests | Fuzz | Bench | Status |
|---|---|---|---|---|---|---|
| HLS playlist writer (header) | `--enable-rust-hlswriter` | `libavformat/hlsplaylist.c` | `make fate-avstring` + `cargo test` | (n/a) | `tools/bench_hlswriter` (A/B) | done |
| HLS playlist parser (hlsproto) | `--enable-rust-hlsparser` | `libavformat/hlsproto.c` | `make fate` + `cargo test` | `tools/target_hlsproto_fuzzer.c` | `tools/bench_hlsparser` (A/B), `tools/bench_startup_latency_hlsproto*.sh` | done |
| HLS playlist parser (HLS demuxer) | `--enable-rust-hlsdemux-parser` | `libavformat/hls.c` | targeted `fate-hls*` + `make fate` | `tools/target_hlsdemux_events_fuzzer.c` | planned | done (subset; parse events staged) |
| DASH MPD parser | `--enable-rust-dash-mpd` | `libavformat/dash*` | targeted `fate-webm-dash-manifest*` + `make fate` | planned | planned | deferred (keep C parser for now) |

### Sidecar / Metadata

| Component | Flag | Wiring | Tests | Fuzz | Bench | Status |
|---|---|---|---|---|---|---|
| WebVTT parser | `--enable-rust-webvtt` | `libavformat/webvttdec.c` (or shared helper) | targeted subtitles FATE + `make fate` | `tools/target_webvtt_fuzzer.c` | `tools/bench_webvtt` | done |
| SRT/SubRip parser | `--enable-rust-subrip` | `libavformat/srtdec.c` | targeted subtitles FATE + `make fate` | `tools/target_subrip_fuzzer.c` | `tools/bench_subrip` | done |
| MicroDVD parser | `--enable-rust-microdvd` | `libavformat/microdvddec.c` | `make fate-sub-microdvd*` + `make fate` | `tools/target_microdvd_line_fuzzer.c` | planned | done (subset) |
| TTML helpers | `--enable-rust-ttml` | `libavformat/ttmlenc.c` | `make fate-sub-ttmlenc` + `make fate` | `tools/target_ttml_extradata_fuzzer.c` | planned | done (subset) |
| MPL2 parser | `--enable-rust-mpl2` | `libavformat/mpl2dec.c` | `make fate-sub-mpl2` + `make fate` | `tools/target_mpl2_line_fuzzer.c` | planned | done (subset) |
| VPlayer parser | `--enable-rust-vplayer` | `libavformat/vplayerdec.c` | `make fate-sub-vplayer` + `make fate` | `tools/target_vplayer_line_fuzzer.c` | planned | done (subset) |
| JACOsub helpers | `--enable-rust-jacosub` | `libavformat/jacosubdec.c` | `make fate-sub-jacosub*` + `make fate` | `tools/target_jacosub_ts_fuzzer.c` | planned | done (subset) |
| SubViewer helpers | `--enable-rust-subviewer` | `libavformat/subviewerdec.c` | `make fate-sub-subviewer` + `make fate` | `tools/target_subviewer_ts_fuzzer.c` | planned | done (subset) |
| SubViewer1 helpers | `--enable-rust-subviewer1` | `libavformat/subviewer1dec.c` | `make fate-sub-subviewer1` + `make fate` | `tools/target_subviewer_ts_fuzzer.c` | planned | done (subset) |
| SCC helpers | `--enable-rust-scc` | `libavformat/sccdec.c` | `make fate-sub-scc*` + `make fate` | `tools/target_scc_words_fuzzer.c` | planned | done (subset) |
| STL helpers | `--enable-rust-stl` | `libavformat/stldec.c` | `make fate-sub-stl` + `make fate` | `tools/target_stl_line_fuzzer.c` | planned | done (subset) |
| LRC helpers | `--enable-rust-lrc` | `libavformat/lrcdec.c` | `make fate-sub-lrc*` + `make fate` | `tools/target_lrc_ts_fuzzer.c` | planned | done (subset) |
| MPSub helpers | `--enable-rust-mpsub` | `libavformat/mpsubdec.c` | `make fate-sub-mpsub*` + `make fate` | `tools/target_mpsub_line_fuzzer.c` | planned | done (subset) |
| PJS helpers | `--enable-rust-pjs` | `libavformat/pjsdec.c` | `make fate-sub-pjs` + `make fate` | `tools/target_pjs_line_fuzzer.c` | planned | done (subset) |
| RealText helpers | `--enable-rust-realtext` | `libavformat/realtextdec.c` | `make fate-sub-realtext` + `make fate` | `tools/target_realtext_ts_fuzzer.c` | planned | done (subset) |
| AQTitle helpers | `--enable-rust-aqtitle` | `libavformat/aqtitledec.c` | `make fate-sub-aqtitle` + `make fate` | `tools/target_aqtitle_marker_fuzzer.c` | planned | done (subset) |
| SAMI helpers | `--enable-rust-sami` | `libavformat/samidec.c` | `make fate-sub-sami*` + `make fate` | `tools/target_sami_start_fuzzer.c` | planned | done (subset) |
| ASS helpers | `--enable-rust-ass` | `libavformat/assdec.c` | `make fate-sub-ass-to-ass-transcode` + `make fate` | `tools/target_ass_dialogue_fuzzer.c` | planned | done (subset) |
| MCC helpers | `--enable-rust-mcc` | `libavformat/mccenc.c`, `libavformat/mccdec.c` | `make fate-sub-mcc*` + `make fate` | `tools/target_mcc_hex_fuzzer.c`, `tools/target_mcc_payload_fuzzer.c` | planned | done (subset) |

### Demuxers / Muxers

| Component | Flag | Wiring | Tests | Fuzz | Bench | Status |
|---|---|---|---|---|---|---|
| Candidate demuxer #1 (TBD) | `--enable-rust-demux-<fmt>` | `libavformat/<fmt>.c` | targeted `fate-lavf-*` + `make fate` | planned | startup latency + A/B parse microbench | planned |
| Candidate demuxer #2 (TBD) | `--enable-rust-demux-<fmt>` | `libavformat/<fmt>.c` | targeted `fate-lavf-*` + `make fate` | planned | startup latency + A/B parse microbench | planned |

### Protocol Handlers

| Component | Flag | Wiring | Tests | Fuzz | Bench | Status |
|---|---|---|---|---|---|---|
| Candidate protocol #1 (TBD) | `--enable-rust-proto-<name>` | `libavformat/<proto>.c` | targeted protocol tests + `make fate` | planned | startup latency + A/B parse microbench | planned |

### Utilities

| Component | Flag | Wiring | Tests | Fuzz | Bench | Status |
|---|---|---|---|---|---|---|
| Shared bounded string/kv parser helpers | `--enable-rust-util-parse` | shared helper call sites | unit tests + `make fate` | planned | microbench planned | planned |
| ID3v2 helpers | `--enable-rust-id3v2` | `libavformat/id3v2.c` | `make fate-id3v2` + `make fate-adts-id3v2-demux` | `tools/target_id3v2_taglen_fuzzer.c` | planned | done (subset) |

### Tier 0: Tooling and scaffolding (do first)

1. **Build + configure integration**
   - Detect `cargo`/`rustc`, configure `RUST_TARGET`, and build Rust crates as part of `make`.
   - Produce deterministic artifacts and integrate with FFmpeg’s build modes (static/shared, cross, etc.).
   - Add CI job(s) that run with Rust enabled.

2. **FFI boundary library**
   - Create a small, audited C header surface for each Rust module.
   - Provide common helpers: bounded parsing, logging adapter, error mapping.

3. **Fuzzing pipeline**
   - Add fuzz targets for each Rust parser.
   - Keep corpus compatibility with existing FFmpeg fuzzing where possible.

### Tier 1: High-bug-density parsers (best ROI)

4. **Playlist/manifest parsing**
   - HLS playlists (`.m3u8`) and DASH MPDs (XML parsing) are complex, parsing-heavy, and relatively self-contained.
   - Plan:
     - Define ABI: `parse_manifest(input_ptr,len, out_ptr,out_len, ...)` producing a structured, bounded output (e.g. a compact table or JSON-like token stream).
     - In C, keep existing demuxer logic but swap the parsing front-end behind a feature flag.
    - Add fuzzers and regression tests for tricky edge cases.

### Initial pilot (this repo)

- **Pilot 1: HLS playlist emission (writer-side) helper**
  - Scope: replace `ff_hls_write_*` string formatting helpers with a Rust implementation behind a build flag.
  - Rationale: low coupling (operates on primitives/strings), easy to test, establishes C↔Rust patterns.
  - Exit criteria: byte-for-byte identical output for representative inputs and passing FATE tests.

5. **Sidecar metadata formats**
   - e.g. WebVTT, SRT, TTML, ID3 tag parsing utilities.
   - Plan:
     - Identify minimal API that returns parsed cues/tags as arrays of POD structs.
     - Keep demuxers/muxers in C; call Rust to parse sidecar text/binary payloads.
     - Ensure all allocations are done by C or via explicit “size query then fill” APIs.

6. **Specific demuxers/muxers for attack-surface formats**
   - Start with formats that are parsing-heavy and security sensitive, but not deeply intertwined.
   - Plan:
     - Pick one format at a time.
     - Introduce a Rust “reader” that consumes `AVIOContext` bytes via a tiny C shim (read/seek callbacks) rather than exposing `AVIOContext*`.
     - Return parsed packet boundaries + metadata to the existing C demuxer/muxer.
     - Gate behind `--enable-rust-demuxer-foo` once the pattern is proven.

### Tier 2: Protocol handlers (good isolation)

7. **HTTP-ish / custom protocol handlers**
   - Protocol handlers often have tricky state machines and security issues.
   - Plan:
     - Provide an opaque `rs_protocol_handle` with callbacks for read/write/seek.
     - Use Rust for URL parsing, header parsing, redirects, and state machine.
     - Keep socket I/O integration in C initially to avoid platform-specific duplication.

8. **Crypto and hashing helpers (careful)**
   - Only if it improves safety without harming performance; keep existing optimized code where critical.
   - Plan:
     - Start with non-hot utilities or ones used primarily for validation.
     - Provide ABI: `hash_update(handle, ptr,len)` etc.
     - Ensure side-channel/security considerations are documented.

### Tier 3: Orchestration (not pixel math)

9. **Filter graph scheduling/orchestration**
   - Focus on correctness: queueing, cancellation, deterministic shutdown, flush logic.
   - Plan:
     - Introduce Rust implementation behind a stable “scheduler” interface.
     - Keep filter implementations in C; Rust manages graph execution and threading.
     - Add heavy test coverage: deadlock regression tests, deterministic teardown.

## Per-Item Conversion Template

For each component to migrate:

1. **Select component + define boundaries**
   - Identify minimal inputs/outputs that can be expressed as bytes + POD.

2. **Design ABI**
   - `init()` / `destroy()` for state.
   - `parse()` / `step()` for processing.
   - “size query then fill” pattern for returned data.

3. **Implement Rust crate**
   - No panics across FFI.
   - Minimize dependencies; audit licenses.
   - Add unit tests.

4. **Wire behind existing interface**
   - Keep the current C path as fallback.
   - Feature-gate, log when Rust path is active.

5. **Fuzz + regression tests**
   - Add fuzz target with corpus seed.
   - Add deterministic unit/integration tests for known tricky inputs.

6. **Performance check**
   - Ensure no extra copies in hot loops.
   - Keep boundary crossings coarse-grained.

7. **Rollout**
   - Start experimental/opt-in.
   - Collect crash/bug reports.
   - Promote to default only when stable.

## Milestones

1. **M1: Rust build + one shipped parser**
   - Rust optional build works on at least macOS/Linux.
   - One parser/protocol path replaced behind a flag.
   - Fuzz target in-tree.

2. **M2: 3–5 Rust parsers**
   - HLS/DASH + 1–2 demuxers + 1 protocol handler.
   - CI jobs run with Rust enabled.

3. **M3: Orchestration pilot (optional)**
   - One clearly bounded scheduling/queueing subsystem behind a flag.
