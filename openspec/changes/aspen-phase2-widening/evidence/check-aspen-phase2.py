#!/usr/bin/env python3
import pathlib
import sys


def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        raise SystemExit(f"missing {label}: {needle}")


def forbid(text: str, needle: str, label: str) -> None:
    if needle in text:
        raise SystemExit(f"unexpected {label}: {needle}")


def main(default_path: str, coord_path: str) -> None:
    default_text = pathlib.Path(default_path).read_text()
    coord_text = pathlib.Path(coord_path).read_text()

    require(
        default_text,
        "[tigerstyle] scope:     -p aspen-time -p aspen-hlc -p aspen-core -p aspen-coordination",
        "widened default scope",
    )
    require(
        default_text,
        "[tigerstyle] lints:     ambient_clock compound_assertion contradictory_time ignored_result no_unwrap",
        "widened default lint set",
    )
    require(default_text, "Checking aspen-coordination v0.1.0", "default coordination check")
    require(coord_text, "[tigerstyle] scope:     -p aspen-coordination", "targeted coordination scope")
    require(coord_text, "Checking aspen-coordination v0.1.0", "targeted coordination check")
    forbid(default_text, "--> crates/aspen-coordination/", "default coordination finding")
    forbid(coord_text, "--> crates/aspen-coordination/", "targeted coordination finding")

    print("default pilot: widened scope and lint set confirmed")
    print("coordination slice: no starter-lint spans in saved transcript")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit("usage: check-aspen-phase2.py <default-log> <coord-log>")
    main(sys.argv[1], sys.argv[2])
