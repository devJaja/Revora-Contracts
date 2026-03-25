# Structured Error Coverage Expansion

## Overview

Every `RevoraError` variant has a fixed numeric discriminant, a precise trigger
condition, and a dedicated test in `src/structured_error_tests.rs`.  This
document is the authoritative reference for integrators and auditors.

## Security assumptions

| Assumption | Enforcement |
|---|---|
| Auth failures (wrong signer) are host panics, not `RevoraError` | `require_auth()` panics; use `try_*` client methods to catch contract errors |
| Discriminants are immutable | Enforced by `sec_all_discriminants_are_unique_and_contiguous` test |
| All typed errors are reachable without auth panics | One test per variant in `structured_error_tests.rs` |
| `NotInitialized` surfaces before any admin-gated operation | `require_admin()` helper returns `Err(NotInitialized)` when `DataKey::Admin` absent |
| `UnauthorizedTransferAccept` is a typed error, not a panic | `accept_issuer_transfer` checks `caller == new_issuer` before `require_auth` |

## Error reference

| # | Variant | Discriminant | Trigger entrypoint | Condition |
|---|---|---|---|---|
| 1 | `InvalidRevenueShareBps` | 1 | `register_offering` | `revenue_share_bps > 10_000` (testnet mode bypasses) |
| 2 | `LimitReached` | 2 | `set_admin`, `set_platform_fee`, multisig ops | Admin already set; fee > 5 000 bps; threshold invalid |
| 3 | `ConcentrationLimitExceeded` | 3 | `report_revenue` | `enforce=true` and stored concentration > `max_bps` |
| 4 | `OfferingNotFound` | 4 | Any issuer-gated entrypoint | `(issuer, namespace, token)` not registered |
| 5 | `PeriodAlreadyDeposited` | 5 | `deposit_revenue` | `PeriodRevenue` key already set for this `period_id` |
| 6 | `NoPendingClaims` | 6 | `claim` | `share_bps == 0` or all periods already claimed |
| 7 | `HolderBlacklisted` | 7 | `claim` | Holder is in the per-offering blacklist |
| 8 | `InvalidShareBps` | 8 | `set_holder_share` | `share_bps > 10_000` |
| 9 | `PaymentTokenMismatch` | 9 | `deposit_revenue` | `payment_token` differs from offering's `payout_asset` |
| 10 | `ContractFrozen` | 10 | All state-mutating entrypoints | `DataKey::Frozen` is `true` |
| 11 | `ClaimDelayNotElapsed` | 11 | `claim` | Next period's deposit time + delay > `now` |
| 12 | `SnapshotNotEnabled` | 12 | `deposit_revenue_with_snapshot` | `SnapshotConfig` not set to `true` |
| 13 | `OutdatedSnapshot` | 13 | `deposit_revenue_with_snapshot` | `snapshot_reference <= last_snapshot_ref` |
| 14 | `PayoutAssetMismatch` | 14 | `report_revenue` | `payout_asset` param != offering's registered `payout_asset` |
| 15 | `IssuerTransferPending` | 15 | `propose_issuer_transfer` | A transfer is already pending |
| 16 | `NoTransferPending` | 16 | `cancel_issuer_transfer`, `accept_issuer_transfer` | No pending transfer exists |
| 17 | `UnauthorizedTransferAccept` | 17 | `accept_issuer_transfer` | `caller != nominated_new_issuer` |
| 18 | `MetadataTooLarge` | 18 | `set_offering_metadata` | `metadata.len() > 256` |
| 19 | `NotAuthorized` | 19 | `meta_set_holder_share`, `meta_approve_revenue_report` | Signer is not the configured delegate |
| 20 | `NotInitialized` | 20 | `set_testnet_mode` | `DataKey::Admin` absent (contract not initialized) |
| 21 | `InvalidAmount` | 21 | `report_revenue`, `deposit_revenue`, `set_min_revenue_threshold` | `amount < 0` (report) or `amount <= 0` (deposit) |
| 22 | `InvalidPeriodId` | 22 | `deposit_revenue` | `period_id == 0` |
| 23 | `SupplyCapExceeded` | 23 | `deposit_revenue` | Cumulative deposits would exceed `supply_cap` |
| 24 | `MetadataInvalidFormat` | 24 | `set_offering_metadata` | No recognised scheme prefix (`ipfs://`, `https://`, `ar://`, `sha256:`) |
| 25 | `ReportingWindowClosed` | 25 | `report_revenue` | Ledger timestamp outside configured reporting window |
| 26 | `ClaimWindowClosed` | 26 | `claim` | Ledger timestamp outside configured claiming window |
| 27 | `SignatureExpired` | 27 | `meta_set_holder_share`, `meta_approve_revenue_report` | `expiry < ledger.timestamp()` |
| 28 | `SignatureReplay` | 28 | `meta_set_holder_share`, `meta_approve_revenue_report` | Nonce already consumed for this signer |
| 29 | `SignerKeyNotRegistered` | 29 | `meta_set_holder_share`, `meta_approve_revenue_report` | No ed25519 key registered for signer |
| 30 | `ShareSumExceeded` | 30 | `set_holder_share` | Aggregate `share_bps` would exceed 10 000 |

## Implementation notes

### `NotInitialized` (discriminant 20)

Previously `set_testnet_mode` panicked with `"admin not set"` when the admin
was absent.  It now calls the `require_admin` helper which returns
`Err(NotInitialized)`, giving callers a typed error they can match on.

```rust
fn require_admin(env: &Env) -> Result<Address, RevoraError> {
    env.storage()
        .persistent()
        .get::<DataKey, Address>(&DataKey::Admin)
        .ok_or(RevoraError::NotInitialized)
}
```

### `UnauthorizedTransferAccept` (discriminant 17)

Previously `accept_issuer_transfer` called `new_issuer.require_auth()` directly,
which would panic (not return a typed error) if the wrong address signed.  The
function now takes an explicit `caller: Address` parameter and checks
`caller == new_issuer` before calling `require_auth`, returning
`Err(UnauthorizedTransferAccept)` on mismatch.

This is a **breaking API change** — all call sites must pass the accepting
address as the first argument.

### Discriminant stability guarantee

The test `sec_all_discriminants_are_unique_and_contiguous` asserts that all 30
discriminants form the contiguous range `1..=30` with no gaps or duplicates.
This test must pass on every CI run; any renumbering is a breaking change for
integrators who store or transmit raw `u32` error codes.

## Test file

`src/structured_error_tests.rs` — 31 tests (one per variant + discriminant table).

Run with:

```bash
cargo test structured_error
```
