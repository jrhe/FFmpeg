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

### Fuzz throughput

Build the fuzzer object (Rust enabled):

`./configure --disable-programs --enable-rust-hlsparser`
`make tools/target_hlsproto_fuzzer.o`

To measure executions/sec, link it with your libFuzzer toolchain (varies by platform) and run with a corpus.

## Notes

- These benchmarks are intentionally simple and are not a substitute for full profiling.
- For “startup latency” in end-to-end scenarios, prefer timing `ffprobe`/`ffmpeg` opening representative HLS inputs.

