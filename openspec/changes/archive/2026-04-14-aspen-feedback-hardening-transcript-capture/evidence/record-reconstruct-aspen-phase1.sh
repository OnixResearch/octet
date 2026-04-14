#!/usr/bin/env bash
set -euo pipefail

script_dir=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
transcript="$script_dir/reconstruct-aspen-phase1.txt"
tmp_transcript=$(mktemp "$script_dir/reconstruct-aspen-phase1.txt.XXXXXX")
record_rel='openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/record-reconstruct-aspen-phase1.sh'
replay_rel='openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/reconstruct-aspen-phase1.sh'

cleanup() {
  rm -f "$tmp_transcript"
}
trap cleanup EXIT

{
  printf '$ bash %s\n' "$record_rel"
  printf '$ bash %s\n' "$replay_rel"
  bash "$script_dir/reconstruct-aspen-phase1.sh"
} | tee "$tmp_transcript"

mv "$tmp_transcript" "$transcript"
trap - EXIT
