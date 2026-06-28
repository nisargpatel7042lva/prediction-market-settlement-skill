# Pyth Fallback Resolution

Pattern for resolving a market against a live price oracle (Pyth, or Switchboard) rather than an off-chain attestor's Merkle-committed batch. Use this file when the market's outcome is a price crossing a threshold (e.g. "will SOL close above $X", session-based markets keyed to an oracle price at a deadline), or as a fallback path when no off-chain attestation arrives in time for a Merkle-based market.

This is deliberately scoped narrower than a general "how to read a Pyth price" guide (that belongs in a core/oracle-safety skill) — this file covers the settlement-specific concerns: reading the price *at the right moment*, defending against staleness, and using it to flip market state safely.

## Standard Flow

**1. Pin the resolution moment.** Decide and store on-chain, at market creation, exactly which timestamp the price is read against (e.g. round expiry, a fixed `resolve_at` slot/timestamp). Never resolve "whenever someone happens to call the instruction" against "whatever the current price is" — that creates a window for someone to time the call to their advantage.

**2. Read the Pyth price account inside the settlement instruction**, not from an off-chain script that then submits a number. Always read on-chain so the value can't be substituted between observation and submission.

```rust
pub fn resolve_price_market(ctx: Context<ResolvePriceMarket>) -> Result<()> {
    let price_update = &ctx.accounts.price_update; // Pyth PriceUpdateV2 account
    let clock = Clock::get()?;

    require!(clock.unix_timestamp >= ctx.accounts.market.resolve_at, MarketError::TooEarly);

    let price = price_update.get_price_no_older_than(
        &clock,
        MAX_STALENESS_SECONDS, // e.g. 60 — tune to your market's time sensitivity
        &ctx.accounts.market.feed_id,
    )?;

    require!(price.conf < MAX_ACCEPTABLE_CONFIDENCE_INTERVAL, MarketError::PriceUncertain);

    // price.price is i64 with price.exponent (negative) — scale before comparing to your threshold.
    // e.g. if exponent is -8 and threshold is stored as a scaled i64 in the same units, compare directly;
    // if threshold is in human-readable USD, compute: actual_price = price.price * 10^price.exponent.
    let outcome = price.price >= ctx.accounts.market.threshold; // assumes threshold uses same exponent
    // → transition to ClaimSubmitted / dispute window, see dispute-windows.md
    Ok(())
}
```

**3. Always check the confidence interval, not just the price.** Pyth prices come with a `conf` field representing oracle uncertainty. A price near the threshold with a wide confidence band is exactly the situation an attacker will try to exploit — reject or delay resolution rather than resolving on a price you can't trust to the precision the market needs.

**4. Enforce staleness bounds appropriate to the market's time sensitivity.** A market resolving on a daily close can tolerate looser staleness than one resolving on second-by-second crossings.

**5. Route through the same dispute window as Merkle-resolved markets.** Don't treat oracle resolution as "more trusted, so skip the window" — oracle glitches happen too (depegs, feed outages, stale aggregator reports).

## Using This as a Fallback for Merkle Markets

If a market is normally resolved via off-chain attestation (see `merkle-outcome-verification.md`) but the attestor fails to publish a root before some deadline, a documented fallback to an on-chain price/threshold proxy (where one exists) is far safer than leaving funds stuck indefinitely or accepting a late, unverified claim. Make the fallback condition explicit and time-boxed in the market account, not an ad hoc admin override.

## Common Mistakes

- Resolving against `get_price_unchecked()` or equivalent — always use the staleness-checked accessor.
- Hardcoding a staleness window copied from another project without checking whether it fits your market's actual time sensitivity.
- Ignoring `conf` entirely and only checking the point price.
- Allowing `resolve_price_market` to be called before `resolve_at`, letting someone pick a favorable moment.
