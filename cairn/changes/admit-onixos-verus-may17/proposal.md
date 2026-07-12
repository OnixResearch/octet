# Admit the OnixOS May 17 lifecycle verifier

## Problem

Octet's `onixos-lifecycle` row remains `local-pin` at the April verifier even though OnixOS has now executed all four lifecycle obligations with the published May 17 package. The central inventory must change only from repository-owned, identity-bound downstream evidence.

## Goal

Promote only the `onixos-lifecycle` compatibility row to `native` at `0.2026.05.17.e479cce`, binding the decision to OnixOS candidate receipt BLAKE3 `445e07a3dc5e5be6ebc6eadd75c34a5fe871de088a30c0578b39c2e8672d7d57` from OnixOS commit `292e55c6f954b7b8a3ba71643d4ddfd5c3b75a44`.

## Non-goals

This change does not alter Aspen, transfer OnixOS proof authority, complete the OnixOS Kamacite/Kani/Valence/Cairn chain, authorize physical mutation, or claim verifier soundness.
