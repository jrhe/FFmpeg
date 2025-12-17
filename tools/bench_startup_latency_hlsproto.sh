#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <playlist.m3u8> [runs]" >&2
  exit 2
fi

PLAYLIST="$1"
RUNS="${2:-50}"

if [[ ! -x ./ffprobe ]]; then
  echo "ffprobe not built; build it with ./configure --enable-ffprobe && make -j" >&2
  exit 2
fi

if [[ ! -f "$PLAYLIST" ]]; then
  echo "playlist not found: $PLAYLIST" >&2
  exit 2
fi

times=()
for ((i=0; i<RUNS; i++)); do
  start_ns=$(python - <<'PY'
import time
print(time.time_ns())
PY
)
  # Use hls protocol handler explicitly to exercise the playlist parser.
  ./ffprobe -hide_banner -loglevel error -show_entries format=duration -of default=nw=1:nk=1 "hls+file:${PLAYLIST}" >/dev/null || true
  end_ns=$(python - <<'PY'
import time
print(time.time_ns())
PY
)
  times+=("$(( (end_ns - start_ns) / 1000000 ))")
done

python - <<'PY'
import statistics, sys
times = [int(x) for x in sys.argv[1:]]
times.sort()
def pct(p):
    if not times:
        return None
    k = int(round((p/100.0) * (len(times)-1)))
    return times[k]

print(f"runs={len(times)} ms: min={times[0]} p50={pct(50)} p90={pct(90)} p99={pct(99)} max={times[-1]} mean={statistics.mean(times):.2f}")
PY "${times[@]}"

