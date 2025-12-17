# Benchmarking (Rust islands)

We add lightweight benchmarks for each Rust-backed component to quickly catch “terribly wrong” issues (e.g. infinite loops, accidental O(n²), extreme slowdowns) and to compare Rust vs C paths.

## HLS playlist parser

### Microbenchmark (CPU + wall)

Build with Rust enabled:

`./configure --disable-programs --enable-rust-hlsparser`

Build the benchmark:

`make tools/bench_hlsparser`

Run (provide any `.m3u8` file):

`./tools/bench_hlsparser /path/to/playlist.m3u8 200000`

Output includes total wall and CPU seconds for the requested iteration count.

This benchmark prints both a small C baseline (`hlsparser(c)`) and the Rust path (`hlsparser(rust)`).

### Startup latency

This measures end-to-end `ffprobe` startup latency while opening an HLS playlist using the `hls` protocol handler.

Build `ffprobe` (once):

`./configure --enable-ffprobe --disable-ffmpeg --disable-ffplay`
`make -j ffprobe`

Then run:

`./tools/bench_startup_latency_hlsproto.sh /path/to/playlist.m3u8 50`

To compare baseline vs Rust automatically (reconfigures and rebuilds `ffprobe`):

`./tools/bench_startup_latency_hlsproto_ab.sh /path/to/playlist.m3u8 50`

### Fuzz throughput

Build the fuzzer object (Rust enabled):

`./configure --disable-programs --enable-rust-hlsparser`
`make tools/target_hlsproto_fuzzer.o`

To measure executions/sec, link it with your libFuzzer toolchain (varies by platform) and run with a corpus.

## Notes

- These benchmarks are intentionally simple and are not a substitute for full profiling.
- For “startup latency” in end-to-end scenarios, prefer timing `ffprobe`/`ffmpeg` opening representative HLS inputs.

## HLS playlist writer

### Microbenchmark (CPU + wall)

Build:

`./configure --disable-programs --enable-rust-hlswriter`
`make tools/bench_hlswriter`

Run:

`./tools/bench_hlswriter 2000000 7`

This benchmark prints both a small C baseline (`hlswriter(c)`) and the Rust path (`hlswriter(rust)`).

## HLS demuxer playlist parser

The demuxer integration is staged: Rust produces a token/event stream, and the
apply layer can remain in C while coverage is expanded.

### Fuzz harness (event parser)

Build the fuzzer object (Rust enabled):

`./configure --disable-programs --enable-rust-hlsdemux-parser`
`make tools/target_hlsdemux_events_fuzzer.o`

## WebVTT parser

Microbenchmark (CPU + wall):

`./configure --disable-programs --enable-rust-webvtt`
`make tools/bench_webvtt`
`./tools/bench_webvtt 2000000`

## SubRip (SRT) parser

Microbenchmark (CPU + wall):

`./configure --disable-programs --enable-rust-subrip`
`make tools/bench_subrip`
`./tools/bench_subrip 2000000`

## MicroDVD parser

Fuzz harness (line parser):

`./configure --disable-programs --enable-rust-microdvd`
`make tools/target_microdvd_line_fuzzer.o`

## TTML helpers

Fuzz harness (extradata parser):

`./configure --disable-programs --enable-rust-ttml`
`make tools/target_ttml_extradata_fuzzer.o`

## MPL2 parser

Fuzz harness (line parser):

`./configure --disable-programs --enable-rust-mpl2`
`make tools/target_mpl2_line_fuzzer.o`

## VPlayer parser

Fuzz harness (line parser):

`./configure --disable-programs --enable-rust-vplayer`
`make tools/target_vplayer_line_fuzzer.o`

## JACOsub helpers

Fuzz harness (timestamp/shift parsing):

`./configure --disable-programs --enable-rust-jacosub`
`make tools/target_jacosub_ts_fuzzer.o`

## SubViewer / SubViewer1 helpers

Fuzz harness (timestamp parsing):

`./configure --disable-programs --enable-rust-subviewer --enable-rust-subviewer1`
`make tools/target_subviewer_ts_fuzzer.o`

## SCC helpers

Fuzz harness (payload word parsing):

`./configure --disable-programs --enable-rust-scc`
`make tools/target_scc_words_fuzzer.o`

## STL helpers

Fuzz harness (line parsing):

`./configure --disable-programs --enable-rust-stl`
`make tools/target_stl_line_fuzzer.o`
