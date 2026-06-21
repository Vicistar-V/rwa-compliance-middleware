#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct CredentialRegistryContract;

#[contractimpl]
impl CredentialRegistryContract {
    pub fn __constructor(_env: Env) {}
}
