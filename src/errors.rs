use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    ContractPaused = 4,
    InsufficientBalance = 5,
    InvalidAmount = 6,
    Overflow = 7,
    Underflow = 8,
    InvalidDecimals = 9,
    InvalidMetadata = 10,
    MaxSupplyExceeded = 11,
    NoPendingAdmin = 12,
    NotPendingAdmin = 13,
}
