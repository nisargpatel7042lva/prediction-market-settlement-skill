# Prediction Market Settlement Skill

> **Extends**: [solana-dev-skill](https://github.com/solana-foundation/solana-dev-skill)

An addon skill for Claude Code covering safe **settlement** of prediction markets, parametric/event derivatives, and outcome-resolved programs on Solana. This is distinct from price-oracle safety (Pyth/Switchboard live price reads) — it covers trusting a *claim about a past event* (a match result, a stat threshold, a parametric trigger) and paying out funds against that claim without getting drained.

## When Claude Should Load This Skill

Load when the user is:
- Building or reviewing a prediction market, parlay, or outcome-derivative program
- Verifying off-chain data (sports stats, election results, weather data) against an on-chain claim
- Implementing a Merkle-proof based settlement instruction
- Designing dispute/challenge windows before payout
- Resolving a market using a price oracle as a fallback or threshold check
- Reviewing a settlement instruction for manipulation or replay risk

Do NOT load for: generic price-oracle reads with no settlement/payout step (use solana-dev-skill or the oracle-safety skill instead), generic DeFi swaps/lending (use sendai/jupiter skills), token launches (use token-launch skill).

## Skill Files

| File | Load when... |
|---|---|
| [merkle-outcome-verification.md](./merkle-outcome-verification.md) | Verifying an off-chain attestor's signed Merkle root against a claimed leaf on-chain |
| [dispute-windows.md](./dispute-windows.md) | Designing the challenge period, who can challenge, what happens on a successful challenge |
| [pyth-fallback-resolution.md](./pyth-fallback-resolution.md) | Falling back to a live price oracle when there's no off-chain attestor, or resolving a price-threshold market |
| [settlement-account-patterns.md](./settlement-account-patterns.md) | Designing the Anchor PDA/escrow/vault structure that holds funds until resolution |
| [anti-manipulation.md](./anti-manipulation.md) | Reviewing for stale roots, replay, attestor key compromise, late-attestation attacks |
| [resources.md](./resources.md) | Links to reference implementations, audits, and further reading |

## Core Principles (always apply)

1. **Never pay out on an unverified claim.** Every settlement instruction must trace back to either a verified Merkle proof against a stored root, or a verified oracle account — never a bare instruction argument.
2. **Every resolution path needs a dispute window**, even a short one. Instant finality on event-outcome data is the single most common exploit vector in this category.
3. **Treat the attestor key like an oracle key.** Compromise/rotation procedures apply the same scrutiny as a price-feed authority.
4. **Settlement is the last line of defense, not the first.** Validate market state, expiry, and one-shot resolution (no double-settlement) before touching funds.

## Relationship to Core Skills

This skill assumes solana-dev-skill's `programs-anchor.md` and `security.md` are already loaded. It does not duplicate generic Anchor account-constraint guidance — only settlement-specific patterns.
