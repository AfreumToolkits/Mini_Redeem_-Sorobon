use soroban_sdk::{Address, Env, String};

use crate::storage_types::{
    DataKey, BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT,
    INSTANCE_LIFETIME_THRESHOLD,
};

pub fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

/// Only called on writes, not reads. Reading a balance is by far the more
/// common operation (every mint/redeem call plus any client-side query),
/// so extending TTL on every read wastes ledger-write budget for no benefit
/// — the entry only expires if nobody *writes* to it for the full window,
/// and a write always happens whenever the balance actually matters.
fn bump_balance_ttl(env: &Env, addr: &Address) {
    env.storage().persistent().extend_ttl(
        &DataKey::Balance(addr.clone()),
        BALANCE_LIFETIME_THRESHOLD,
        BALANCE_BUMP_AMOUNT,
    );
}

pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

pub fn read_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

pub fn write_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn read_pending_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::PendingAdmin)
}

pub fn write_pending_admin(env: &Env, addr: &Address) {
    env.storage().instance().set(&DataKey::PendingAdmin, addr);
}

pub fn clear_pending_admin(env: &Env) {
    env.storage().instance().remove(&DataKey::PendingAdmin);
}

pub fn read_redeemer(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Redeemer)
        .unwrap_or_else(|| read_admin(env))
}

pub fn write_redeemer(env: &Env, redeemer: &Address) {
    env.storage().instance().set(&DataKey::Redeemer, redeemer);
}

pub fn is_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false)
}

pub fn write_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&DataKey::Paused, &paused);
}

/// Read-only; does NOT bump TTL. Use in every mint/redeem check.
pub fn read_balance(env: &Env, addr: &Address) -> i128 {
    env.storage()
        .persistent()
        .get::<_, i128>(&DataKey::Balance(addr.clone()))
        .unwrap_or(0)
}

/// Write path; bumps TTL since a write means the entry is actively in use.
pub fn write_balance(env: &Env, addr: &Address, amount: i128) {
    let key = DataKey::Balance(addr.clone());
    env.storage().persistent().set(&key, &amount);
    bump_balance_ttl(env, addr);
}

pub fn read_total_supply(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalSupply)
        .unwrap_or(0)
}

pub fn write_total_supply(env: &Env, amount: i128) {
    env.storage().instance().set(&DataKey::TotalSupply, &amount);
}

pub fn read_max_supply(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::MaxSupply)
        .unwrap_or(0)
}

pub fn write_max_supply(env: &Env, amount: i128) {
    env.storage().instance().set(&DataKey::MaxSupply, &amount);
}

pub fn write_metadata(env: &Env, decimals: u32, name: &String, symbol: &String) {
    env.storage().instance().set(&DataKey::Decimals, &decimals);
    env.storage().instance().set(&DataKey::Name, name);
    env.storage().instance().set(&DataKey::Symbol, symbol);
}

pub fn read_decimals(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::Decimals).unwrap_or(7)
}

pub fn read_name(env: &Env) -> String {
    env.storage()
        .instance()
        .get(&DataKey::Name)
        .unwrap_or_else(|| String::from_str(env, "Afreum Fiat Token"))
}

pub fn read_symbol(env: &Env) -> String {
    env.storage()
        .instance()
        .get(&DataKey::Symbol)
        .unwrap_or_else(|| String::from_str(env, "AFX"))
}
