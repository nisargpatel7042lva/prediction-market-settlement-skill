# Merkle Outcome Verification

Pattern for trusting a claim about a past off-chain event (a match result, a stat threshold, an election outcome) and verifying it on-chain before any payout. This is the core primitive prediction markets need that generic price-oracle skills don't cover.

## The Problem

A program needs to settle a market based on something that happened off-chain (e.g. "Team A won," "player scored 30+ points"). You can't put raw off-chain data directly into an instruction argument and trust it — anyone could submit a fabricated argument. You need a way to:
1. Have a trusted off-chain attestor (a data provider, an oracle network, or a committee) commit to a batch of outcomes
2. Let anyone settle an individual market by proving their specific outcome was part of that committed batch
3. Do this without re-uploading the entire dataset on-chain for every market

Merkle proofs solve this: the attestor publishes one small root hash on-chain; anyone can later prove a specific leaf (one outcome) belongs under that root with a small proof, without trusting the submitter — only the root.

## Standard Flow

**1. Off-chain: attestor builds the tree**
The attestor (data provider / oracle committee) collects all outcomes for a batch (e.g. all matches finishing in a given window), hashes each outcome into a leaf, builds a Merkle tree, and signs the resulting root.

**2. On-chain: commit the root**
A `commit_root` instruction stores `{ root: [u8; 32], epoch_or_batch_id, attestor, published_at }` in a PDA. Only an authorized attestor key (or a multisig/oracle committee) can call this. Treat this key with the same operational rigor as a price-oracle authority — see `anti-manipulation.md`.

**3. On-chain: settle with proof**
A `settle_market` instruction takes the claimed outcome data plus a Merkle proof (array of sibling hashes). The program:
- Re-hashes the claimed leaf the same way the attestor did
- Walks the proof, recomputing parent hashes up to a root
- Compares the computed root against the stored root for that batch
- Only if it matches does it proceed to mark the market resolved and release funds

```rust
// Illustrative Anchor sketch — adapt hashing scheme to your leaf encoding
pub fn settle_market(ctx: Context<SettleMarket>, outcome: OutcomeData, proof: Vec<[u8; 32]>) -> Result<()> {
    let root_account = &ctx.accounts.committed_root;
    let leaf = hash_leaf(&outcome); // must match attestor's exact leaf encoding
    let computed_root = verify_proof(leaf, &proof, &outcome.leaf_index)?;
    require!(computed_root == root_account.root, MarketError::InvalidProof);
    require!(!ctx.accounts.market.is_settled, MarketError::AlreadySettled);
    // proceed: mark settled, move into dispute window (see dispute-windows.md)
    Ok(())
}
```

**4. Domain-separate your leaf hash.** Prefix the leaf data with a unique tag (e.g. `b"outcome_leaf"`) before hashing so an internal tree node can never be submitted as a valid leaf (second-preimage attack). The attestor and on-chain verifier must use the same prefix.

**5. Leaf encoding must be exact and documented.** A mismatch between how the attestor hashes a leaf off-chain and how the program reconstructs it on-chain is the single most common bug in this pattern. Pin the encoding (field order, byte lengths, hash function — typically `keccak256` or `sha256`) in one shared schema, not duplicated independently in two codebases.

## Common Mistakes

- **Trusting the proof submitter's claimed leaf index without checking it against the proof path.** An attacker can submit a valid proof for the *wrong* leaf if the index isn't bound into the verification.
- **No replay protection.** Always check `is_settled` (or an equivalent one-shot flag) before paying out — a valid proof can otherwise be replayed to drain funds repeatedly.
- **Root account with no batch/epoch identifier.** If you overwrite a single root account in place, you lose the ability to settle markets from a prior batch once a new root is committed. Use a PDA keyed by batch/epoch, or an append-only root history.
- **No expiry on unsettled markets.** Decide and enforce what happens if nobody calls `settle_market` before some deadline (refund path, fallback resolution — see `pyth-fallback-resolution.md`).

## When NOT to Use This Pattern

If the outcome is itself a live, continuously-updating price (not a discrete past event), you likely want a price oracle read instead — see `pyth-fallback-resolution.md`. Merkle proofs add real complexity (tree construction, proof generation, leaf-encoding discipline); only reach for this when the data source genuinely needs to commit to a large batch of discrete outcomes cheaply.
