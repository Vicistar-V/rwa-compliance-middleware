#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct RuleGovernanceContract;

#[contractimpl]
impl RuleGovernanceContract {
    pub fn __constructor(_env: Env) {}
}
