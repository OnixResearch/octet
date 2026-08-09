# Change-local acceptance

a[update-cairn-input.lock] The generated root `cairn` lock node MUST name revision `695124d459574ba7aeba6097310d237f393c243c`, and only Nix MAY generate the lock change.

a[update-cairn-input.migration] The native `.cairn/` tree MUST contain every legacy lifecycle file with the same bytes before the legacy tree is removed.

a[update-cairn-input.paths] Maintained Octet policy, source, fixture, and operator paths MUST select `.cairn/`; synthetic compatibility, historical, and external repository paths MAY retain their original spelling.

a[update-cairn-input.validation] Cairn traceability, policy freshness, focused Octet checks, and the final flake check MUST pass or retain only the exact recorded pre-change blocker.
