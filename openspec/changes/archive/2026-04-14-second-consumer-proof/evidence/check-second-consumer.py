#!/usr/bin/env python3
import pathlib
import sys


def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        raise SystemExit(f"missing {label}: {needle}")


def main(path: str) -> None:
    text = pathlib.Path(path).read_text()
    require(text, "[tigerstyle] workspace: aspen-verus-metrics", "workspace banner")
    require(text, "[tigerstyle] scope:     -p aspen-verus-metrics", "edited default scope")
    require(text, "[tigerstyle] profile:   ambient_clock compound_assertion contradictory_time", "starter profile banner")
    require(text, "Checking aspen-verus-metrics v0.1.0", "consumer package build")
    require(text, "Finished `dev` profile", "successful check completion")
    print("second consumer: template run reached aspen-verus-metrics with starter profile")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("usage: check-second-consumer.py <proof-log>")
    main(sys.argv[1])
