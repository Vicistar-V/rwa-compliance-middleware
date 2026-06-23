#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, vec, Address, Env, String, Vec};

use arcm_types::{AssetClass, JurisdictionRule, TransferPolicy};

const TIMELOCK_SECONDS: u64 = 172_800;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    CountryRule(String, AssetClass),
    RuleKeys,
    Proposal(u64),
    NextProposalId,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub country_code: String,
    pub asset_class: AssetClass,
    pub new_rule: JurisdictionRule,
    pub proposed_at: u64,
    pub executed: bool,
}

#[contract]
pub struct JurisdictionEngineContract;

#[contractimpl]
impl JurisdictionEngineContract {
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::NextProposalId, &1u64);
    }

    pub fn get_rule(
        env: Env,
        country_code: String,
        asset_class: AssetClass,
    ) -> JurisdictionRule {
        env.storage()
            .instance()
            .get(&DataKey::CountryRule(country_code, asset_class))
            .expect("rule not found")
    }

    pub fn set_rule(
        env: Env,
        admin: Address,
        country_code: String,
        asset_class: AssetClass,
        rule: JurisdictionRule,
    ) {
        admin.require_auth();
        Self::check_admin(&env, &admin);

        let key = DataKey::CountryRule(country_code.clone(), asset_class.clone());
        env.storage().instance().set(&key, &rule);

        let mut keys: Vec<(String, AssetClass)> = env
            .storage()
            .instance()
            .get(&DataKey::RuleKeys)
            .unwrap_or(vec![&env]);

        let exists = (0..keys.len()).any(|i| {
            let k = keys.get(i).unwrap();
            k.0 == country_code && k.1 == asset_class
        });
        if !exists {
            keys.push_back((country_code, asset_class));
            env.storage().instance().set(&DataKey::RuleKeys, &keys);
        }
    }

    pub fn propose_rule_update(
        env: Env,
        proposer: Address,
        country_code: String,
        asset_class: AssetClass,
        new_rule: JurisdictionRule,
    ) -> u64 {
        proposer.require_auth();

        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextProposalId)
            .unwrap_or(1);
        env.storage()
            .instance()
            .set(&DataKey::NextProposalId, &(id + 1));

        let proposal = Proposal {
            id,
            proposer: proposer.clone(),
            country_code,
            asset_class,
            new_rule,
            proposed_at: env.ledger().timestamp(),
            executed: false,
        };
        env.storage()
            .instance()
            .set(&DataKey::Proposal(id), &proposal);
        id
    }

    pub fn execute_rule_update(env: Env, proposal_id: u64) {
        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");

        if proposal.executed {
            panic!("proposal already executed");
        }

        let current_ts = env.ledger().timestamp();
        if current_ts < proposal.proposed_at + TIMELOCK_SECONDS {
            panic!("timelock not yet expired");
        }

        let key = DataKey::CountryRule(
            proposal.country_code.clone(),
            proposal.asset_class.clone(),
        );
        env.storage().instance().set(&key, &proposal.new_rule);

        proposal.executed = true;
        env.storage()
            .instance()
            .set(&DataKey::Proposal(proposal_id), &proposal);
    }

    pub fn list_country_rules(env: Env, country_code: String) -> Vec<JurisdictionRule> {
        let keys: Vec<(String, AssetClass)> = env
            .storage()
            .instance()
            .get(&DataKey::RuleKeys)
            .unwrap_or(vec![&env]);

        let mut result: Vec<JurisdictionRule> = vec![&env];
        for i in 0..keys.len() {
            let (cc, ac) = keys.get(i).unwrap();
            if cc == country_code {
                if let Some(rule) = env
                    .storage()
                    .instance()
                    .get(&DataKey::CountryRule(cc, ac))
                {
                    result.push_back(rule);
                }
            }
        }
        result
    }

    pub fn is_sanctioned(env: Env, country_code: String) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::CountryRule(
                country_code,
                AssetClass::Generic,
            ))
            .map_or(false, |rule: JurisdictionRule| {
                rule.transfer_policy == TransferPolicy::Sanctioned
            })
    }

    fn check_admin(env: &Env, admin: &Address) {
        let stored: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("admin not set");
        if stored != *admin {
            panic!("not authorized");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use arcm_types::{evaluate_transfer, ReasonCode};
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::{Address, Env, IntoVal, String};

    fn make_country(env: &Env, code: &str) -> String {
        code.into_val(env)
    }

    fn make_rule(
        env: &Env,
        country: &str,
        policy: TransferPolicy,
        min_kyc: u32,
        max_transfer: Option<u128>,
        max_holding: Option<u128>,
        min_hold_period: Option<u64>,
        requires_approval: bool,
        clawback_on_expiry: bool,
    ) -> JurisdictionRule {
        JurisdictionRule::new(
            make_country(env, country),
            AssetClass::Generic,
            policy,
            min_kyc,
            max_transfer,
            max_holding,
            min_hold_period,
            requires_approval,
            clawback_on_expiry,
            1,
            1_700_000_000,
        )
    }

    fn setup_env<'a>() -> (Env, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        (env, admin)
    }

    fn deploy<'a>(
        env: &'a Env,
        admin: &Address,
    ) -> JurisdictionEngineContractClient<'a> {
        let contract_id =
            env.register(JurisdictionEngineContract, (admin.clone(),));
        JurisdictionEngineContractClient::new(env, &contract_id)
    }

    // --- evaluate_transfer tests ---

    #[test]
    fn test_evaluate_sanctioned_sender() {
        let env = Env::default();
        let sanctioned = make_rule(
            &env, "IR", TransferPolicy::Sanctioned, 1, None, None, None, false, false,
        );
        let open = make_rule(
            &env, "US", TransferPolicy::Open, 1, None, None, None, false, false,
        );
        let decision = evaluate_transfer(
            &sanctioned, &open, 1000, 1, 1, 9999999999, 1_700_000_000, None, None,
        );
        assert!(decision.is_reject());
        assert_eq!(
            decision.reason_code(),
            Some(&ReasonCode::SanctionedJurisdiction)
        );
    }

    #[test]
    fn test_evaluate_sanctioned_receiver() {
        let env = Env::default();
        let open = make_rule(
            &env, "US", TransferPolicy::Open, 1, None, None, None, false, false,
        );
        let sanctioned = make_rule(
            &env, "KP", TransferPolicy::Sanctioned, 1, None, None, None, false, false,
        );
        let decision = evaluate_transfer(
            &open, &sanctioned, 1000, 1, 1, 9999999999, 1_700_000_000, None, None,
        );
        assert!(decision.is_reject());
        assert_eq!(
            decision.reason_code(),
            Some(&ReasonCode::SanctionedJurisdiction)
        );
    }

    #[test]
    fn test_evaluate_prohibited_jurisdiction() {
        let env = Env::default();
        let prohibited = make_rule(
            &env, "CN", TransferPolicy::Prohibited, 1, None, None, None, false, false,
        );
        let open = make_rule(
            &env, "US", TransferPolicy::Open, 1, None, None, None, false, false,
        );
        let decision = evaluate_transfer(
            &prohibited, &open, 1000, 1, 1, 9999999999, 1_700_000_000, None, None,
        );
        assert!(decision.is_reject());
        assert_eq!(
            decision.reason_code(),
            Some(&ReasonCode::ProhibitedJurisdiction)
        );
    }

    #[test]
    fn test_evaluate_kyc_tier_too_low() {
        let env = Env::default();
        let sender = make_rule(
            &env, "US", TransferPolicy::Restricted, 2, None, None, None, false, false,
        );
        let receiver = make_rule(
            &env, "DE", TransferPolicy::Open, 1, None, None, None, false, false,
        );
        let decision = evaluate_transfer(
            &sender, &receiver, 1000, 1, 1, 9999999999, 1_700_000_000, None, None,
        );
        assert!(decision.is_reject());
        assert_eq!(
            decision.reason_code(),
            Some(&ReasonCode::InsufficientKycTier)
        );
    }

    #[test]
    fn test_evaluate_kyc_expired_lock() {
        let env = Env::default();
        let sender = make_rule(
            &env, "US", TransferPolicy::Open, 1, None, None, None, false, false,
        );
        let receiver = make_rule(
            &env, "DE", TransferPolicy::Open, 1, None, None, None, false, false,
        );
        let decision = evaluate_transfer(
            &sender,
            &receiver,
            1000,
            1,
            1,
            1_500_000_000,
            1_700_000_000,
            None,
            None,
        );
        assert!(decision.is_lock());
        assert_eq!(decision.reason_code(), Some(&ReasonCode::KycExpired));
    }

    #[test]
    fn test_evaluate_kyc_expired_clawback() {
        let env = Env::default();
        let sender = make_rule(
            &env, "US", TransferPolicy::Open, 1, None, None, None, false, false,
        );
        let receiver = make_rule(
            &env, "DE", TransferPolicy::Open, 1, None, None, None, false, true,
        );
        let decision = evaluate_transfer(
            &sender,
            &receiver,
            1000,
            1,
            1,
            1_500_000_000,
            1_700_000_000,
            None,
            None,
        );
        assert!(decision.is_clawback());
        assert_eq!(decision.reason_code(), Some(&ReasonCode::KycExpired));
    }

    #[test]
    fn test_evaluate_amount_exceeds_cap() {
        let env = Env::default();
        let sender = make_rule(
            &env, "US", TransferPolicy::Open, 1, None, None, None, false, false,
        );
        let receiver = make_rule(
            &env, "DE", TransferPolicy::Open, 1, Some(500), None, None, false, false,
        );
        let decision = evaluate_transfer(
            &sender, &receiver, 1000, 1, 1, 9999999999, 1_700_000_000, None, None,
        );
        assert!(decision.is_reject());
        assert_eq!(
            decision.reason_code(),
            Some(&ReasonCode::AmountExceedsJurisdictionCap)
        );
    }

    #[test]
    fn test_evaluate_holding_period_not_met() {
        let env = Env::default();
        let sender = make_rule(
            &env, "US", TransferPolicy::Open, 1, None, None, Some(1000), false, false,
        );
        let receiver = make_rule(
            &env, "DE", TransferPolicy::Open, 1, None, None, None, false, false,
        );
        let decision = evaluate_transfer(
            &sender,
            &receiver,
            1000,
            1,
            1,
            9999999999,
            1_700_000_000,
            Some(1_700_000_000),
            None,
        );
        assert!(decision.is_reject());
        assert_eq!(
            decision.reason_code(),
            Some(&ReasonCode::HoldingPeriodNotMet)
        );
    }

    #[test]
    fn test_evaluate_holdings_cap_exceeded() {
        let env = Env::default();
        let sender = make_rule(
            &env, "US", TransferPolicy::Open, 1, None, None, None, false, false,
        );
        let receiver = make_rule(
            &env, "DE", TransferPolicy::Open, 1, None, Some(5000), None, false, false,
        );
        let decision = evaluate_transfer(
            &sender,
            &receiver,
            3000,
            1,
            1,
            9999999999,
            1_700_000_000,
            None,
            Some(3000),
        );
        assert!(decision.is_reject());
        assert_eq!(
            decision.reason_code(),
            Some(&ReasonCode::HoldingsCapExceeded)
        );
    }

    #[test]
    fn test_evaluate_issuer_approval_required() {
        let env = Env::default();
        let sender = make_rule(
            &env, "US", TransferPolicy::Open, 1, None, None, None, false, false,
        );
        let receiver = make_rule(
            &env, "DE", TransferPolicy::Open, 1, None, None, None, true, false,
        );
        let decision = evaluate_transfer(
            &sender, &receiver, 1000, 1, 1, 9999999999, 1_700_000_000, None, None,
        );
        assert!(decision.is_pending());
    }

    #[test]
    fn test_evaluate_all_checks_pass() {
        let env = Env::default();
        let sender = make_rule(
            &env, "US", TransferPolicy::Open, 1, None, None, None, false, false,
        );
        let receiver = make_rule(
            &env, "DE", TransferPolicy::Open, 1, Some(5000), Some(10000), None, false, false,
        );
        let decision = evaluate_transfer(
            &sender,
            &receiver,
            1000,
            2,
            2,
            9999999999,
            1_700_000_000,
            Some(1_600_000_000),
            Some(500),
        );
        assert!(decision.is_approve());
    }

    // --- Contract function tests ---

    #[test]
    fn test_set_and_get_rule() {
        let (env, admin) = setup_env();
        let client = deploy(&env, &admin);
        let country = make_country(&env, "US");
        let rule = make_rule(
            &env, "US", TransferPolicy::AccreditedOnly, 2, Some(1_000_000), None, None, false, false,
        );

        client.set_rule(&admin, &country, &AssetClass::Generic, &rule);
        let retrieved = client.get_rule(&country, &AssetClass::Generic);
        assert_eq!(retrieved.transfer_policy, TransferPolicy::AccreditedOnly);
        assert_eq!(retrieved.min_kyc_tier, 2);
    }

    #[test]
    #[should_panic(expected = "HostError")]
    fn test_get_nonexistent_rule_panics() {
        let (env, admin) = setup_env();
        let client = deploy(&env, &admin);
        let country = make_country(&env, "XX");
        client.get_rule(&country, &AssetClass::Generic);
    }

    #[test]
    fn test_propose_and_execute_rule_update() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_700_000_000);
        let admin = Address::generate(&env);
        let proposer = Address::generate(&env);

        let contract_id =
            env.register(JurisdictionEngineContract, (admin.clone(),));
        let client =
            JurisdictionEngineContractClient::new(&env, &contract_id);

        let country = make_country(&env, "US");
        let old_rule = make_rule(
            &env, "US", TransferPolicy::Open, 1, None, None, None, false, false,
        );
        let new_rule = make_rule(
            &env, "US", TransferPolicy::AccreditedOnly, 2, None, None, None, false, false,
        );

        client.set_rule(&admin, &country, &AssetClass::Generic, &old_rule);

        let proposal_id = client.propose_rule_update(&proposer, &country, &AssetClass::Generic, &new_rule);
        assert_eq!(proposal_id, 1);

        env.ledger().set_timestamp(1_700_000_000 + TIMELOCK_SECONDS + 1);
        client.execute_rule_update(&proposal_id);

        let retrieved = client.get_rule(&country, &AssetClass::Generic);
        assert_eq!(retrieved.transfer_policy, TransferPolicy::AccreditedOnly);
        assert_eq!(retrieved.min_kyc_tier, 2);
    }

    #[test]
    #[should_panic(expected = "HostError")]
    fn test_execute_before_timelock_expires() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_700_000_000);
        let admin = Address::generate(&env);
        let proposer = Address::generate(&env);

        let contract_id =
            env.register(JurisdictionEngineContract, (admin.clone(),));
        let client =
            JurisdictionEngineContractClient::new(&env, &contract_id);

        let country = make_country(&env, "US");
        let old_rule = make_rule(
            &env, "US", TransferPolicy::Open, 1, None, None, None, false, false,
        );
        let new_rule = make_rule(
            &env, "US", TransferPolicy::AccreditedOnly, 2, None, None, None, false, false,
        );

        client.set_rule(&admin, &country, &AssetClass::Generic, &old_rule);
        let proposal_id = client.propose_rule_update(&proposer, &country, &AssetClass::Generic, &new_rule);

        env.ledger().set_timestamp(1_700_000_000 + 1);
        client.execute_rule_update(&proposal_id);
    }

    #[test]
    #[should_panic(expected = "HostError")]
    fn test_double_execute() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_700_000_000);
        let admin = Address::generate(&env);
        let proposer = Address::generate(&env);

        let contract_id =
            env.register(JurisdictionEngineContract, (admin.clone(),));
        let client =
            JurisdictionEngineContractClient::new(&env, &contract_id);

        let country = make_country(&env, "US");
        let old_rule = make_rule(
            &env, "US", TransferPolicy::Open, 1, None, None, None, false, false,
        );
        let new_rule = make_rule(
            &env, "US", TransferPolicy::AccreditedOnly, 2, None, None, None, false, false,
        );

        client.set_rule(&admin, &country, &AssetClass::Generic, &old_rule);
        let proposal_id = client.propose_rule_update(&proposer, &country, &AssetClass::Generic, &new_rule);

        env.ledger().set_timestamp(1_700_000_000 + TIMELOCK_SECONDS + 1);
        client.execute_rule_update(&proposal_id);
        client.execute_rule_update(&proposal_id);
    }

    #[test]
    fn test_list_country_rules() {
        let (env, admin) = setup_env();
        let client = deploy(&env, &admin);
        let us = make_country(&env, "US");
        let de = make_country(&env, "DE");

        let rule_us = make_rule(
            &env, "US", TransferPolicy::Open, 1, None, None, None, false, false,
        );
        let rule_de = make_rule(
            &env, "DE", TransferPolicy::Restricted, 2, Some(5000), None, None, false, false,
        );
        let rule_us_equity = make_rule(
            &env, "US", TransferPolicy::AccreditedOnly, 2, None, None, None, false, false,
        );

        client.set_rule(&admin, &us, &AssetClass::Generic, &rule_us);
        client.set_rule(&admin, &de, &AssetClass::RealEstate, &rule_de);
        client.set_rule(&admin, &us, &AssetClass::Equity, &rule_us_equity);

        let us_rules = client.list_country_rules(&us);
        assert_eq!(us_rules.len(), 2);

        let de_rules = client.list_country_rules(&de);
        assert_eq!(de_rules.len(), 1);
    }

    #[test]
    fn test_is_sanctioned() {
        let (env, admin) = setup_env();
        let client = deploy(&env, &admin);
        let ir = make_country(&env, "IR");
        let us = make_country(&env, "US");

        let sanctioned_rule = make_rule(
            &env, "IR", TransferPolicy::Sanctioned, 1, None, None, None, false, false,
        );

        client.set_rule(&admin, &ir, &AssetClass::Generic, &sanctioned_rule);
        assert!(client.is_sanctioned(&ir));
        assert!(!client.is_sanctioned(&us));
    }

    #[test]
    fn test_is_sanctioned_default_false() {
        let (env, admin) = setup_env();
        let client = deploy(&env, &admin);
        let unknown = make_country(&env, "ZZ");
        assert!(!client.is_sanctioned(&unknown));
    }

    #[test]
    #[should_panic(expected = "HostError")]
    fn test_non_admin_cannot_set_rule() {
        let (env, admin) = setup_env();
        let client = deploy(&env, &admin);
        let attacker = Address::generate(&env);

        let country = make_country(&env, "US");
        let rule = make_rule(
            &env, "US", TransferPolicy::Open, 1, None, None, None, false, false,
        );
        client.set_rule(&attacker, &country, &AssetClass::Generic, &rule);
    }
}
