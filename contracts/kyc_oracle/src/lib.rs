#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct KycOracleContract;

#[contractimpl]
impl KycOracleContract {
    pub fn __constructor(_env: Env) {}
}
