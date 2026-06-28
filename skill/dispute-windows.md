# Dispute Windows

A resolution that becomes final the instant it's submitted is the single biggest exploit surface in outcome-settled markets. This file covers designing the window between "a settlement claim was submitted" and "funds actually move."

## Why You Need One

Even with a verified Merkle proof or oracle read (see `merkle-outcome-verification.md`, `pyth-fallback-resolution.md`), the *underlying data* can still be wrong: an attestor's feed had a bug, a price oracle glitched, a stat provider corrected a result after the fact. A dispute window doesn't replace data verification — it's the safety net for when verified-but-wrong data gets submitted.

## Core State Machine

```
Open → ClaimSubmitted → (dispute window open) → Finalized → Paid
                              │
                              ▼ (challenge raised + accepted)
                         Disputed → Open (re-open for new claim) or Cancelled
```

Track this explicitly in the market account, not implicitly via timestamps alone:

```rust
pub enum MarketState {
    Open,
    ClaimSubmitted { claimed_at: i64, claimed_outcome: OutcomeData },
    Finalized { outcome: OutcomeData },
    Disputed { challenger: Pubkey, raised_at: i64 },
    Cancelled,
}
```

## Designing the Window

**Length.** Long enough that a legitimate challenger (with a financial stake or a watcher bot) has time to notice and respond, short enough that the market doesn't feel broken to honest users. There's no universal number — it depends on how fast your data source can be independently checked. A few minutes is reasonable for a price-threshold market with multiple independent oracle sources to cross-check against; longer (hours) makes sense for outcomes that require human/manual cross-verification (e.g. a disputed sports result).

**Who can challenge.** Decide explicitly:
- Anyone, with a bond (refunded if the challenge succeeds, slashed if it fails) — prevents spam challenges while keeping it permissionless
- Only addresses with an existing position in the market — limits the attack surface to people with real skin in the game
- A designated committee/multisig — simpler, but reintroduces centralization you may be trying to avoid elsewhere

**What a successful challenge does.** Most robust option: revert to `Open` or `Cancelled` (full refund) rather than trying to programmatically "fix" the outcome on-chain — resolving *what actually happened* is an off-chain problem; the contract's job is just to not pay out on bad data.

## Common Mistakes

- **No bond on challenges** → free spam vector that can stall every market indefinitely.
- **Window too short to be checked, but long enough to be annoying** — worse than no window at all, since it adds latency without adding real safety.
- **Forgetting to lock the funds during the window.** `ClaimSubmitted` must still block withdrawals/deposits; only `Finalized` should unlock the payout path.
- **No cap on how many times a market can be disputed.** Pair with a maximum dispute count or escalating bond requirement to prevent indefinite stalling by a wealthy attacker.
