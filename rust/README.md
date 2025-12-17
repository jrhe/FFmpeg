# Rust components (experimental)

This tree hosts Rust implementations wired into FFmpeg behind stable C ABI entrypoints.

- Rust code must expose `extern "C"` functions only.
- Pass data as pointers + lengths; avoid depending on FFmpeg internal struct layout.
- Do not panic across FFI; builds use `panic = abort`.

