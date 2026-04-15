#!/usr/bin/env bash
set -euo pipefail

script_dir=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
transcript="$script_dir/aspen-phase1-revalidation.txt"
tmp_transcript=$(mktemp "$script_dir/aspen-phase1-revalidation.txt.XXXXXX")
record_rel='openspec/changes/diagnostic-quality/evidence/record-revalidate-aspen-phase1.sh'
replay_rel='openspec/changes/diagnostic-quality/evidence/revalidate-aspen-phase1.sh'

cleanup() {
  rm -f "$tmp_transcript"
}
trap cleanup EXIT

{
  printf '$ bash %s\n' "$record_rel"
  printf '$ bash %s\n' "$replay_rel"
  bash "$script_dir/revalidate-aspen-phase1.sh"
} | tee "$tmp_transcript"

mv "$tmp_transcript" "$transcript"
trap - EXIT
