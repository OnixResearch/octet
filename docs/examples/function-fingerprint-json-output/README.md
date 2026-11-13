# Function fingerprint JSON output example

This example documents script-facing `--output-format=json` behavior for
`cargo octet fingerprint`. It records update, clean check, drift check,
accepted update, and final clean-check JSON payloads so automation can rely on
exact fields for mode, per-record status, review metadata, evidence metadata,
macro flags, and hash values.

The executable regression reruns update, clean check, drift, and repair in a
scratch directory. It parses the JSON records and checks statuses, hashes,
review metadata, and the resulting source. The transcript is an observed
capture, not a byte-for-byte wording assertion in the test suite.
