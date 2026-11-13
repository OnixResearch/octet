# Function fingerprint warning summary example

This example documents operator-facing warning summary fields for
`cargo octet fingerprint` human output. It keeps one marked function with a
macro call but no `macro=allow`, and one marked function without `review=` or
`evidence=` metadata, so the transcript records both `macro_warnings` and
`metadata_missing` counts.

The executable regression checks that matching hashes still fail when macro
review or required metadata is missing, and checks the updated source. It does
not require byte-for-byte transcript wording.
