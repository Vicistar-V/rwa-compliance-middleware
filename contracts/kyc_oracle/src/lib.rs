#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, vec, Address, Env, String, Vec};

use arcm_credentials::CredentialRegistryContractClient;
use arcm_types::KycCredential;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    RegistryAddress,
}

#[contract]
pub struct KycOracleContract;

#[contractimpl]
impl KycOracleContract {
    pub fn __constructor(env: Env, admin: Address, registry_address: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::RegistryAddress, &registry_address);
    }

    pub fn get_kyc_status(env: Env, wallet: Address) -> Option<KycCredential> {
        let registry: Address = env
            .storage()
            .instance()
            .get(&DataKey::RegistryAddress)
            .expect("registry not set");
        let client = CredentialRegistryContractClient::new(&env, &registry);
        client.get_credential(&wallet)
    }

    pub fn is_kyc_valid(env: Env, wallet: Address, required_tier: u32) -> bool {
        let registry: Address = env
            .storage()
            .instance()
            .get(&DataKey::RegistryAddress)
            .expect("registry not set");
        let client = CredentialRegistryContractClient::new(&env, &registry);

        let cred = match client.get_credential(&wallet) {
            Some(c) => c,
            None => return false,
        };

        if cred.is_sanctioned {
            return false;
        }

        if cred.expires_at < env.ledger().timestamp() {
            return false;
        }

        cred.tier >= required_tier
    }

    pub fn submit_credential(env: Env, anchor: Address, credential: KycCredential) {
        let registry: Address = env
            .storage()
            .instance()
            .get(&DataKey::RegistryAddress)
            .expect("registry not set");
        let client = CredentialRegistryContractClient::new(&env, &registry);
        client.submit_credential(&anchor, &credential);
    }

    pub fn revoke_credential(env: Env, anchor: Address, wallet: Address, reason: String) {
        let registry: Address = env
            .storage()
            .instance()
            .get(&DataKey::RegistryAddress)
            .expect("registry not set");
        let client = CredentialRegistryContractClient::new(&env, &registry);
        client.revoke_credential(&anchor, &wallet, &reason);
    }

    pub fn flag_sanctioned(env: Env, oracle_authority: Address, wallet: Address) {
        let registry: Address = env
            .storage()
            .instance()
            .get(&DataKey::RegistryAddress)
            .expect("registry not set");
        let client = CredentialRegistryContractClient::new(&env, &registry);
        client.flag_sanctioned(&oracle_authority, &wallet);
    }

    pub fn check_sanctions(env: Env, wallet: Address) -> bool {
        let registry: Address = env
            .storage()
            .instance()
            .get(&DataKey::RegistryAddress)
            .expect("registry not set");
        let client = CredentialRegistryContractClient::new(&env, &registry);
        client.is_sanctioned(&wallet)
    }

    pub fn batch_check_sanctions(env: Env, wallets: Vec<Address>) -> Vec<(Address, bool)> {
        let registry: Address = env
            .storage()
            .instance()
            .get(&DataKey::RegistryAddress)
            .expect("registry not set");
        let client = CredentialRegistryContractClient::new(&env, &registry);

        let mut results: Vec<(Address, bool)> = vec![&env];
        for i in 0..wallets.len() {
            let wallet = wallets.get(i).unwrap();
            let sanctioned = client.is_sanctioned(&wallet);
            results.push_back((wallet, sanctioned));
        }
        results
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::{vec, BytesN, Env, IntoVal};

    use arcm_credentials::CredentialRegistryContract;

    fn zero_hash(env: &Env) -> BytesN<32> {
        BytesN::from_array(env, &[0u8; 32])
    }

    fn make_credential(
        env: &Env,
        wallet: &Address,
        anchor: &Address,
        tier: u32,
        expires_at: u64,
        sanctioned: bool,
    ) -> KycCredential {
        KycCredential {
            wallet: wallet.clone(),
            tier,
            country_code: "US".into_val(env),
            credential_hash: zero_hash(env),
            issued_at: 1_700_000_000,
            expires_at,
            issuer_anchor: anchor.clone(),
            is_sanctioned: sanctioned,
            sanctions_lists_checked: vec![env, "OFAC".into_val(env)],
        }
    }

    fn setup_env<'a>() -> (Env, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let admin = Address::generate(&env);
        let anchor = Address::generate(&env);
        let oracle_auth = Address::generate(&env);
        (env, admin, anchor, oracle_auth)
    }

    fn deploy<'a>(
        env: &'a Env,
        admin: &Address,
        anchor: &Address,
        oracle_auth: &Address,
    ) -> KycOracleContractClient<'a> {
        let registry_id = env.register(
            CredentialRegistryContract,
            (admin.clone(), anchor.clone(), oracle_auth.clone()),
        );
        let oracle_id = env.register(
            KycOracleContract,
            (admin.clone(), registry_id.clone()),
        );
        KycOracleContractClient::new(env, &oracle_id)
    }

    #[test]
    fn test_is_kyc_valid() {
        let (env, _admin, anchor, _oracle_auth) = setup_env();
        let client = deploy(&env, &_admin, &anchor, &_oracle_auth);
        let wallet = Address::generate(&env);
        let cred = make_credential(&env, &wallet, &anchor, 2, 9999999999, false);

        client.submit_credential(&anchor, &cred);
        assert!(client.is_kyc_valid(&wallet, &2));
        assert!(client.is_kyc_valid(&wallet, &1));
    }

    #[test]
    fn test_is_kyc_valid_wrong_tier() {
        let (env, _admin, anchor, _oracle_auth) = setup_env();
        let client = deploy(&env, &_admin, &anchor, &_oracle_auth);
        let wallet = Address::generate(&env);
        let cred = make_credential(&env, &wallet, &anchor, 1, 9999999999, false);

        client.submit_credential(&anchor, &cred);
        assert!(!client.is_kyc_valid(&wallet, &2));
        assert!(client.is_kyc_valid(&wallet, &1));
    }

    #[test]
    fn test_is_kyc_valid_expired() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        env.ledger().set_timestamp(1_700_000_000);
        let admin = Address::generate(&env);
        let anchor = Address::generate(&env);
        let oracle_auth = Address::generate(&env);

        let registry_id = env.register(
            CredentialRegistryContract,
            (admin.clone(), anchor.clone(), oracle_auth.clone()),
        );

        let oracle_id = env.register(
            KycOracleContract,
            (admin.clone(), registry_id.clone()),
        );
        let oracle_client =
            KycOracleContractClient::new(&env, &oracle_id);

        let wallet = Address::generate(&env);
        let cred = make_credential(&env, &wallet, &anchor, 2, 1_700_000_000 + 100, false);
        oracle_client.submit_credential(&anchor, &cred);

        assert!(oracle_client.is_kyc_valid(&wallet, &2));

        env.ledger().set_timestamp(1_700_000_000 + 200);
        assert!(!oracle_client.is_kyc_valid(&wallet, &2));
    }

    #[test]
    fn test_is_kyc_valid_no_credential() {
        let (env, _admin, _anchor, _oracle_auth) = setup_env();
        let client = deploy(&env, &_admin, &_anchor, &_oracle_auth);
        let wallet = Address::generate(&env);
        assert!(!client.is_kyc_valid(&wallet, &1));
    }

    #[test]
    fn test_is_kyc_valid_sanctioned() {
        let (env, _admin, anchor, oracle_auth) = setup_env();
        let client = deploy(&env, &_admin, &anchor, &oracle_auth);
        let wallet = Address::generate(&env);
        let cred = make_credential(&env, &wallet, &anchor, 2, 9999999999, false);

        client.submit_credential(&anchor, &cred);
        assert!(client.is_kyc_valid(&wallet, &1));

        client.flag_sanctioned(&oracle_auth, &wallet);
        assert!(!client.is_kyc_valid(&wallet, &1));
    }

    #[test]
    fn test_check_sanctions() {
        let (env, _admin, _anchor, oracle_auth) = setup_env();
        let client = deploy(&env, &_admin, &_anchor, &oracle_auth);
        let wallet = Address::generate(&env);

        assert!(!client.check_sanctions(&wallet));
        client.flag_sanctioned(&oracle_auth, &wallet);
        assert!(client.check_sanctions(&wallet));
    }

    #[test]
    fn test_batch_check_sanctions() {
        let (env, _admin, _anchor, oracle_auth) = setup_env();
        let client = deploy(&env, &_admin, &_anchor, &oracle_auth);
        let wallet1 = Address::generate(&env);
        let wallet2 = Address::generate(&env);
        let wallet3 = Address::generate(&env);

        client.flag_sanctioned(&oracle_auth, &wallet2);

        let wallets = vec![&env, wallet1.clone(), wallet2.clone(), wallet3.clone()];
        let results = client.batch_check_sanctions(&wallets);

        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_submit_credential_through_oracle() {
        let (env, _admin, anchor, _oracle_auth) = setup_env();
        let client = deploy(&env, &_admin, &anchor, &_oracle_auth);
        let wallet = Address::generate(&env);
        let cred = make_credential(&env, &wallet, &anchor, 3, 9999999999, false);

        client.submit_credential(&anchor, &cred);

        let status = client.get_kyc_status(&wallet);
        assert!(status.is_some());
        assert_eq!(status.unwrap().tier, 3);
    }

    #[test]
    fn test_revoke_credential_through_oracle() {
        let (env, _admin, anchor, _oracle_auth) = setup_env();
        let client = deploy(&env, &_admin, &anchor, &_oracle_auth);
        let wallet = Address::generate(&env);
        let cred = make_credential(&env, &wallet, &anchor, 2, 9999999999, false);
        let reason: String = "kyc expired".into_val(&env);

        client.submit_credential(&anchor, &cred);

        let status = client.get_kyc_status(&wallet);
        assert!(status.is_some());

        client.revoke_credential(&anchor, &wallet, &reason);

        let updated = client.get_kyc_status(&wallet).unwrap();
        assert!(updated.is_sanctioned);
    }

    #[test]
    fn test_full_integration() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        env.ledger().set_timestamp(1_700_000_000);
        let admin = Address::generate(&env);
        let anchor = Address::generate(&env);
        let oracle_auth = Address::generate(&env);

        let registry_id = env.register(
            CredentialRegistryContract,
            (admin.clone(), anchor.clone(), oracle_auth.clone()),
        );

        let oracle_id = env.register(
            KycOracleContract,
            (admin.clone(), registry_id.clone()),
        );
        let oracle_client =
            KycOracleContractClient::new(&env, &oracle_id);

        let wallet = Address::generate(&env);

        assert!(!oracle_client.is_kyc_valid(&wallet, &1));

        let cred = make_credential(&env, &wallet, &anchor, 2, 1_700_000_000 + 86400, false);
        oracle_client.submit_credential(&anchor, &cred);
        assert!(oracle_client.is_kyc_valid(&wallet, &1));
        assert!(oracle_client.is_kyc_valid(&wallet, &2));

        oracle_client.flag_sanctioned(&oracle_auth, &wallet);
        assert!(!oracle_client.is_kyc_valid(&wallet, &1));
        assert!(oracle_client.check_sanctions(&wallet));

        let reason: String = "sanctions".into_val(&env);
        oracle_client.revoke_credential(&anchor, &wallet, &reason);
        let status = oracle_client.get_kyc_status(&wallet).unwrap();
        assert!(status.is_sanctioned);
    }
}
