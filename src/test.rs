#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String};

use crate::{contract::MiniRedeemContract, contract::MiniRedeemContractClient};

fn setup<'a>() -> (Env, MiniRedeemContractClient<'a>, Address) {
    setup_with_cap(0)
}

fn setup_with_cap<'a>(max_supply: i128) -> (Env, MiniRedeemContractClient<'a>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MiniRedeemContract);
    let client = MiniRedeemContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(
        &admin,
        &String::from_str(&env, "Afreum Fiat Token"),
        &String::from_str(&env, "AFX"),
        &7,
        &max_supply,
    );

    (env, client, admin)
}

#[test]
fn mint_increases_balance_and_supply() {
    let (env, client, _admin) = setup();
    let user = Address::generate(&env);

    client.mint(&user, &1_000_0000000);

    assert_eq!(client.balance(&user), 1_000_0000000);
    assert_eq!(client.total_supply(), 1_000_0000000);
}

#[test]
fn redeem_burns_from_holder() {
    let (env, client, _admin) = setup();
    let user = Address::generate(&env);

    client.mint(&user, &500_0000000);
    client.redeem(&user, &200_0000000);

    assert_eq!(client.balance(&user), 300_0000000);
    assert_eq!(client.total_supply(), 300_0000000);
}

#[test]
fn redeem_for_uses_designated_redeemer() {
    let (env, client, _admin) = setup();
    let user = Address::generate(&env);
    let redeemer = Address::generate(&env);

    client.set_redeemer(&redeemer);
    client.mint(&user, &100_0000000);
    client.redeem_for(&user, &40_0000000);

    assert_eq!(client.balance(&user), 60_0000000);
}

#[test]
fn redeem_fails_on_insufficient_balance() {
    let (env, client, _admin) = setup();
    let user = Address::generate(&env);

    client.mint(&user, &10_0000000);
    let result = client.try_redeem(&user, &20_0000000);

    assert!(result.is_err());
}

#[test]
fn mint_fails_when_paused() {
    let (env, client, _admin) = setup();
    let user = Address::generate(&env);

    client.set_paused(&true);
    let result = client.try_mint(&user, &10_0000000);

    assert!(result.is_err());
}

#[test]
fn double_initialize_fails() {
    let (env, client, admin) = setup();

    let result = client.try_initialize(
        &admin,
        &String::from_str(&env, "Afreum Fiat Token"),
        &String::from_str(&env, "AFX"),
        &7,
        &0,
    );

    assert!(result.is_err());
}

#[test]
fn zero_and_negative_amounts_rejected() {
    let (env, client, _admin) = setup();
    let user = Address::generate(&env);

    assert!(client.try_mint(&user, &0).is_err());
    assert!(client.try_mint(&user, &-5).is_err());
}

#[test]
fn invalid_decimals_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, MiniRedeemContract);
    let client = MiniRedeemContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    let result = client.try_initialize(
        &admin,
        &String::from_str(&env, "Afreum Fiat Token"),
        &String::from_str(&env, "AFX"),
        &19,
        &0,
    );

    assert!(result.is_err());
}

#[test]
fn mint_respects_max_supply_cap() {
    let (env, client, _admin) = setup_with_cap(100_0000000);
    let user = Address::generate(&env);

    client.mint(&user, &100_0000000);
    let result = client.try_mint(&user, &1);

    assert!(result.is_err());
    assert_eq!(client.total_supply(), 100_0000000);
}

#[test]
fn two_step_admin_transfer() {
    let (env, client, admin) = setup();
    let successor = Address::generate(&env);

    assert_eq!(client.admin(), admin);
    assert_eq!(client.pending_admin(), None);

    client.transfer_admin(&successor);
    assert_eq!(client.pending_admin(), Some(successor.clone()));
    // Admin doesn't change until the successor explicitly accepts.
    assert_eq!(client.admin(), admin);

    client.accept_admin();
    assert_eq!(client.admin(), successor);
    assert_eq!(client.pending_admin(), None);
}

#[test]
fn accept_admin_without_pending_fails() {
    let (_env, client, _admin) = setup();
    let result = client.try_accept_admin();
    assert!(result.is_err());
}

#[test]
fn mint_at_max_supply_succeeds() {
    let (env, client, _) = setup_with_cap(50_0000000);
    let user = Address::generate(&env);
    client.mint(&user, &50_0000000);
    assert_eq!(client.total_supply(), 50_0000000);
}

#[test]
fn mint_over_max_supply_fails() {
    let (env, client, _) = setup_with_cap(50_0000000);
    let user = Address::generate(&env);
    client.mint(&user, &50_0000000);
    let result = client.try_mint(&user, &1);
    assert!(result.is_err(), "mint over max supply should fail");
    assert_eq!(client.total_supply(), 50_0000000);
}

#[test]
fn redeem_more_than_balance_fails() {
    let (env, client, _) = setup();
    let user = Address::generate(&env);
    client.mint(&user, &10_0000000);
    let result = client.try_redeem(&user, &20_0000000);
    assert!(result.is_err(), "redeem more than balance should fail");
    assert_eq!(client.balance(&user), 10_0000000);
}

#[test]
fn mint_zero_amount_fails() {
    let (env, client, _) = setup();
    let user = Address::generate(&env);
    let result = client.try_mint(&user, &0);
    assert!(result.is_err(), "mint zero amount should fail");
}

#[test]
fn mint_negative_amount_fails() {
    let (env, client, _) = setup();
    let user = Address::generate(&env);
    let result = client.try_mint(&user, &-1);
    assert!(result.is_err(), "mint negative amount should fail");
}

#[test]
fn invariant_total_supply_matches_sum() {
    // Invariant: total_supply should equal sum of all individual balances
    let (env, client, _) = setup_with_cap(1_000_000_000);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);

    client.mint(&user1, &30_0000000);
    client.mint(&user2, &20_0000000);
    client.mint(&user3, &10_0000000);

    assert_eq!(client.total_supply(), 60_0000000);
    assert_eq!(client.balance(&user1), 30_0000000);
    assert_eq!(client.balance(&user2), 20_0000000);
    assert_eq!(client.balance(&user3), 10_0000000);
}

#[test]
fn invariant_supply_under_cap() {
    // Invariant: when max_supply is set, total_supply must not exceed it
    let cap = 100_0000000;
    let (env, client, _) = setup_with_cap(cap);
    let user = Address::generate(&env);
    client.mint(&user, &cap);

    assert_eq!(client.total_supply(), cap);

    // Attempt one more mint - should fail
    let result = client.try_mint(&user, &1);
    assert!(result.is_err());
    assert_eq!(client.total_supply(), cap);
}

#[test]
fn mint_fails_for_non_admin() {
    let (env, client, _) = setup();
    let user = Address::generate(&env);
    let result = client.try_mint(&user, &10_0000000);
    assert!(result.is_err(), "mint by non-admin should fail");
}

#[test]
fn redeem_for_fails_for_non_redeemer() {
    let (env, client, _) = setup();
    let user = Address::generate(&env);
    let bad_redeemer = Address::generate(&env);
    client.mint(&user, &100_0000000);
    let result = client.try_redeem_for(&user, &10_0000000, &bad_redeemer);
    assert!(result.is_err(), "redeem_for by non-redeemer should fail");
}

#[test]
fn set_paused_fails_for_non_admin() {
    let (env, client, _) = setup();
    let user = Address::generate(&env);
    let result = client.try_set_paused(&user, &true);
    assert!(result.is_err(), "set_paused by non-admin should fail");
}

#[test]
fn set_redeemer_fails_for_non_admin() {
    let (env, client, _) = setup();
    let user = Address::generate(&env);
    let result = client.try_set_redeemer(&user, &Address::generate(&env));
    assert!(result.is_err(), "set_redeemer by non-admin should fail");
}

#[test]
fn transfer_admin_fails_for_non_admin() {
    let (env, client, _) = setup();
    let bad_admin = Address::generate(&env);
    let result = client.try_transfer_admin(&bad_admin);
    assert!(result.is_err(), "transfer_admin by non-admin should fail");
}
