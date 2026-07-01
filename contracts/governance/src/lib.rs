#![no_std]
use arcm_types::{AssetClass, JurisdictionRule};
use soroban_sdk::{contract, contractimpl, contracttype, vec, Address, Env, Vec};

const DEFAULT_QUORUM_THRESHOLD: u32 = 3;
const DEFAULT_TIMELOCK_SECONDS: u64 = 172_800;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Proposal(u64),
    NextProposalId,
    Vote(u64, Address),
    QuorumThreshold,
    TimelockDuration,
    Governors,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub country_code: soroban_sdk::String,
    pub asset_class: AssetClass,
    pub new_rule: JurisdictionRule,
    pub proposed_at: u64,
    pub executed: bool,
    pub votes_for: u32,
    pub votes_against: u32,
}

#[contract]
pub struct RuleGovernanceContract;

#[contractimpl]
impl RuleGovernanceContract {
    /// Initializes the governance contract with an admin, default quorum threshold, timelock duration, and an empty governor list.
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::NextProposalId, &1u64);
        env.storage()
            .instance()
            .set(&DataKey::QuorumThreshold, &DEFAULT_QUORUM_THRESHOLD);
        env.storage()
            .instance()
            .set(&DataKey::TimelockDuration, &DEFAULT_TIMELOCK_SECONDS);
        let initial_gov: Vec<Address> = vec![&env];
        env.storage().instance().set(&DataKey::Governors, &initial_gov);
    }

    /// Proposes a new jurisdiction rule update. Returns the unique proposal ID.
    /// Requires authentication from the `proposer`.
    pub fn propose_rule_update(
        env: Env,
        proposer: Address,
        country_code: soroban_sdk::String,
        asset_class: AssetClass,
        new_rule: JurisdictionRule,
    ) -> u64 {
        proposer.require_auth();

        let id: u64 = env.storage().instance().get(&DataKey::NextProposalId).unwrap_or(1);
        env.storage().instance().set(&DataKey::NextProposalId, &(id + 1));

        let proposal = Proposal {
            id,
            proposer: proposer.clone(),
            country_code,
            asset_class,
            new_rule,
            proposed_at: env.ledger().timestamp(),
            executed: false,
            votes_for: 0,
            votes_against: 0,
        };
        env.storage().instance().set(&DataKey::Proposal(id), &proposal);
        id
    }

    /// Casts a vote (for or against) on an active proposal. Each governor may vote only once.
    /// Panics if the caller is not a governor or has already voted.
    pub fn vote_on_proposal(env: Env, governor: Address, proposal_id: u64, approve: bool) {
        governor.require_auth();
        Self::check_governor(&env, &governor);

        let voted: bool = env
            .storage()
            .instance()
            .get(&DataKey::Vote(proposal_id, governor.clone()))
            .unwrap_or(false);
        if voted {
            panic!("already voted");
        }

        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");

        if proposal.executed {
            panic!("proposal already executed");
        }

        if approve {
            proposal.votes_for += 1;
        } else {
            proposal.votes_against += 1;
        }

        env.storage().instance().set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage()
            .instance()
            .set(&DataKey::Vote(proposal_id, governor), &true);
    }

    /// Executes a proposal once it has reached quorum and the timelock period has elapsed.
    /// Marks the proposal as executed.
    pub fn execute_proposal(env: Env, proposal_id: u64) {
        let proposal: Proposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");

        if proposal.executed {
            panic!("proposal already executed");
        }

        let quorum: u32 = env
            .storage()
            .instance()
            .get(&DataKey::QuorumThreshold)
            .unwrap_or(DEFAULT_QUORUM_THRESHOLD);

        if proposal.votes_for < quorum {
            panic!("quorum not reached");
        }

        let timelock: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TimelockDuration)
            .unwrap_or(DEFAULT_TIMELOCK_SECONDS);

        let current_ts = env.ledger().timestamp();
        if current_ts < proposal.proposed_at + timelock {
            panic!("timelock not yet expired");
        }

        let mut updated = proposal;
        updated.executed = true;
        env.storage().instance().set(&DataKey::Proposal(proposal_id), &updated);
    }

    /// Cancels an active proposal. Only the original proposer may cancel.
    /// Marks the proposal as executed (effectively voiding it).
    pub fn cancel_proposal(env: Env, proposer: Address, proposal_id: u64) {
        proposer.require_auth();

        let proposal: Proposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");

        if proposal.proposer != proposer {
            panic!("not the proposer");
        }

        if proposal.executed {
            panic!("proposal already executed");
        }

        let mut updated = proposal;
        updated.executed = true;
        env.storage().instance().set(&DataKey::Proposal(proposal_id), &updated);
    }

    /// Returns the full `Proposal` struct for a given proposal ID.
    /// Panics if the proposal does not exist.
    pub fn get_proposal(env: Env, proposal_id: u64) -> Proposal {
        env.storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found")
    }

    /// Adds a new governor address to the governor list. Admin-only.
    /// No-op if the governor is already registered.
    pub fn add_governor(env: Env, admin: Address, governor: Address) {
        admin.require_auth();
        Self::check_admin(&env, &admin);

        let mut governors: Vec<Address> = env.storage().instance().get(&DataKey::Governors).unwrap_or(vec![&env]);

        let exists = (0..governors.len()).any(|i| governors.get(i).unwrap() == governor);
        if !exists {
            governors.push_back(governor);
            env.storage().instance().set(&DataKey::Governors, &governors);
        }
    }

    /// Updates the minimum number of affirmative votes required to execute a proposal. Admin-only.
    pub fn set_quorum_threshold(env: Env, admin: Address, threshold: u32) {
        admin.require_auth();
        Self::check_admin(&env, &admin);
        env.storage().instance().set(&DataKey::QuorumThreshold, &threshold);
    }

    /// Sets the timelock duration (in seconds) that must elapse before a proposal can be executed. Admin-only.
    pub fn set_timelock_duration(env: Env, admin: Address, duration: u64) {
        admin.require_auth();
        Self::check_admin(&env, &admin);
        env.storage().instance().set(&DataKey::TimelockDuration, &duration);
    }

    fn check_admin(env: &Env, admin: &Address) {
        let stored: Address = env.storage().instance().get(&DataKey::Admin).expect("admin not set");
        if stored != *admin {
            panic!("not authorized");
        }
    }

    fn check_governor(env: &Env, governor: &Address) {
        let governors: Vec<Address> = env.storage().instance().get(&DataKey::Governors).unwrap_or(vec![env]);
        let is_governor = (0..governors.len()).any(|i| governors.get(i).unwrap() == *governor);
        if !is_governor {
            panic!("not a governor");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::{Env, IntoVal, String};

    fn make_country(env: &Env, code: &str) -> String {
        code.into_val(env)
    }

    fn make_rule(env: &Env) -> JurisdictionRule {
        JurisdictionRule::new(
            make_country(env, "US"),
            AssetClass::Generic,
            arcm_types::TransferPolicy::Open,
            1,
            None,
            None,
            None,
            false,
            false,
            1,
            1_700_000_000,
        )
    }

    fn setup_env() -> (Env, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        (env, admin)
    }

    fn deploy<'a>(env: &'a Env, admin: &Address) -> RuleGovernanceContractClient<'a> {
        let contract_id = env.register(RuleGovernanceContract, (admin.clone(),));
        RuleGovernanceContractClient::new(env, &contract_id)
    }

    #[test]
    fn test_propose_rule_update() {
        let (env, admin) = setup_env();
        let client = deploy(&env, &admin);
        let proposer = Address::generate(&env);
        let country = make_country(&env, "US");
        let rule = make_rule(&env);

        let id = client.propose_rule_update(&proposer, &country, &AssetClass::Generic, &rule);
        assert_eq!(id, 1);

        let proposal = client.get_proposal(&id);
        assert_eq!(proposal.proposer, proposer);
        assert!(!proposal.executed);
        assert_eq!(proposal.votes_for, 0);
    }

    #[test]
    fn test_vote_on_proposal() {
        let (env, admin) = setup_env();
        let client = deploy(&env, &admin);
        let proposer = Address::generate(&env);
        let governor = Address::generate(&env);
        let country = make_country(&env, "US");
        let rule = make_rule(&env);

        client.add_governor(&admin, &governor);
        let id = client.propose_rule_update(&proposer, &country, &AssetClass::Generic, &rule);
        client.vote_on_proposal(&governor, &id, &true);

        let proposal = client.get_proposal(&id);
        assert_eq!(proposal.votes_for, 1);
        assert_eq!(proposal.votes_against, 0);
    }

    #[test]
    fn test_vote_against_proposal() {
        let (env, admin) = setup_env();
        let client = deploy(&env, &admin);
        let proposer = Address::generate(&env);
        let governor = Address::generate(&env);
        let country = make_country(&env, "US");
        let rule = make_rule(&env);

        client.add_governor(&admin, &governor);
        let id = client.propose_rule_update(&proposer, &country, &AssetClass::Generic, &rule);
        client.vote_on_proposal(&governor, &id, &false);

        let proposal = client.get_proposal(&id);
        assert_eq!(proposal.votes_for, 0);
        assert_eq!(proposal.votes_against, 1);
    }

    #[test]
    fn test_execute_proposal_after_quorum_and_timelock() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_700_000_000);
        let admin = Address::generate(&env);

        let contract_id = env.register(RuleGovernanceContract, (admin.clone(),));
        let client = RuleGovernanceContractClient::new(&env, &contract_id);

        let proposer = Address::generate(&env);
        let governor1 = Address::generate(&env);
        let governor2 = Address::generate(&env);
        let governor3 = Address::generate(&env);
        let country = make_country(&env, "US");
        let rule = make_rule(&env);

        client.add_governor(&admin, &governor1);
        client.add_governor(&admin, &governor2);
        client.add_governor(&admin, &governor3);

        let id = client.propose_rule_update(&proposer, &country, &AssetClass::Generic, &rule);
        client.vote_on_proposal(&governor1, &id, &true);
        client.vote_on_proposal(&governor2, &id, &true);
        client.vote_on_proposal(&governor3, &id, &true);

        env.ledger().set_timestamp(1_700_000_000 + DEFAULT_TIMELOCK_SECONDS + 1);
        client.execute_proposal(&id);

        let proposal = client.get_proposal(&id);
        assert!(proposal.executed);
    }

    #[test]
    fn test_cancel_proposal() {
        let (env, admin) = setup_env();
        let client = deploy(&env, &admin);
        let proposer = Address::generate(&env);
        let country = make_country(&env, "US");
        let rule = make_rule(&env);

        let id = client.propose_rule_update(&proposer, &country, &AssetClass::Generic, &rule);
        client.cancel_proposal(&proposer, &id);

        let proposal = client.get_proposal(&id);
        assert!(proposal.executed);
    }

    #[test]
    #[should_panic(expected = "HostError")]
    fn test_double_vote_prevented() {
        let (env, admin) = setup_env();
        let client = deploy(&env, &admin);
        let proposer = Address::generate(&env);
        let governor = Address::generate(&env);
        let country = make_country(&env, "US");
        let rule = make_rule(&env);

        client.add_governor(&admin, &governor);
        let id = client.propose_rule_update(&proposer, &country, &AssetClass::Generic, &rule);
        client.vote_on_proposal(&governor, &id, &true);
        client.vote_on_proposal(&governor, &id, &true);
    }

    #[test]
    #[should_panic(expected = "HostError")]
    fn test_non_proposer_cannot_cancel() {
        let (env, admin) = setup_env();
        let client = deploy(&env, &admin);
        let proposer = Address::generate(&env);
        let attacker = Address::generate(&env);
        let country = make_country(&env, "US");
        let rule = make_rule(&env);

        let id = client.propose_rule_update(&proposer, &country, &AssetClass::Generic, &rule);
        client.cancel_proposal(&attacker, &id);
    }

    #[test]
    #[should_panic(expected = "HostError")]
    fn test_execute_before_timelock() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_700_000_000);
        let admin = Address::generate(&env);

        let contract_id = env.register(RuleGovernanceContract, (admin.clone(),));
        let client = RuleGovernanceContractClient::new(&env, &contract_id);

        let proposer = Address::generate(&env);
        let governor = Address::generate(&env);
        let country = make_country(&env, "US");
        let rule = make_rule(&env);

        client.add_governor(&admin, &governor);
        let id = client.propose_rule_update(&proposer, &country, &AssetClass::Generic, &rule);
        client.vote_on_proposal(&governor, &id, &true);

        client.execute_proposal(&id);
    }

    #[test]
    #[should_panic(expected = "HostError")]
    fn test_execute_without_quorum() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_700_000_000);
        let admin = Address::generate(&env);

        let contract_id = env.register(RuleGovernanceContract, (admin.clone(),));
        let client = RuleGovernanceContractClient::new(&env, &contract_id);

        let proposer = Address::generate(&env);
        let governor = Address::generate(&env);
        let country = make_country(&env, "US");
        let rule = make_rule(&env);

        client.add_governor(&admin, &governor);
        let id = client.propose_rule_update(&proposer, &country, &AssetClass::Generic, &rule);
        client.vote_on_proposal(&governor, &id, &true);

        env.ledger().set_timestamp(1_700_000_000 + DEFAULT_TIMELOCK_SECONDS + 1);
        client.execute_proposal(&id);
    }

    #[test]
    #[should_panic(expected = "HostError")]
    fn test_non_governor_cannot_vote() {
        let (env, admin) = setup_env();
        let client = deploy(&env, &admin);
        let proposer = Address::generate(&env);
        let non_governor = Address::generate(&env);
        let country = make_country(&env, "US");
        let rule = make_rule(&env);

        let id = client.propose_rule_update(&proposer, &country, &AssetClass::Generic, &rule);
        client.vote_on_proposal(&non_governor, &id, &true);
    }

    #[test]
    fn test_add_multiple_governors() {
        let (env, admin) = setup_env();
        let client = deploy(&env, &admin);
        let g1 = Address::generate(&env);
        let g2 = Address::generate(&env);

        client.add_governor(&admin, &g1);
        client.add_governor(&admin, &g2);

        let proposer = Address::generate(&env);
        let country = make_country(&env, "US");
        let rule = make_rule(&env);

        let id = client.propose_rule_update(&proposer, &country, &AssetClass::Generic, &rule);
        client.vote_on_proposal(&g1, &id, &true);
        client.vote_on_proposal(&g2, &id, &true);
    }

    #[test]
    fn test_set_quorum_threshold_affects_execution() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_700_000_000);
        let admin = Address::generate(&env);

        let contract_id = env.register(RuleGovernanceContract, (admin.clone(),));
        let client = RuleGovernanceContractClient::new(&env, &contract_id);

        client.set_quorum_threshold(&admin, &1);

        let proposer = Address::generate(&env);
        let governor = Address::generate(&env);
        let country = make_country(&env, "US");
        let rule = make_rule(&env);

        client.add_governor(&admin, &governor);

        let id = client.propose_rule_update(&proposer, &country, &AssetClass::Generic, &rule);
        client.vote_on_proposal(&governor, &id, &true);

        env.ledger().set_timestamp(1_700_000_000 + DEFAULT_TIMELOCK_SECONDS + 1);
        client.execute_proposal(&id);

        let proposal = client.get_proposal(&id);
        assert!(proposal.executed);
    }

    #[test]
    #[should_panic(expected = "HostError")]
    fn test_non_admin_cannot_set_quorum() {
        let (env, admin) = setup_env();
        let client = deploy(&env, &admin);
        let attacker = Address::generate(&env);
        client.set_quorum_threshold(&attacker, &1);
    }
}
