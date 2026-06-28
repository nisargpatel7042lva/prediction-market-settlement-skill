# prediction-market-settlement-skill

A Claude Code skill for safely settling prediction markets and outcome-resolved programs on Solana.

## What this skill covers — and what it doesn't

**This skill is about trusting a claim about a discrete past event and paying funds out against it.** That is a different problem from reading a live price feed safely for a swap or lending protocol (the `solana-oracle-skill` covers that). Here the question is: someone is telling you "Team A won" or "Player X scored 30+ points" — how do you verify that on-chain, open a dispute window so a wrong result can be challenged, and then release escrow funds exactly once?

Topics:
- Merkle-proof verification of off-chain attestations (batch outcomes, sports stats, event data)
- Dispute/challenge windows between claim submission and payout finalization
- Price-threshold resolution via Pyth as a fallback path
- Settlement-specific PDA/escrow/vault account patterns
- Anti-manipulation checklist (timing attacks, replay, attestor key risks, oracle confidence near thresholds)

## Relationship to solana-dev-skill

This is an addon skill — it extends [solana-dev-skill](https://github.com/solana-foundation/solana-dev-skill) the same way `solana-game-skill` does. It assumes `programs-anchor.md` and `security.md` from that core skill are already loaded and does not duplicate generic Anchor or security guidance.

## Installation

### Quick (non-interactive, recommended defaults)

```bash
curl -fsSL https://raw.githubusercontent.com/nisargpatel7042lva/prediction-market-settlement-skill/main/install.sh | bash
```

### Interactive (choose what to install)

```bash
curl -fsSL https://raw.githubusercontent.com/nisargpatel7042lva/prediction-market-settlement-skill/main/install-custom.sh | bash
```

### Manual

Copy the `skill/` directory into your project, then add this line to your project's `CLAUDE.md`:

```
skill: ./skill/SKILL.md
```

## Skill files

| File | When Claude loads it |
|---|---|
| `skill/SKILL.md` | Entry point — routing and core principles |
| `skill/merkle-outcome-verification.md` | Verifying an off-chain attestor's signed Merkle root on-chain |
| `skill/dispute-windows.md` | Designing the challenge period and state machine |
| `skill/pyth-fallback-resolution.md` | Price-threshold resolution and oracle fallback path |
| `skill/settlement-account-patterns.md` | PDA/escrow/vault structure for holding and releasing funds |
| `skill/anti-manipulation.md` | Review checklist for timing attacks, replay, and economic manipulation |
| `skill/resources.md` | Reference implementations, oracle docs, related skills |

## Background

This skill was developed alongside real projects:

- **EPOCH** — a Solana prediction market using MagicBlock Ephemeral Rollups + Pyth oracle resolution
- A World Cup prediction-market submission for the TxODDS/Superteam hackathon using TxLINE's on-chain `validateStat` instruction with Merkle proof verification

The skill itself is provider-agnostic. If you're integrating a specific attestor or oracle API, add a project-local `.md` file documenting that provider's leaf encoding and API surface rather than editing the core pattern files.

## License

MIT — see [LICENSE](./LICENSE).
