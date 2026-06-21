#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct AuditLedgerContract;

#[contractimpl]
impl AuditLedgerContract {
    pub fn __constructor(_env: Env) {}
}
