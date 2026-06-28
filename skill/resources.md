# Resources

## Reference Patterns (general Merkle/airdrop verification, same primitive)
- Solana's own token-distribution / Merkle-airdrop pattern repos use the identical leaf-hash + proof-walk verification primitive described in `merkle-outcome-verification.md` — useful as a working reference for the verification math even though the domain (airdrops vs outcome data) differs.

## Oracle Documentation
- Pyth Network's price-feed integration docs — confidence intervals, staleness, `get_price_no_older_than` and equivalents.
- Switchboard's on-demand feed docs — relevant if a project uses Switchboard instead of/alongside Pyth for the fallback path in `pyth-fallback-resolution.md`.

## Related Kit Skills
- `solana-dev-skill` (core) — Anchor account constraints, general security checklist. This skill assumes that one is loaded first.
- An oracle price-read-safety skill, if present in the kit, covers *reading* Pyth/Switchboard prices generally; this skill only covers the *settlement* use of that read, not generic price consumption (e.g. for swaps/lending).

## Notes for Maintainers
This skill intentionally does not vendor a specific data provider's API (e.g. a named third-party sports-data or oracle service) — the patterns here are provider-agnostic so the skill stays correct regardless of which attestor or oracle a given project integrates with. When adapting this skill for a specific integration, add a project-local `*.md` file documenting that provider's exact leaf encoding and API surface rather than editing the core pattern files.
