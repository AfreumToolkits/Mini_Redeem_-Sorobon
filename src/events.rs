use soroban_sdk::{symbol_short, Address, Env};

pub fn mint(env: &Env, admin: &Address, to: &Address, amount: i128) {
    let topics = (symbol_short!("mint"), admin, to);
    env.events().publish(topics, amount);
}

pub fn redeem(env: &Env, from: &Address, amount: i128) {
    let topics = (symbol_short!("redeem"), from);
    env.events().publish(topics, amount);
}

pub fn set_admin(env: &Env, new_admin: &Address) {
    let topics = (symbol_short!("set_admin"),);
    env.events().publish(topics, new_admin);
}

pub fn set_redeemer(env: &Env, new_redeemer: &Address) {
    let topics = (symbol_short!("set_redm"),);
    env.events().publish(topics, new_redeemer);
}

pub fn paused(env: &Env, is_paused: bool) {
    let topics = (symbol_short!("paused"),);
    env.events().publish(topics, is_paused);
}
