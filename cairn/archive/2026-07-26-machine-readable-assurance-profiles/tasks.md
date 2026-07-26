# Tasks: Machine-readable assurance profiles

## Implementation

- [x] [serial] I1 Add the closed Nickel profile contracts and cross-reference rules r[octet.assurance_profiles.typed_contract]
- [x] [serial] I2 Add the full OnixOS lifecycle pilot plus a minimal positive fixture r[octet.assurance_profiles.onixos_pilot]
- [x] [serial] I3 Add negative fixtures for empty, duplicate, unresolved, unreferenced, unknown-plane, and unknown-field inputs r[octet.assurance_profiles.fail_closed]
- [x] [serial] I4 Add deterministic generated JSON and a Nickel Export BLAKE3 manifest r[octet.assurance_profiles.identity]
- [x] [serial] I5 Register the family and add a first-class Nix package and focused check r[octet.assurance_profiles.distribution]
- [x] [serial] I6 Document profile selection, ownership, and non-claim boundaries r[octet.assurance_profiles.boundaries]

## Verification

- [x] [parallel] V1 Run valid and invalid Nickel fixtures and save the focused result r[octet.assurance_profiles.fail_closed]
- [x] [parallel] V2 Regenerate JSON and the BLAKE3 manifest twice and compare exact outputs r[octet.assurance_profiles.identity]
- [x] [parallel] V3 Build the assurance-profile package and validate its installed paths and profile identity r[octet.assurance_profiles.distribution]
- [x] [parallel] V4 Run Octet policy validation and focused standards tests after the change r[octet.assurance_profiles.typed_contract]
- [x] [serial] V5 Run Cairn validation and proposal, design, and tasks gates r[octet.assurance_profiles.boundaries]

## Verification coverage

- `Scenario: Complete profile exports` -> I1, I2, V1
- `Scenario: Malformed or disconnected profile fails closed` -> I3, V1
- `Scenario: Profile identity is deterministic` -> I4, V2
- `Scenario: Profile drift changes or invalidates identity` -> I4, V2
- `Scenario: OnixOS pilot preserves all assurance planes and gaps` -> I2, V1
- `Scenario: Packaged profile is consumable without runtime Nickel` -> I5, V3
- `Scenario: Ordinary Octet checks remain unchanged` -> I6, V4
- `Scenario: Profile language preserves non-claims` -> I6, V5
