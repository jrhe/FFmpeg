# Testing FFmpeg (this repo)

This repository uses `make` targets under `tests/`.

## Quick start: run everything

1. Configure and build:

```sh
./configure
make -j$(sysctl -n hw.ncpu 2>/dev/null || nproc)
```

2. Run the full test suite:

```sh
FATE_SAMPLES=./fate-suite make check
```

`make check` runs a broad set of build+test targets including `testprogs` and `fate`.

## FATE sample set (`./fate-suite`)

Many FATE tests need external media samples (the “fate-suite” corpus). This is
not included in the FFmpeg source tree.

Fetch it into the repo root as `./fate-suite`:

```sh
git clone https://github.com/FFmpeg/fate-suite.git fate-suite
```

Optionally, update samples via rsync (some tests expect the sample checkout to
be in sync with what your build expects):

```sh
FATE_SAMPLES=./fate-suite make fate-rsync
```

Then run FATE:

```sh
FATE_SAMPLES=./fate-suite make fate
```

## Understanding results

- The exit status of `make check` / `make fate` is non-zero if any tests fail.
- FATE prints `PASS`/`FAIL` lines in the terminal.
- Detailed logs and outputs are written under `tests/fate/` in the build tree
  (failures typically mention the log file path).

## Helpful targets

- List all FATE tests: `make fate-list`
- List tests that failed last run: `make fate-list-failing`
- Re-run only failing tests (if available in your build): `make fate-rerun`
- Run asm verification: `make checkasm` (or `make run-checkasm`)

## Notes

- You must run `./configure` and `make` before running tests.
- If you build out-of-tree (separate build directory), run these commands from
  that build directory, and keep `FATE_SAMPLES` pointing at the samples path.
