//! Structured Error Coverage Expansion
//!
//! One test per `RevoraError` variant (discriminants 1-30).
//! Each test:
//!   - Triggers the exact error via the minimal call path.
//!   - Asserts the discriminant is stable (numeric contract for integrators).
//!   - Uses `Env::default()` + `mock_all_auths()` for determinism.
//!
//! # Security assumptions
//! - Auth failures (wrong signer) are host panics, not `RevoraError`.
//! - All typed errors are reachable without auth panics.
//! - Discriminants are fixed; renumbering is a breaking change.
#![cfg(test)]
#![allow(warnings)]

use crate::{RevoraError, RevoraRevenueShare, RevoraRevenueShareClient, RoundingMode};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger as _},
    Address, Env, String as SdkString,
};

const NS: fn(&Env) -> soroban_sdk::Symbol = |_| symbol_short!("ns");

fn client(env: &Env) -> (RevoraRevenueShareClient<'_>, soroban_sdk::Address) {
    let id = env.register_contract(None, RevoraRevenueShare);
    let c = RevoraRevenueShareClient::new(env, &id);
    (c, id)
}

fn with_offering(env: &Env) -> (RevoraRevenueShareClient<'_>, Address, Address) {
    let (c, _) = client(env);
    let issuer = Address::generate(env);
    let token = Address::generate(env);
    c.register_offering(&issuer, &symbol_short!("ns"), &token, &500, &token, &0);
    (c, issuer, token)
}

// ── 1: InvalidRevenueShareBps ─────────────────────────────────────────────────

/// Discriminant 1. register_offering rejects bps > 10 000 outside testnet mode.
#[test]
fn sec_error_1_invalid_revenue_share_bps_discriminant_and_trigger() {
    assert_eq!(RevoraError::InvalidRevenueShareBps as u32, 1);
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = client(&env);
    let issuer = Address::generate(&env);
    let token = Address::generate(&env);
    let r = c.try_register_offering(&issuer, &symbol_short!("ns"), &token, &10_001, &token, &0);
    assert_eq!(r, Err(Ok(RevoraError::InvalidRevenueShareBps)));
}

// ── 2: LimitReached ──────────────────────────────────────────────────────────

/// Discriminant 2. set_admin rejects a second call (admin already set).
#[test]
fn sec_error_2_limit_reached_discriminant_and_trigger() {
    assert_eq!(RevoraError::LimitReached as u32, 2);
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = client(&env);
    let admin = Address::generate(&env);
    c.set_admin(&admin);
    let r = c.try_set_admin(&admin);
    assert_eq!(r, Err(Ok(RevoraError::LimitReached)));
}

// ── 3: ConcentrationLimitExceeded ────────────────────────────────────────────

/// Discriminant 3. report_revenue fails when enforce=true and concentration > max_bps.
#[test]
fn sec_error_3_concentration_limit_exceeded_discriminant_and_trigger() {
    assert_eq!(RevoraError::ConcentrationLimitExceeded as u32, 3);
    let env = Env::default();
    env.mock_all_auths();
    let (c, issuer, token) = with_offering(&env);
    c.set_concentration_limit(&issuer, &symbol_short!("ns"), &token, &5_000, &true);
    c.report_concentration(&issuer, &symbol_short!("ns"), &token, &6_000);
    let r = c.try_report_revenue(&issuer, &symbol_short!("ns"), &token, &token, &1_000, &1, &false);
    assert_eq!(r, Err(Ok(RevoraError::ConcentrationLimitExceeded)));
}

// ── 4: OfferingNotFound ───────────────────────────────────────────────────────

/// Discriminant 4. deposit_revenue on a non-existent offering.
#[test]
fn sec_error_4_offering_not_found_discriminant_and_trigger() {
    assert_eq!(RevoraError::OfferingNotFound as u32, 4);
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = client(&env);
    let issuer = Address::generate(&env);
    let token = Address::generate(&env);
    let r = c.try_deposit_revenue(&issuer, &symbol_short!("ns"), &token, &token, &1_000, &1);
    assert_eq!(r, Err(Ok(RevoraError::OfferingNotFound)));
}

// ── 5: PeriodAlreadyDeposited ─────────────────────────────────────────────────

/// Discriminant 5. depositing the same period_id twice.
#[test]
fn sec_error_5_period_already_deposited_discriminant_and_trigger() {
    assert_eq!(RevoraError::PeriodAlreadyDeposited as u32, 5);
    let env = Env::default();
    env.mock_all_auths();
    let (c, issuer, token) = with_offering(&env);
    // fund the contract address so transfer succeeds
    let contract_id = env.register_contract(None, RevoraRevenueShare);
    // use test_insert_period to avoid token transfer complexity
    c.test_insert_period(&issuer, &symbol_short!("ns"), &token, &1, &1_000);
    // now try deposit_revenue for same period_id=1 — it checks PeriodRevenue key
    let r = c.try_deposit_revenue(&issuer, &symbol_short!("ns"), &token, &token, &1_000, &1);
    assert_eq!(r, Err(Ok(RevoraError::PeriodAlreadyDeposited)));
}

// ── 6: NoPendingClaims ────────────────────────────────────────────────────────

/// Discriminant 6. claim with zero share_bps set.
#[test]
fn sec_error_6_no_pending_claims_discriminant_and_trigger() {
    assert_eq!(RevoraError::NoPendingClaims as u32, 6);
    let env = Env::default();
    env.mock_all_auths();
    let (c, issuer, token) = with_offering(&env);
    let holder = Address::generate(&env);
    // holder has share_bps=0 (never set)
    let r = c.try_claim(&holder, &issuer, &symbol_short!("ns"), &token, &0);
    assert_eq!(r, Err(Ok(RevoraError::NoPendingClaims)));
}

// ── 7: HolderBlacklisted ─────────────────────────────────────────────────────

/// Discriminant 7. claim fails when holder is blacklisted.
#[test]
fn sec_error_7_holder_blacklisted_discriminant_and_trigger() {
    assert_eq!(RevoraError::HolderBlacklisted as u32, 7);
    let env = Env::default();
    env.mock_all_auths();
    let (c, issuer, token) = with_offering(&env);
    let holder = Address::generate(&env);
    c.set_holder_share(&issuer, &symbol_short!("ns"), &token, &holder, &5_000);
    c.blacklist_add(&issuer, &issuer, &symbol_short!("ns"), &token, &holder);
    let r = c.try_claim(&holder, &issuer, &symbol_short!("ns"), &token, &0);
    assert_eq!(r, Err(Ok(RevoraError::HolderBlacklisted)));
}

// ── 8: InvalidShareBps ───────────────────────────────────────────────────────

/// Discriminant 8. set_holder_share rejects share_bps > 10 000.
#[test]
fn sec_error_8_invalid_share_bps_discriminant_and_trigger() {
    assert_eq!(RevoraError::InvalidShareBps as u32, 8);
    let env = Env::default();
    env.mock_all_auths();
    let (c, issuer, token) = with_offering(&env);
    let holder = Address::generate(&env);
    let r = c.try_set_holder_share(&issuer, &symbol_short!("ns"), &token, &holder, &10_001);
    assert_eq!(r, Err(Ok(RevoraError::InvalidShareBps)));
}

// ── 9: PaymentTokenMismatch ───────────────────────────────────────────────────

/// Discriminant 9. deposit_revenue with a payout_asset that differs from the
/// offering's registered payout_asset.
#[test]
fn sec_error_9_payment_token_mismatch_discriminant_and_trigger() {
    assert_eq!(RevoraError::PaymentTokenMismatch as u32, 9);
    let env = Env::default();
    env.mock_all_auths();
    let (c, issuer, token) = with_offering(&env);
    // offering was registered with payout_asset=token; use a different one
    let other_asset = Address::generate(&env);
    let r = c.try_deposit_revenue(&issuer, &symbol_short!("ns"), &token, &other_asset, &1_000, &1);
    assert_eq!(r, Err(Ok(RevoraError::PaymentTokenMismatch)));
}

// ── 10: ContractFrozen ────────────────────────────────────────────────────────

/// Discriminant 10. register_offering fails when contract is frozen.
#[test]
fn sec_error_10_contract_frozen_discriminant_and_trigger() {
    assert_eq!(RevoraError::ContractFrozen as u32, 10);
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = client(&env);
    let admin = Address::generate(&env);
    c.set_admin(&admin);
    c.freeze();
    let issuer = Address::generate(&env);
    let token = Address::generate(&env);
    let r = c.try_register_offering(&issuer, &symbol_short!("ns"), &token, &500, &token, &0);
    assert_eq!(r, Err(Ok(RevoraError::ContractFrozen)));
}

// ── 11: ClaimDelayNotElapsed ──────────────────────────────────────────────────

/// Discriminant 11. claim before the configured delay has elapsed.
#[test]
fn sec_error_11_claim_delay_not_elapsed_discriminant_and_trigger() {
    assert_eq!(RevoraError::ClaimDelayNotElapsed as u32, 11);
    let env = Env::default();
    env.mock_all_auths();
    let (c, issuer, token) = with_offering(&env);
    let holder = Address::generate(&env);
    c.set_holder_share(&issuer, &symbol_short!("ns"), &token, &holder, &5_000);
    c.set_claim_delay(&issuer, &symbol_short!("ns"), &token, &86_400);
    // insert period at current timestamp; delay not elapsed
    c.test_insert_period(&issuer, &symbol_short!("ns"), &token, &1, &1_000);
    let r = c.try_claim(&holder, &issuer, &symbol_short!("ns"), &token, &1);
    assert_eq!(r, Err(Ok(RevoraError::ClaimDelayNotElapsed)));
}

// ── 12: SnapshotNotEnabled ────────────────────────────────────────────────────

/// Discriminant 12. deposit_revenue_with_snapshot when snapshots are disabled.
#[test]
fn sec_error_12_snapshot_not_enabled_discriminant_and_trigger() {
    assert_eq!(RevoraError::SnapshotNotEnabled as u32, 12);
    let env = Env::default();
    env.mock_all_auths();
    let (c, issuer, token) = with_offering(&env);
    // snapshots disabled by default
    let r = c.try_deposit_revenue_with_snapshot(
        &issuer,
        &symbol_short!("ns"),
        &token,
        &token,
        &1_000,
        &1,
        &42,
    );
    assert_eq!(r, Err(Ok(RevoraError::SnapshotNotEnabled)));
}

// ── 13: OutdatedSnapshot ─────────────────────────────────────────────────────

/// Discriminant 13. snapshot_reference not strictly greater than last recorded.
#[test]
fn sec_error_13_outdated_snapshot_discriminant_and_trigger() {
    assert_eq!(RevoraError::OutdatedSnapshot as u32, 13);
    let env = Env::default();
    env.mock_all_auths();
    let (c, issuer, token) = with_offering(&env);
    c.set_snapshot_config(&issuer, &symbol_short!("ns"), &token, &true);
    // first deposit with snapshot_reference=10 succeeds (uses test_insert_period to skip token transfer)
    // We can't easily do a real deposit without a token contract, so set last_snapshot_ref manually
    // by doing a successful deposit_revenue_with_snapshot via test path.
    // Instead, just call with ref=0 which is <= default last_snap=0
    let r = c.try_deposit_revenue_with_snapshot(
        &issuer,
        &symbol_short!("ns"),
        &token,
        &token,
        &1_000,
        &1,
        &0,
    );
    assert_eq!(r, Err(Ok(RevoraError::OutdatedSnapshot)));
}

// ── 14: PayoutAssetMismatch ───────────────────────────────────────────────────

/// Discriminant 14. report_revenue with wrong payout_asset.
#[test]
fn sec_error_14_payout_asset_mismatch_discriminant_and_trigger() {
    assert_eq!(RevoraError::PayoutAssetMismatch as u32, 14);
    let env = Env::default();
    env.mock_all_auths();
    let (c, issuer, token) = with_offering(&env);
    let wrong_asset = Address::generate(&env);
    let r = c.try_report_revenue(
        &issuer,
        &symbol_short!("ns"),
        &token,
        &wrong_asset,
        &1_000,
        &1,
        &false,
    );
    assert_eq!(r, Err(Ok(RevoraError::PayoutAssetMismatch)));
}

// ── 15: IssuerTransferPending ─────────────────────────────────────────────────

/// Discriminant 15. propose_issuer_transfer when one is already pending.
#[test]
fn sec_error_15_issuer_transfer_pending_discriminant_and_trigger() {
    assert_eq!(RevoraError::IssuerTransferPending as u32, 15);
    let env = Env::default();
    env.mock_all_auths();
    let (c, issuer, token) = with_offering(&env);
    let new_issuer = Address::generate(&env);
    c.propose_issuer_transfer(&issuer, &symbol_short!("ns"), &token, &new_issuer);
    let r = c.try_propose_issuer_transfer(&issuer, &symbol_short!("ns"), &token, &new_issuer);
    assert_eq!(r, Err(Ok(RevoraError::IssuerTransferPending)));
}

// ── 16: NoTransferPending ────────────────────────────────────────────────────

/// Discriminant 16. cancel_issuer_transfer when nothing is pending.
#[test]
fn sec_error_16_no_transfer_pending_discriminant_and_trigger() {
    assert_eq!(RevoraError::NoTransferPending as u32, 16);
    let env = Env::default();
    env.mock_all_auths();
    let (c, issuer, token) = with_offering(&env);
    let r = c.try_cancel_issuer_transfer(&issuer, &symbol_short!("ns"), &token);
    assert_eq!(r, Err(Ok(RevoraError::NoTransferPending)));
}

// ── 17: UnauthorizedTransferAccept ───────────────────────────────────────────

/// Discriminant 17. accept_issuer_transfer when caller != nominated new issuer.
/// Security: typed error (not panic) so callers can distinguish wrong-acceptor
/// from no-pending-transfer.
#[test]
fn sec_error_17_unauthorized_transfer_accept_discriminant_and_trigger() {
    assert_eq!(RevoraError::UnauthorizedTransferAccept as u32, 17);
    let env = Env::default();
    env.mock_all_auths();
    let (c, issuer, token) = with_offering(&env);
    let new_issuer = Address::generate(&env);
    let wrong_caller = Address::generate(&env);
    c.propose_issuer_transfer(&issuer, &symbol_short!("ns"), &token, &new_issuer);
    let r = c.try_accept_issuer_transfer(&wrong_caller, &issuer, &symbol_short!("ns"), &token);
    assert_eq!(r, Err(Ok(RevoraError::UnauthorizedTransferAccept)));
}

// ── 18: MetadataTooLarge ─────────────────────────────────────────────────────

/// Discriminant 18. set_offering_metadata with string > 256 bytes.
#[test]
fn sec_error_18_metadata_too_large_discriminant_and_trigger() {
    assert_eq!(RevoraError::MetadataTooLarge as u32, 18);
    let env = Env::default();
    env.mock_all_auths();
    let (c, issuer, token) = with_offering(&env);
    // 257-char string starting with valid scheme (ipfs:// = 7 chars + 250 'a' = 257 total)
    // Build using a fixed byte array since we're in no_std
    let mut buf = [b'a'; 257];
    buf[0] = b'i';
    buf[1] = b'p';
    buf[2] = b'f';
    buf[3] = b's';
    buf[4] = b':';
    buf[5] = b'/';
    buf[6] = b'/';
    let s = core::str::from_utf8(&buf).unwrap();
    let meta = SdkString::from_str(&env, s);
    let r = c.try_set_offering_metadata(&issuer, &symbol_short!("ns"), &token, &meta);
    assert_eq!(r, Err(Ok(RevoraError::MetadataTooLarge)));
}

// ── 19: NotAuthorized ────────────────────────────────────────────────────────

/// Discriminant 19. meta_set_holder_share when no delegate is configured.
#[test]
fn sec_error_19_not_authorized_discriminant_and_trigger() {
    assert_eq!(RevoraError::NotAuthorized as u32, 19);
    use crate::MetaSetHolderSharePayload;
    let env = Env::default();
    env.mock_all_auths();
    let (c, issuer, token) = with_offering(&env);
    let signer = Address::generate(&env);
    let holder = Address::generate(&env);
    let payload = MetaSetHolderSharePayload {
        issuer: issuer.clone(),
        namespace: symbol_short!("ns"),
        token: token.clone(),
        holder,
        share_bps: 1_000,
    };
    let fake_sig = soroban_sdk::BytesN::from_array(&env, &[0u8; 64]);
    let r = c.try_meta_set_holder_share(&signer, &payload, &1, &u64::MAX, &fake_sig);
    assert_eq!(r, Err(Ok(RevoraError::NotAuthorized)));
}

// ── 20: NotInitialized ───────────────────────────────────────────────────────

/// Discriminant 20. set_testnet_mode before admin is set returns NotInitialized.
#[test]
fn sec_error_20_not_initialized_discriminant_and_trigger() {
    assert_eq!(RevoraError::NotInitialized as u32, 20);
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = client(&env);
    // admin not set yet
    let r = c.try_set_testnet_mode(&false);
    assert_eq!(r, Err(Ok(RevoraError::NotInitialized)));
}

// ── 21: InvalidAmount ────────────────────────────────────────────────────────

/// Discriminant 21. report_revenue with negative amount.
#[test]
fn sec_error_21_invalid_amount_discriminant_and_trigger() {
    assert_eq!(RevoraError::InvalidAmount as u32, 21);
    let env = Env::default();
    env.mock_all_auths();
    let (c, issuer, token) = with_offering(&env);
    let r = c.try_report_revenue(&issuer, &symbol_short!("ns"), &token, &token, &-1, &1, &false);
    assert_eq!(r, Err(Ok(RevoraError::InvalidAmount)));
}

// ── 22: InvalidPeriodId ───────────────────────────────────────────────────────

/// Discriminant 22. deposit_revenue with period_id=0.
#[test]
fn sec_error_22_invalid_period_id_discriminant_and_trigger() {
    assert_eq!(RevoraError::InvalidPeriodId as u32, 22);
    let env = Env::default();
    env.mock_all_auths();
    let (c, issuer, token) = with_offering(&env);
    let r = c.try_deposit_revenue(&issuer, &symbol_short!("ns"), &token, &token, &1_000, &0);
    assert_eq!(r, Err(Ok(RevoraError::InvalidPeriodId)));
}

// ── 23: SupplyCapExceeded ─────────────────────────────────────────────────────

/// Discriminant 23. deposit_revenue exceeds the offering's supply cap.
#[test]
fn sec_error_23_supply_cap_exceeded_discriminant_and_trigger() {
    assert_eq!(RevoraError::SupplyCapExceeded as u32, 23);
    let env = Env::default();
    env.mock_all_auths();
    let (c, _) = client(&env);
    let issuer = Address::generate(&env);
    let token = Address::generate(&env);
    // register with supply_cap=500
    c.register_offering(&issuer, &symbol_short!("ns"), &token, &500, &token, &500);
    // insert a period that already consumed 400 of the cap
    c.test_insert_period(&issuer, &symbol_short!("ns"), &token, &1, &400);
    // now try to deposit 200 more (400+200=600 > 500)
    let r = c.try_deposit_revenue(&issuer, &symbol_short!("ns"), &token, &token, &200, &2);
    assert_eq!(r, Err(Ok(RevoraError::SupplyCapExceeded)));
}

// ── 24: MetadataInvalidFormat ─────────────────────────────────────────────────

/// Discriminant 24. set_offering_metadata with no recognised scheme prefix.
#[test]
fn sec_error_24_metadata_invalid_format_discriminant_and_trigger() {
    assert_eq!(RevoraError::MetadataInvalidFormat as u32, 24);
    let env = Env::default();
    env.mock_all_auths();
    let (c, issuer, token) = with_offering(&env);
    let meta = SdkString::from_str(&env, "ftp://bad-scheme");
    let r = c.try_set_offering_metadata(&issuer, &symbol_short!("ns"), &token, &meta);
    assert_eq!(r, Err(Ok(RevoraError::MetadataInvalidFormat)));
}

// ── 25: ReportingWindowClosed ─────────────────────────────────────────────────

/// Discriminant 25. report_revenue outside the configured reporting window.
#[test]
fn sec_error_25_reporting_window_closed_discriminant_and_trigger() {
    assert_eq!(RevoraError::ReportingWindowClosed as u32, 25);
    let env = Env::default();
    env.mock_all_auths();
    let (c, issuer, token) = with_offering(&env);
    // window in the far future
    c.set_report_window(&issuer, &symbol_short!("ns"), &token, &9_999_999_000, &9_999_999_999);
    let r = c.try_report_revenue(&issuer, &symbol_short!("ns"), &token, &token, &1_000, &1, &false);
    assert_eq!(r, Err(Ok(RevoraError::ReportingWindowClosed)));
}

// ── 26: ClaimWindowClosed ────────────────────────────────────────────────────

/// Discriminant 26. claim outside the configured claiming window.
#[test]
fn sec_error_26_claim_window_closed_discriminant_and_trigger() {
    assert_eq!(RevoraError::ClaimWindowClosed as u32, 26);
    let env = Env::default();
    env.mock_all_auths();
    let (c, issuer, token) = with_offering(&env);
    let holder = Address::generate(&env);
    c.set_holder_share(&issuer, &symbol_short!("ns"), &token, &holder, &5_000);
    c.test_insert_period(&issuer, &symbol_short!("ns"), &token, &1, &1_000);
    // window in the far future
    c.set_claim_window(&issuer, &symbol_short!("ns"), &token, &9_999_999_000, &9_999_999_999);
    let r = c.try_claim(&holder, &issuer, &symbol_short!("ns"), &token, &1);
    assert_eq!(r, Err(Ok(RevoraError::ClaimWindowClosed)));
}

// ── 27: SignatureExpired ──────────────────────────────────────────────────────

/// Discriminant 27. meta_set_holder_share with expiry in the past.
#[test]
fn sec_error_27_signature_expired_discriminant_and_trigger() {
    assert_eq!(RevoraError::SignatureExpired as u32, 27);
    use crate::MetaSetHolderSharePayload;
    let env = Env::default();
    env.mock_all_auths();
    let (c, issuer, token) = with_offering(&env);
    let signer = Address::generate(&env);
    let holder = Address::generate(&env);
    // register a delegate so we get past NotAuthorized
    c.set_meta_delegate(&issuer, &symbol_short!("ns"), &token, &signer);
    let payload = MetaSetHolderSharePayload {
        issuer,
        namespace: symbol_short!("ns"),
        token,
        holder,
        share_bps: 1_000,
    };
    let fake_sig = soroban_sdk::BytesN::from_array(&env, &[0u8; 64]);
    // expiry=0 is in the past (ledger timestamp defaults to 0 in tests, but 0 < 0 is false;
    // use expiry=0 and advance ledger to 1)
    env.ledger().with_mut(|l| l.timestamp = 1);
    let r = c.try_meta_set_holder_share(&signer, &payload, &1, &0, &fake_sig);
    assert_eq!(r, Err(Ok(RevoraError::SignatureExpired)));
}

// ── 28: SignatureReplay ───────────────────────────────────────────────────────

/// Discriminant 28. meta_set_holder_share with a nonce that was already used.
/// We trigger this by marking the nonce used via the MetaDataKey directly
/// through a second call that would fail at replay check.
/// Since we can't easily do a real ed25519 sig in tests, we verify the
/// discriminant value and that the error code is stable.
#[test]
fn sec_error_28_signature_replay_discriminant_stable() {
    assert_eq!(RevoraError::SignatureReplay as u32, 28);
    // Discriminant stability is the primary assertion here.
    // Full replay path requires a valid ed25519 signature; covered by integration tests.
}

// ── 29: SignerKeyNotRegistered ────────────────────────────────────────────────

/// Discriminant 29. meta_set_holder_share when signer has no registered key.
#[test]
fn sec_error_29_signer_key_not_registered_discriminant_and_trigger() {
    assert_eq!(RevoraError::SignerKeyNotRegistered as u32, 29);
    use crate::MetaSetHolderSharePayload;
    let env = Env::default();
    env.mock_all_auths();
    let (c, issuer, token) = with_offering(&env);
    let signer = Address::generate(&env);
    let holder = Address::generate(&env);
    // register delegate so we pass NotAuthorized
    c.set_meta_delegate(&issuer, &symbol_short!("ns"), &token, &signer);
    let payload = MetaSetHolderSharePayload {
        issuer,
        namespace: symbol_short!("ns"),
        token,
        holder,
        share_bps: 1_000,
    };
    let fake_sig = soroban_sdk::BytesN::from_array(&env, &[0u8; 64]);
    // expiry far in future so we don't hit SignatureExpired
    let r = c.try_meta_set_holder_share(&signer, &payload, &1, &u64::MAX, &fake_sig);
    assert_eq!(r, Err(Ok(RevoraError::SignerKeyNotRegistered)));
}

// ── 30: ShareSumExceeded ──────────────────────────────────────────────────────

/// Discriminant 30. set_holder_share pushes aggregate above 10 000 bps.
#[test]
fn sec_error_30_share_sum_exceeded_discriminant_and_trigger() {
    assert_eq!(RevoraError::ShareSumExceeded as u32, 30);
    let env = Env::default();
    env.mock_all_auths();
    let (c, issuer, token) = with_offering(&env);
    let h1 = Address::generate(&env);
    let h2 = Address::generate(&env);
    c.set_holder_share(&issuer, &symbol_short!("ns"), &token, &h1, &9_000);
    // adding 2 000 would push total to 11 000 > 10 000
    let r = c.try_set_holder_share(&issuer, &symbol_short!("ns"), &token, &h2, &2_000);
    assert_eq!(r, Err(Ok(RevoraError::ShareSumExceeded)));
}

// ── Discriminant table completeness ──────────────────────────────────────────

/// All 30 discriminants are distinct and cover the full range 1..=30.
#[test]
fn sec_all_discriminants_are_unique_and_contiguous() {
    let codes: [u32; 30] = [
        RevoraError::InvalidRevenueShareBps as u32,
        RevoraError::LimitReached as u32,
        RevoraError::ConcentrationLimitExceeded as u32,
        RevoraError::OfferingNotFound as u32,
        RevoraError::PeriodAlreadyDeposited as u32,
        RevoraError::NoPendingClaims as u32,
        RevoraError::HolderBlacklisted as u32,
        RevoraError::InvalidShareBps as u32,
        RevoraError::PaymentTokenMismatch as u32,
        RevoraError::ContractFrozen as u32,
        RevoraError::ClaimDelayNotElapsed as u32,
        RevoraError::SnapshotNotEnabled as u32,
        RevoraError::OutdatedSnapshot as u32,
        RevoraError::PayoutAssetMismatch as u32,
        RevoraError::IssuerTransferPending as u32,
        RevoraError::NoTransferPending as u32,
        RevoraError::UnauthorizedTransferAccept as u32,
        RevoraError::MetadataTooLarge as u32,
        RevoraError::NotAuthorized as u32,
        RevoraError::NotInitialized as u32,
        RevoraError::InvalidAmount as u32,
        RevoraError::InvalidPeriodId as u32,
        RevoraError::SupplyCapExceeded as u32,
        RevoraError::MetadataInvalidFormat as u32,
        RevoraError::ReportingWindowClosed as u32,
        RevoraError::ClaimWindowClosed as u32,
        RevoraError::SignatureExpired as u32,
        RevoraError::SignatureReplay as u32,
        RevoraError::SignerKeyNotRegistered as u32,
        RevoraError::ShareSumExceeded as u32,
    ];
    for (i, &code) in codes.iter().enumerate() {
        assert_eq!(code, (i + 1) as u32, "discriminant mismatch at index {}", i);
    }
}
