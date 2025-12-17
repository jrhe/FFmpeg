#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <playlist.m3u8> [runs]" >&2
  exit 2
fi

PLAYLIST="$1"
RUNS="${2:-50}"

if [[ ! -f "$PLAYLIST" ]]; then
  echo "playlist not found: $PLAYLIST" >&2
  exit 2
fi

echo "== baseline (C) =="
./configure --disable-doc --disable-ffmpeg --disable-ffplay --enable-ffprobe >/dev/null
make -j ffprobe >/dev/null
./tools/bench_startup_latency_hlsproto.sh "$PLAYLIST" "$RUNS"

echo "== rust-hlsparser =="
./configure --disable-doc --disable-ffmpeg --disable-ffplay --enable-ffprobe --enable-rust-hlsparser >/dev/null
make -j ffprobe >/dev/null
./tools/bench_startup_latency_hlsproto.sh "$PLAYLIST" "$RUNS"

