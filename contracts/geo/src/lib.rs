#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct CountryResolverContract;

#[contractimpl]
impl CountryResolverContract {
    pub fn __constructor(_env: Env) {}
}
