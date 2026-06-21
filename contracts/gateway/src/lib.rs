#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct GatewayContract;

#[contractimpl]
impl GatewayContract {
    pub fn __constructor(_env: Env) {}
}
