#![cfg(test)]
#![allow(warnings)] // Silences the unused variable errors failing the CI

use soroban_sdk::{testutils::Address as _, Address, Env};
use crate::{RevoraRevenueShare, RevoraRevenueShareClient};

/// Core test utilities avoiding self-referential struct lifetime errors.
pub fn setup_context<'a>(
    env: &'a Env,
) -> (RevoraRevenueShareClient<'a>, Address, Address, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RevoraRevenueShare);
    let client = RevoraRevenueShareClient::new(env, &contract_id);
    let issuer = Address::generate(env);
    let token = Address::generate(env);
    let payout_asset = Address::generate(env);
    (client, contract_id, issuer, token, payout_asset)
}

// Helper to create a new namespace
