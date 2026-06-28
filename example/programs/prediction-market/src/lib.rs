use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::system_instruction;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// ── Space ────────────────────────────────────────────────────────────────────
// MarketState largest variant: Disputed { Pubkey(32) + i64(8) } = 40 + 1 tag = 41
const MARKET_STATE_SIZE: usize = 41;

impl Market {
    // 8 disc + 32 authority + 32 resolver + 41 state
    // + 8 resolve_at + 8 dispute_window_seconds + 32 feed_id
    // + 8 total_staked + 1 market_bump + 1 vault_bump = 171; pad to 200
    pub const SPACE: usize = 200;
}

impl Position {
    // 8 + 32 market + 32 owner + 1 side + 8 amount + 1 claimed = 82
    pub const SPACE: usize = 82;
}

impl CommittedRoot {
    // 8 + 32 root + 8 batch_id + 32 resolver + 8 published_at = 88
    pub const SPACE: usize = 88;
}

impl VaultAccount {
    // 8 discriminator only — vault holds SOL in lamports, no data
    pub const SPACE: usize = 8;
}

// ── Program ──────────────────────────────────────────────────────────────────

#[program]
pub mod prediction_market {
    use super::*;

    /// Initialize a new prediction market with a designated resolver.
    pub fn create_market(
        ctx: Context<CreateMarket>,
        resolve_at: i64,
        dispute_window_seconds: i64,
        feed_id: [u8; 32],
    ) -> Result<()> {
        let market = &mut ctx.accounts.market;
        market.authority = ctx.accounts.authority.key();
        market.resolver = ctx.accounts.resolver.key();
        market.state = MarketState::Open;
        market.resolve_at = resolve_at;
        market.dispute_window_seconds = dispute_window_seconds;
        market.feed_id = feed_id;
        market.total_staked = 0;
        market.market_bump = ctx.bumps.market;
        market.vault_bump = ctx.bumps.vault;
        Ok(())
    }

    /// Take a position — SOL is escrowed in the vault PDA.
    pub fn take_position(
        ctx: Context<TakePosition>,
        side: u8,
        amount: u64,
    ) -> Result<()> {
        require!(
            matches!(ctx.accounts.market.state, MarketState::Open),
            MarketError::InvalidState
        );
        require!(amount > 0, MarketError::ZeroAmount);

        let position = &mut ctx.accounts.position;
        position.market = ctx.accounts.market.key();
        position.owner = ctx.accounts.owner.key();
        position.side = side;
        position.amount = amount;
        position.claimed = false;

        // Transfer SOL from owner to vault via system program CPI
        invoke(
            &system_instruction::transfer(
                &ctx.accounts.owner.key(),
                &ctx.accounts.vault.key(),
                amount,
            ),
            &[
                ctx.accounts.owner.to_account_info(),
                ctx.accounts.vault.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        ctx.accounts.market.total_staked = ctx.accounts.market.total_staked
            .checked_add(amount)
            .ok_or(MarketError::Overflow)?;
        Ok(())
    }

    /// Attestor publishes a Merkle root committing to a batch of outcomes.
    pub fn commit_root(
        ctx: Context<CommitRoot>,
        root: [u8; 32],
        batch_id: u64,
    ) -> Result<()> {
        require!(
            ctx.accounts.resolver.key() == ctx.accounts.market.resolver,
            MarketError::Unauthorized
        );
        let committed = &mut ctx.accounts.committed_root;
        committed.root = root;
        committed.batch_id = batch_id;
        committed.resolver = ctx.accounts.resolver.key();
        committed.published_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Submit a Merkle proof to move the market into ClaimSubmitted state.
    /// Anyone can call this — proof validity is what authorizes settlement.
    pub fn settle_market(
        ctx: Context<SettleMarket>,
        outcome: u8,
        leaf_index: u64,
        proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        let market = &mut ctx.accounts.market;
        require!(matches!(market.state, MarketState::Open), MarketError::InvalidState);
        require!(
            Clock::get()?.unix_timestamp >= market.resolve_at,
            MarketError::TooEarly
        );
        require!(proof.len() <= 20, MarketError::ProofTooLong);

        // Re-hash the leaf the same way the attestor did, then walk the proof.
        // The domain prefix "outcome_leaf" prevents second-preimage attacks.
        let leaf = hash_leaf(market.key(), outcome, leaf_index);
        let computed = verify_proof(&leaf, &proof, leaf_index as usize);
        require!(
            computed == ctx.accounts.committed_root.root,
            MarketError::InvalidProof
        );

        market.state = MarketState::ClaimSubmitted {
            claimed_at: Clock::get()?.unix_timestamp,
            claimed_outcome: outcome,
        };
        Ok(())
    }

    /// Finalize the market once the dispute window has elapsed.
    /// Anyone can call this — permissionless finalization prevents griefing.
    pub fn finalize_market(ctx: Context<FinalizeMarket>) -> Result<()> {
        let market = &mut ctx.accounts.market;
        let now = Clock::get()?.unix_timestamp;

        match market.state {
            MarketState::ClaimSubmitted { claimed_at, claimed_outcome } => {
                require!(
                    now >= claimed_at + market.dispute_window_seconds,
                    MarketError::DisputeWindowOpen
                );
                market.state = MarketState::Finalized { outcome: claimed_outcome };
            }
            _ => return Err(MarketError::InvalidState.into()),
        }
        Ok(())
    }

    /// Each winning position holder pulls their own payout (pull pattern).
    /// Prevents push-to-N DoS and gives each holder a separate claimed flag.
    pub fn claim_payout(ctx: Context<ClaimPayout>) -> Result<()> {
        let position = &mut ctx.accounts.position;
        require!(!position.claimed, MarketError::AlreadyClaimed);

        let winning_outcome = match ctx.accounts.market.state {
            MarketState::Finalized { outcome } => outcome,
            _ => return Err(MarketError::NotFinalized.into()),
        };
        require!(position.side == winning_outcome, MarketError::LosingPosition);

        let payout = position.amount;

        // Set claimed BEFORE transferring — no reentrancy path in Anchor's model,
        // but good habit if CPI callbacks are added later.
        position.claimed = true;

        // Vault is owned by this program — raw lamport transfer is safe.
        **ctx.accounts.vault.to_account_info().try_borrow_mut_lamports()? -= payout;
        **ctx.accounts.owner.try_borrow_mut_lamports()? += payout;

        Ok(())
    }
}

// ── Merkle helpers ───────────────────────────────────────────────────────────

/// Domain-separated leaf hash. The "outcome_leaf" prefix prevents an internal
/// tree node from being submitted as a valid leaf (second-preimage attack).
fn hash_leaf(market: Pubkey, outcome: u8, leaf_index: u64) -> [u8; 32] {
    keccak::hashv(&[
        b"outcome_leaf",
        market.as_ref(),
        &[outcome],
        &leaf_index.to_le_bytes(),
    ])
    .0
}

/// Walk the proof from leaf to root. Index parity determines sibling order,
/// matching standard binary Merkle tree conventions.
fn verify_proof(leaf: &[u8; 32], proof: &[[u8; 32]], mut index: usize) -> [u8; 32] {
    let mut computed = *leaf;
    for sibling in proof {
        computed = if index % 2 == 0 {
            keccak::hashv(&[&computed, sibling]).0
        } else {
            keccak::hashv(&[sibling, &computed]).0
        };
        index /= 2;
    }
    computed
}

// ── Account contexts ─────────────────────────────────────────────────────────

#[derive(Accounts)]
pub struct CreateMarket<'info> {
    #[account(
        init,
        payer = authority,
        space = Market::SPACE,
        seeds = [b"market", authority.key().as_ref()],
        bump,
    )]
    pub market: Account<'info, Market>,
    #[account(
        init,
        payer = authority,
        space = VaultAccount::SPACE,
        seeds = [b"vault", market.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, VaultAccount>,
    pub resolver: SystemAccount<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TakePosition<'info> {
    #[account(
        init,
        payer = owner,
        space = Position::SPACE,
        seeds = [b"position", market.key().as_ref(), owner.key().as_ref()],
        bump,
    )]
    pub position: Account<'info, Position>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(
        mut,
        seeds = [b"vault", market.key().as_ref()],
        bump = market.vault_bump,
    )]
    pub vault: Account<'info, VaultAccount>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(root: [u8; 32], batch_id: u64)]
pub struct CommitRoot<'info> {
    #[account(
        init,
        payer = resolver,
        space = CommittedRoot::SPACE,
        seeds = [b"root", market.key().as_ref(), &batch_id.to_le_bytes()],
        bump,
    )]
    pub committed_root: Account<'info, CommittedRoot>,
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub resolver: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SettleMarket<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    pub committed_root: Account<'info, CommittedRoot>,
    pub settler: Signer<'info>,
}

#[derive(Accounts)]
pub struct FinalizeMarket<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    pub finalizer: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimPayout<'info> {
    #[account(
        mut,
        has_one = market,
        has_one = owner,
    )]
    pub position: Account<'info, Position>,
    pub market: Account<'info, Market>,
    #[account(
        mut,
        seeds = [b"vault", market.key().as_ref()],
        bump = market.vault_bump,
    )]
    pub vault: Account<'info, VaultAccount>,
    #[account(mut)]
    pub owner: Signer<'info>,
}

// ── Account structs ──────────────────────────────────────────────────────────

#[account]
pub struct Market {
    pub authority: Pubkey,
    pub resolver: Pubkey,
    pub state: MarketState,
    pub resolve_at: i64,
    pub dispute_window_seconds: i64,
    /// Pyth price feed ID ([u8; 32]) — zero-filled for Merkle-only markets.
    pub feed_id: [u8; 32],
    pub total_staked: u64,
    pub market_bump: u8,
    pub vault_bump: u8,
}

#[account]
pub struct Position {
    pub market: Pubkey,
    pub owner: Pubkey,
    pub side: u8,
    pub amount: u64,
    pub claimed: bool,
}

#[account]
pub struct CommittedRoot {
    pub root: [u8; 32],
    pub batch_id: u64,
    pub resolver: Pubkey,
    pub published_at: i64,
}

/// Empty account — vault holds SOL in lamports only, no data fields.
#[account]
pub struct VaultAccount {}

/// Explicit state machine for the market lifecycle.
/// Defined once here; referenced in settlement-account-patterns.md and dispute-windows.md.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum MarketState {
    Open,
    ClaimSubmitted { claimed_at: i64, claimed_outcome: u8 },
    Finalized { outcome: u8 },
    Disputed { challenger: Pubkey, raised_at: i64 },
    Cancelled,
}

// ── Errors ───────────────────────────────────────────────────────────────────

#[error_code]
pub enum MarketError {
    #[msg("Merkle proof does not match the committed root")]
    InvalidProof,
    #[msg("Market is not finalized — dispute window may still be open")]
    NotFinalized,
    #[msg("Position has already been claimed")]
    AlreadyClaimed,
    #[msg("Dispute window is still open — call finalize_market later")]
    DisputeWindowOpen,
    #[msg("Too early to resolve — resolve_at has not passed")]
    TooEarly,
    #[msg("Invalid market state for this instruction")]
    InvalidState,
    #[msg("Signer is not the authorized resolver")]
    Unauthorized,
    #[msg("Proof depth exceeds maximum of 20")]
    ProofTooLong,
    #[msg("This position backed the losing outcome")]
    LosingPosition,
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    #[msg("Arithmetic overflow")]
    Overflow,
}
