#![no_std]

mod admin;
mod contract;
mod errors;
mod events;
mod storage_types;

pub use crate::contract::{MiniRedeemContract, MiniRedeemContractClient};
pub use crate::errors::Error;

#[cfg(test)]
mod test;
