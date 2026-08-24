use soroban_sdk::{contracttype, Address};

/// How long (in ledgers) persistent instance data stays alive before it
/// must be bumped again. ~30 days assuming 5s ledgers.
pub const INSTANCE_BUMP_AMOUNT: u32 = 518_400;
pub const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - 100_800;

/// Balance entries get their own (cheaper) bump window, ~7 days.
pub const BALANCE_BUMP_AMOUNT: u32 = 120_960;
pub const BALANCE_LIFETIME_THRESHOLD: u32 = BALANCE_BUMP_AMOUNT - 20_160;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Contract admin, allowed to mint, set the redeemer, and pause/unpause.
    Admin,
    /// Two-step admin handoff: set by `transfer_admin`, cleared by
    /// `accept_admin`. Avoids permanently losing control to a typo'd address.
    PendingAdmin,
    /// Address allowed to burn on behalf of users during a redeem flow
    /// (e.g. a custodian/off-ramp service). Optional — defaults to Admin.
    Redeemer,
    /// Whether the contract is paused (mint/redeem disabled).
    Paused,
    /// Per-holder balance.
    Balance(Address),
    /// Running total supply, kept in sync with mint/redeem.
    TotalSupply,
    /// Optional hard cap on total supply. 0 means unlimited.
    MaxSupply,
    /// Token metadata.
    Decimals,
    Name,
    Symbol,
}
