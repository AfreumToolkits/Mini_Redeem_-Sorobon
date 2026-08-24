# afreum-mini-redeem

A minimal Soroban smart contract for the Stellar blockchain that mints and
redeems a fiat-backed token. Written from scratch as a lightweight companion
to `Afreum_Fiat_Mint_Redeem_Contract`, intended for the `AfreumToolkits` org.

## What it does

- **Mint** — the contract admin mints tokens 1:1 against fiat received
  off-chain (e.g. a bank deposit confirmed by Afreum's backend).
- **Redeem** — a token holder burns their tokens to signal an off-chain
  fiat payout. A `redeem` event is emitted for the off-chain payout service
  to listen for and act on.
- **Redeem-for** — an optional designated `redeemer` address (a custodian
  or automated payout service) can process redemptions on a user's behalf,
  separate from the admin key.
- **Pause switch** — admin can pause mint/redeem in an emergency.

This contract intentionally does **not** move real fiat — it only tracks
the on-chain token leg. The mint/redeem events are the integration point
for whatever off-chain banking rail you're using.

## Contract interface

| Function | Auth required | Description |
|---|---|---|
| `initialize(admin, name, symbol, decimals, max_supply)` | `admin` | One-time setup. `max_supply` of `0` = uncapped |
| `mint(to, amount)` | `admin` | Mint tokens to `to`; checked arithmetic, cap-enforced |
| `redeem(from, amount)` | `from` | Holder burns their own tokens |
| `redeem_for(from, amount)` | `redeemer` | Custodian burns on holder's behalf |
| `balance(id)` | — | Read balance |
| `total_supply()` / `max_supply()` | — | Read supply state |
| `admin()` / `pending_admin()` | — | Read current/pending admin |
| `transfer_admin(new_admin)` | `admin` | Step 1: nominate a successor admin |
| `accept_admin()` | pending admin | Step 2: successor claims adminship |
| `set_redeemer(new_redeemer)` | `admin` | Set/rotate redeemer key |
| `set_paused(bool)` | `admin` | Pause/unpause mint & redeem |
| `is_paused()` | — | Read pause state |
| `name()` / `symbol()` / `decimals()` | — | Token metadata |

## Robustness changes from the first draft

- **Checked arithmetic everywhere.** `mint`/`redeem` used raw `+`/`-` before,
  which panics (aborts the whole transaction ungracefully) on overflow.
  Now every balance/supply update goes through `checked_add`/`checked_sub`
  and returns a typed `Error` instead.
- **Two-step admin transfer.** A single `set_admin(new_admin)` call is one
  typo away from permanently bricking the contract (nobody can call
  admin-only functions again). `transfer_admin` + `accept_admin` requires
  the successor to prove control of their key before the handoff finalizes.
- **Optional max-supply cap**, enforced at mint time — useful if you want
  an on-chain ceiling independent of off-chain reserve attestations.
- **Input validation on `initialize`**: decimals capped at 18, name/symbol
  can't be empty, negative max_supply rejected — bad init calls fail fast
  with a clear error instead of leaving a half-configured contract.

## Project layout

```
src/
  lib.rs            crate root, module wiring
  contract.rs        public contract functions
  admin.rs           storage read/write helpers
  storage_types.rs   DataKey enum + TTL constants
  errors.rs           contract error codes
  events.rs           event publishing helpers
  test.rs             unit tests (cfg(test))
.github/workflows/ci.yml   build + test on push/PR
```

## Building & testing

Requires the Rust toolchain with the `wasm32-unknown-unknown` target and
the Soroban CLI.

```bash
rustup target add wasm32-unknown-unknown

# run unit tests
cargo test

# build the optimized wasm binary
cargo build --target wasm32-unknown-unknown --release

# optional: install the Soroban CLI for deploys
cargo install --locked soroban-cli
```

## Deploying (testnet example)

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/afreum_mini_redeem.wasm \
  --source <your-key> \
  --network testnet

soroban contract invoke \
  --id <contract-id> \
  --source <admin-key> \
  --network testnet \
  -- initialize \
  --admin <admin-address> \
  --name "Afreum Fiat Token" \
  --symbol AFX \
  --decimals 7
```

## Making it more efficient

Soroban fees are driven by **resource consumption**, not gas-per-opcode like
EVM: CPU instructions, memory, ledger I/O (reads/writes), and the size of
the transaction's storage "footprint." Concretely, for a contract like this:

- **Storage tiering matters more than code speed.** Instance storage
  (admin, paused, supply, metadata) is loaded once per invocation regardless
  of how many instance keys you touch, so it's cheap to read. Persistent
  storage (per-user balances) is billed per key touched — this contract
  already keeps balances there and everything else in instance storage,
  which is the right split.
- **Don't bump TTL on reads.** The original draft called `extend_ttl` on
  every `balance()` read. That adds a write to the transaction footprint
  purely to keep an entry alive, even for a read-only query that shouldn't
  cost anything. This version only bumps TTL when a balance is actually
  written (mint/redeem), since a write already proves the entry is in
  active use — reads stay free of storage-footprint cost.
- **Batch instance writes instead of scattering them.** Every `env.storage().instance().set(...)`
  call adds to the footprint. Where possible, group related state (e.g.
  pack `paused` + `decimals` into one small struct) instead of separate
  keys, if you're optimizing hard for footprint size — this contract keeps
  them separate for readability since the win is marginal at this scale.
- **Minimize event payload size.** Events are billed by size. Keep event
  data to what a listener actually needs (address + amount), not full
  struct dumps.
- **Wasm binary size affects deploy cost and, moderately, invocation cost.**
  The `Cargo.toml` release profile already sets `opt-level = "z"`,
  `lto = true`, `strip = "symbols"`, and `panic = "abort"` — these shrink
  the compiled wasm significantly versus defaults.
- **Avoid unnecessary `Address.clone()` calls** in hot paths — small in Rust
  terms, but each clone of an `Address`/`String` in Soroban's host-object
  model is a host-function round trip, not a free stack copy.
- **Simulate before submitting.** Soroban RPC's `simulateTransaction` returns
  the exact resource footprint and fee before you submit — always simulate
  in your client/backend rather than hardcoding a fee guess, so you're not
  overpaying on quiet calls or under-paying (and failing) on writes to
  fresh storage keys.
- **For high-volume redemption flows**, consider whether `redeem_for` calls
  can be batched into fewer transactions (e.g. netting multiple small
  redemptions server-side before submitting one on-chain burn) — each
  transaction has fixed overhead independent of the amount moved.

## Security notes

- This is a from-scratch reference implementation, not audited. Review
  before mainnet use.
- The admin key can mint arbitrary amounts — pair with off-chain proof of
  fiat reserves and multi-sig in production.
- `redeem_for` trusts the `redeemer` address entirely; make sure that key
  is only held by a hardened backend service.

## License

Apache-2.0
