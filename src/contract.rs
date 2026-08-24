use soroban_sdk::{contract, contractimpl, Address, Env, String};

use crate::admin::{
    bump_instance, clear_pending_admin, has_admin, is_paused, read_admin, read_balance,
    read_decimals, read_max_supply, read_name, read_pending_admin, read_redeemer, read_symbol,
    read_total_supply, write_admin, write_balance, write_max_supply, write_metadata,
    write_paused, write_pending_admin, write_redeemer, write_total_supply,
};
use crate::errors::Error;
use crate::events;

#[contract]
pub struct MiniRedeemContract;

#[contractimpl]
impl MiniRedeemContract {
    /// One-time setup. `admin` can mint and manage the contract.
    /// `max_supply` of 0 means no cap.
    pub fn initialize(
        env: Env,
        admin: Address,
        name: String,
        symbol: String,
        decimals: u32,
        max_supply: i128,
    ) -> Result<(), Error> {
        if has_admin(&env) {
            return Err(Error::AlreadyInitialized);
        }
        if decimals > 18 {
            return Err(Error::InvalidDecimals);
        }
        if name.len() == 0 || symbol.len() == 0 {
            return Err(Error::InvalidMetadata);
        }
        if max_supply < 0 {
            return Err(Error::InvalidAmount);
        }
        admin.require_auth();
        write_admin(&env, &admin);
        write_metadata(&env, decimals, &name, &symbol);
        write_total_supply(&env, 0);
        write_max_supply(&env, max_supply);
        write_paused(&env, false);
        bump_instance(&env);
        Ok(())
    }

    /// Mint new tokens to `to`, backed 1:1 by fiat the admin has custody of
    /// off-chain. Admin-only. Fails on overflow or if it would exceed the
    /// configured max supply, instead of panicking.
    pub fn mint(env: Env, to: Address, amount: i128) -> Result<(), Error> {
        Self::require_not_paused(&env)?;
        Self::require_positive(amount)?;
        let admin = read_admin(&env);
        admin.require_auth();

        let balance = read_balance(&env, &to);
        let new_balance = balance.checked_add(amount).ok_or(Error::Overflow)?;

        let supply = read_total_supply(&env);
        let new_supply = supply.checked_add(amount).ok_or(Error::Overflow)?;

        let cap = read_max_supply(&env);
        if cap > 0 && new_supply > cap {
            return Err(Error::MaxSupplyExceeded);
        }

        write_balance(&env, &to, new_balance);
        write_total_supply(&env, new_supply);

        events::mint(&env, &admin, &to, amount);
        bump_instance(&env);
        Ok(())
    }

    /// Burn `amount` from `from`'s balance to redeem it for off-chain fiat.
    /// Requires authorization from `from`. Does not move fiat itself — the
    /// `redeem` event is the signal for an off-chain payout service.
    pub fn redeem(env: Env, from: Address, amount: i128) -> Result<(), Error> {
        Self::require_not_paused(&env)?;
        Self::require_positive(amount)?;
        from.require_auth();
        Self::burn(&env, &from, amount)
    }

    /// Redeem on behalf of `from`, callable only by the designated
    /// redeemer (e.g. a custodian service processing a payout request).
    pub fn redeem_for(env: Env, from: Address, amount: i128) -> Result<(), Error> {
        Self::require_not_paused(&env)?;
        Self::require_positive(amount)?;
        let redeemer = read_redeemer(&env);
        redeemer.require_auth();
        Self::burn(&env, &from, amount)
    }

    pub fn balance(env: Env, id: Address) -> i128 {
        read_balance(&env, &id)
    }

    pub fn total_supply(env: Env) -> i128 {
        read_total_supply(&env)
    }

    pub fn max_supply(env: Env) -> i128 {
        read_max_supply(&env)
    }

    pub fn admin(env: Env) -> Address {
        read_admin(&env)
    }

    pub fn pending_admin(env: Env) -> Option<Address> {
        read_pending_admin(&env)
    }

    /// Step 1 of a two-step admin handoff: current admin nominates a
    /// successor. The successor must separately call `accept_admin`.
    /// This prevents permanently bricking the contract by mistyping an
    /// admin address in a single-step transfer.
    pub fn transfer_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        let admin = read_admin(&env);
        admin.require_auth();
        write_pending_admin(&env, &new_admin);
        bump_instance(&env);
        Ok(())
    }

    /// Step 2: the nominated address claims adminship.
    pub fn accept_admin(env: Env) -> Result<(), Error> {
        let pending = read_pending_admin(&env).ok_or(Error::NoPendingAdmin)?;
        pending.require_auth();
        write_admin(&env, &pending);
        clear_pending_admin(&env);
        events::set_admin(&env, &pending);
        bump_instance(&env);
        Ok(())
    }

    pub fn set_redeemer(env: Env, new_redeemer: Address) -> Result<(), Error> {
        let admin = read_admin(&env);
        admin.require_auth();
        write_redeemer(&env, &new_redeemer);
        events::set_redeemer(&env, &new_redeemer);
        bump_instance(&env);
        Ok(())
    }

    pub fn set_paused(env: Env, paused: bool) -> Result<(), Error> {
        let admin = read_admin(&env);
        admin.require_auth();
        write_paused(&env, paused);
        events::paused(&env, paused);
        bump_instance(&env);
        Ok(())
    }

    pub fn is_paused(env: Env) -> bool {
        is_paused(&env)
    }

    pub fn name(env: Env) -> String {
        read_name(&env)
    }

    pub fn symbol(env: Env) -> String {
        read_symbol(&env)
    }

    pub fn decimals(env: Env) -> u32 {
        read_decimals(&env)
    }

    fn require_not_paused(env: &Env) -> Result<(), Error> {
        if is_paused(env) {
            return Err(Error::ContractPaused);
        }
        Ok(())
    }

    fn require_positive(amount: i128) -> Result<(), Error> {
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        Ok(())
    }

    fn burn(env: &Env, from: &Address, amount: i128) -> Result<(), Error> {
        let balance = read_balance(env, from);
        let new_balance = balance.checked_sub(amount).ok_or(Error::Underflow)?;
        if new_balance < 0 {
            return Err(Error::InsufficientBalance);
        }

        let supply = read_total_supply(env);
        let new_supply = supply.checked_sub(amount).ok_or(Error::Underflow)?;

        write_balance(env, from, new_balance);
        write_total_supply(env, new_supply);

        events::redeem(env, from, amount);
        bump_instance(env);
        Ok(())
    }
}
