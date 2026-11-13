# Security policy

Octet is under active development. Reports should identify the exact revision,
platform, configuration, and affected command. There is no promised response
SLA or long-term support window for historical revisions.

## Reporting a vulnerability

Use GitHub's **Report a vulnerability** action in the repository's Security tab:

<https://github.com/OnixResearch/octet/security/advisories/new>

If GitHub does not offer a private report form, do not enter vulnerability
details. Open an issue at <https://github.com/OnixResearch/octet/issues> asking
only for a private contact channel, without exploit details, sensitive source,
credentials, or private receipts. Do not post a vulnerability reproduction
publicly before coordinated review. The legacy `octet-public` repository is
not a supported reporting route.

Include the full affected revision, platform, exact command, configured scope,
prerequisites, minimal reproduction, expected and observed behavior, impact,
and redacted evidence. Redact credentials and private input data.
License, lint, proof, and build results are scoped evidence, not a guarantee
that Octet or checked programs are secure.
