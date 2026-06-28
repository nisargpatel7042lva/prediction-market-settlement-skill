# Settlement Account Patterns

Anchor account and PDA design for safely holding funds until resolution and releasing them once, correctly. This complements solana-dev-skill's general `programs-anchor.md` — only settlement/escrow-specific guidance lives here.

## Core Accounts

```rust
#[account]
pub struct Market {
    pub authority: Pubkey,        // who can create/cancel, NOT who can resolve
    pub resolver: Pubkey,         // attestor key or oracle feed_id authority
    pub state: MarketState,       // see dispute-windows.md
    pub resolve_at: i64,
    pub vault: Pubkey,            // PDA holding staked/wagered funds
    pub total_staked: u64,
    pub feed_id: [u8; 32],        // Pyth price feed ID — only used for price-threshold markets (see pyth-fallback-resolution.md); zero-filled for Merkle-only markets
    pub bump: u8,
}

#[account]
pub struct Position {
    pub market: Pubkey,
    pub owner: Pubkey,
    pub side: u8,                 // which outcome this position backs
    pub amount: u64,
    pub claimed: bool,            // one-shot payout flag — see below
}
```

## PDA Seeds

Derive deterministically so anyone can find the vault without an off-chain index, and so the program — not an arbitrary signer — controls fund movement:

```rust
let (vault_pda, vault_bump) = Pubkey::find_program_address(
    &[b"vault", market.key().as_ref()],
    program_id,
);
```

Use the vault PDA as the *owner* of a token account (or hold native SOL directly in the PDA's lamports), never a separate keypair-controlled account. The program should be the only signer capable of authorizing a transfer out, via `invoke_signed` with these seeds.

## Payout: One-Shot, Not Aggregate

The most common settlement bug is paying out from a shared pool without individually marking each position as claimed:

```rust
pub fn claim_payout(ctx: Context<ClaimPayout>) -> Result<()> {
    let position = &mut ctx.accounts.position;
    require!(!position.claimed, MarketError::AlreadyClaimed);
    require!(matches!(ctx.accounts.market.state, MarketState::Finalized { .. }), MarketError::NotFinalized);

    let payout = compute_payout(&ctx.accounts.market, position)?;
    // transfer `payout` from vault PDA to position.owner via invoke_signed
    position.claimed = true; // set BEFORE transfer completes in the same instruction is fine in Anchor's account model,
                              // but ensure no reentrancy path exists if you add CPI callbacks later
    Ok(())
}
```

Let each position holder pull their own payout (pull pattern) rather than the program pushing payouts to every holder in one instruction — pushing to N holders in one instruction risks hitting compute/account limits and turns one bad account into a DoS for everyone else's payout.

## Vault Drain Safety

- Never let `total_staked` accounting drift from the vault's actual balance — recompute or assert invariants (`vault.balance >= sum of unclaimed positions`) in tests, not just in production logic.
- Cancellation/refund paths need the same one-shot-claim discipline as normal payout paths — a "refund everyone" instruction is just as exploitable as a payout instruction if it lacks a claimed-flag check.
- If using Token-2022 with transfer hooks or extensions, verify the vault's token account is compatible with whatever extensions the staked mint uses (transfer fees, confidential transfers) before assuming a simple `transfer_checked` is sufficient.

## Common Mistakes

- Authority key (market creator/admin) also being trusted as the resolver — keep these separate so a single compromised key can't both create favorable markets and resolve them in its own favor.
- Missing a `bump` validation (`#[account(seeds = [...], bump = market.bump)]`) on the vault PDA, allowing an attacker to pass a different PDA that happens to satisfy a looser constraint.
- No explicit account closing / rent reclaim path once a market is fully settled and claimed — left-open accounts are minor but add up across many markets.
