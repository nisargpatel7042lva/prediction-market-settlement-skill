# Anti-Manipulation Checklist

Settlement-specific attack patterns to check for when reviewing or building a prediction-market/outcome-resolution program. Pair with solana-dev-skill's general `security.md`; this file only covers settlement-specific vectors not already in a generic checklist.

## Attestor / Resolver Key Risks

- **Single key, no rotation plan.** If the attestor key that signs Merkle roots (or the authority allowed to call price-resolution) is a single hot keypair, a compromise lets an attacker resolve every open market in their favor. Use a multisig or a time-locked rotation path for production deployments.
- **No on-chain record of *which* key resolved which batch.** Store the resolver/attestor pubkey alongside the committed root or resolution, so a compromised-key incident can be audited and old roots invalidated without ambiguity.
- **Resolver key reused across unrelated programs.** Treat it as a single-purpose key; reuse increases blast radius if it leaks elsewhere.

## Timing Attacks

- **Resolving before the market's natural deadline.** Always enforce `resolve_at` / batch-closing checks server-side (on-chain), never rely on an off-chain caller to "be honest" about timing.
- **Late attestation accepted without a cutoff.** Decide explicitly how late a Merkle root can be committed before a market should instead go to its fallback or refund path — an attacker who controls timing of attestation can otherwise wait to see how positions are distributed before "attesting" a convenient result, in setups where the attestor itself is not fully independent of market participants.
- **Front-running the dispute window's expiry.** If finalization is a separate instruction anyone can call once the window closes, a bot racing to call it isn't itself a bug, but make sure nothing privileged happens differently based on *who* calls it.

## Data Integrity

- **Replay across batches.** Confirm a Merkle proof is checked against the root for the *specific* batch/epoch the market belongs to, not just "any root the program has ever stored" — store batch IDs and bind the check to the right one (see `merkle-outcome-verification.md`).
- **Leaf-encoding drift between off-chain and on-chain.** Re-verify after any change to either side; a silent encoding mismatch doesn't fail loudly, it just makes every proof fail or, worse, makes a *different* leaf's proof accidentally validate.
- **Oracle confidence ignored near the threshold.** See `pyth-fallback-resolution.md` — a price within its own confidence interval of the threshold is not a safe resolution point.

## Economic Manipulation

- **Wash-trading or self-dealing positions to manufacture a "consensus" challenge or to avoid a legitimate one** if your dispute mechanism weighs participation by position size — consider bond-based challenges (flat cost) over stake-weighted ones where possible.
- **Last-block stuffing before a price-threshold resolution** — a market resolving on a single observed price at a precise deadline is more exploitable than one using a short TWAP-style average if the underlying asset is thin enough to move in one transaction. Flag this risk explicitly to the user if their market resolves on illiquid or easily-moved price feeds.

## Review Checklist (quick pass)

- [ ] Is there a one-shot settle/claim flag, checked before every payout?
- [ ] Is the dispute window enforced as on-chain state, not just convention?
- [ ] Is the resolver/attestor key distinct from the market-creation authority?
- [ ] Is Merkle batch/epoch binding explicit, not implicit?
- [ ] Is oracle confidence checked, not just point price?
- [ ] Is there a documented fallback/refund path if resolution never arrives?
