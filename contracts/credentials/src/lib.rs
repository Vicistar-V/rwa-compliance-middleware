#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, vec, Address, Env, String, Vec};
use arcm_types::KycCredential;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Anchor,
    OracleAuthority,
    Credential(Address),
    Sanctioned(Address),
    CredentialWallets,
}

#[contract]
pub struct CredentialRegistryContract;

#[contractimpl]
impl CredentialRegistryContract {
    pub fn __constructor(env: Env, admin: Address, anchor: Address, oracle_authority: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Anchor, &anchor);
        env.storage()
            .instance()
            .set(&DataKey::OracleAuthority, &oracle_authority);
    }

    pub fn submit_credential(env: Env, anchor: Address, credential: KycCredential) {
        anchor.require_auth();
        let stored_anchor: Address = env
            .storage()
            .instance()
            .get(&DataKey::Anchor)
            .expect("anchor not set");
        if stored_anchor != anchor {
            panic!("not authorized");
        }

        let wallet = credential.wallet.clone();
        env.storage()
            .instance()
            .set(&DataKey::Credential(wallet.clone()), &credential);

        let mut wallets: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::CredentialWallets)
            .unwrap_or(vec![&env]);

        let exists = (0..wallets.len())
            .any(|i| wallets.get(i).unwrap() == wallet);
        if !exists {
            wallets.push_back(wallet);
            env.storage()
                .instance()
                .set(&DataKey::CredentialWallets, &wallets);
        }
    }

    pub fn get_credential(env: Env, wallet: Address) -> Option<KycCredential> {
        env.storage()
            .instance()
            .get(&DataKey::Credential(wallet))
    }

    pub fn revoke_credential(env: Env, anchor: Address, wallet: Address, _reason: String) {
        anchor.require_auth();
        let stored_anchor: Address = env
            .storage()
            .instance()
            .get(&DataKey::Anchor)
            .expect("anchor not set");
        if stored_anchor != anchor {
            panic!("not authorized");
        }

        let mut cred: KycCredential = env
            .storage()
            .instance()
            .get(&DataKey::Credential(wallet.clone()))
            .expect("credential not found");

        cred.is_sanctioned = true;
        env.storage()
            .instance()
            .set(&DataKey::Credential(wallet), &cred);
    }

    pub fn flag_sanctioned(env: Env, oracle_authority: Address, wallet: Address) {
        oracle_authority.require_auth();
        let stored_oracle: Address = env
            .storage()
            .instance()
            .get(&DataKey::OracleAuthority)
            .expect("oracle authority not set");
        if stored_oracle != oracle_authority {
            panic!("not authorized");
        }

        env.storage()
            .instance()
            .set(&DataKey::Sanctioned(wallet.clone()), &true);

        let existing: Option<KycCredential> = env
            .storage()
            .instance()
            .get(&DataKey::Credential(wallet.clone()));
        if let Some(mut cred) = existing {
            cred.is_sanctioned = true;
            env.storage()
                .instance()
                .set(&DataKey::Credential(wallet), &cred);
        }
    }

    pub fn unflag_sanctioned(env: Env, oracle_authority: Address, wallet: Address) {
        oracle_authority.require_auth();
        let stored_oracle: Address = env
            .storage()
            .instance()
            .get(&DataKey::OracleAuthority)
            .expect("oracle authority not set");
        if stored_oracle != oracle_authority {
            panic!("not authorized");
        }

        env.storage()
            .instance()
            .remove(&DataKey::Sanctioned(wallet.clone()));

        let existing: Option<KycCredential> = env
            .storage()
            .instance()
            .get(&DataKey::Credential(wallet.clone()));
        if let Some(mut cred) = existing {
            cred.is_sanctioned = false;
            env.storage()
                .instance()
                .set(&DataKey::Credential(wallet), &cred);
        }
    }

    pub fn is_sanctioned(env: Env, wallet: Address) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Sanctioned(wallet))
            .unwrap_or(false)
    }

    pub fn expiring_credentials(env: Env, within_seconds: u64) -> Vec<Address> {
        let wallets: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::CredentialWallets)
            .unwrap_or(vec![&env]);

        let now = env.ledger().timestamp();
        let cutoff = now + within_seconds;

        let mut result: Vec<Address> = vec![&env];
        for i in 0..wallets.len() {
            let wallet = wallets.get(i).unwrap();
            let existing: Option<KycCredential> = env
                .storage()
                .instance()
                .get(&DataKey::Credential(wallet.clone()));
            if let Some(cred) = existing {
                if cred.expires_at <= cutoff && cred.expires_at >= now {
                    result.push_back(wallet);
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::{BytesN, Env, IntoVal};

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

    fn setup_env() -> (Env, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let anchor = Address::generate(&env);
        let oracle = Address::generate(&env);
        (env, admin, anchor, oracle)
    }

    fn deploy<'a>(
        env: &'a Env,
        admin: &Address,
        anchor: &Address,
        oracle: &Address,
    ) -> CredentialRegistryContractClient<'a> {
        let contract_id = env.register(
            CredentialRegistryContract,
            (admin.clone(), anchor.clone(), oracle.clone()),
        );
        CredentialRegistryContractClient::new(env, &contract_id)
    }

    #[test]
    fn test_submit_and_get_credential() {
        let (env, admin, anchor, oracle) = setup_env();
        let client = deploy(&env, &admin, &anchor, &oracle);
        let wallet = Address::generate(&env);
        let cred = make_credential(&env, &wallet, &anchor, 2, 9999999999, false);

        client.submit_credential(&anchor, &cred);
        let retrieved = client.get_credential(&wallet);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().tier, 2);
    }

    #[test]
    fn test_get_nonexistent_credential() {
        let (env, admin, anchor, oracle) = setup_env();
        let client = deploy(&env, &admin, &anchor, &oracle);
        let wallet = Address::generate(&env);
        let retrieved = client.get_credential(&wallet);
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_revoke_credential() {
        let (env, admin, anchor, oracle) = setup_env();
        let client = deploy(&env, &admin, &anchor, &oracle);
        let wallet = Address::generate(&env);
        let cred = make_credential(&env, &wallet, &anchor, 2, 9999999999, false);

        client.submit_credential(&anchor, &cred);
        let reason: String = "fraud".into_val(&env);
        client.revoke_credential(&anchor, &wallet, &reason);

        let retrieved = client.get_credential(&wallet).unwrap();
        assert!(retrieved.is_sanctioned);
    }

    #[test]
    fn test_flag_and_unflag_sanctioned() {
        let (env, admin, anchor, oracle) = setup_env();
        let client = deploy(&env, &admin, &anchor, &oracle);
        let wallet = Address::generate(&env);

        assert!(!client.is_sanctioned(&wallet));
        client.flag_sanctioned(&oracle, &wallet);
        assert!(client.is_sanctioned(&wallet));
        client.unflag_sanctioned(&oracle, &wallet);
        assert!(!client.is_sanctioned(&wallet));
    }

    #[test]
    fn test_flag_sanctioned_updates_credential() {
        let (env, admin, anchor, oracle) = setup_env();
        let client = deploy(&env, &admin, &anchor, &oracle);
        let wallet = Address::generate(&env);
        let cred = make_credential(&env, &wallet, &anchor, 2, 9999999999, false);

        client.submit_credential(&anchor, &cred);
        client.flag_sanctioned(&oracle, &wallet);

        let retrieved = client.get_credential(&wallet).unwrap();
        assert!(retrieved.is_sanctioned);
    }

    #[test]
    fn test_expiring_credentials() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_700_000_000);
        let admin = Address::generate(&env);
        let anchor = Address::generate(&env);
        let oracle = Address::generate(&env);

        let contract_id = env.register(
            CredentialRegistryContract,
            (admin.clone(), anchor.clone(), oracle.clone()),
        );
        let client =
            CredentialRegistryContractClient::new(&env, &contract_id);

        let wallet1 = Address::generate(&env);
        let wallet2 = Address::generate(&env);
        let wallet3 = Address::generate(&env);

        let cred1 = make_credential(&env, &wallet1, &anchor, 1, 1_700_000_000 + 1000, false);
        let cred2 = make_credential(&env, &wallet2, &anchor, 1, 1_700_000_000 + 5000, false);
        let cred3 = make_credential(&env, &wallet3, &anchor, 1, 1_700_000_000 + 99999, false);

        client.submit_credential(&anchor, &cred1);
        client.submit_credential(&anchor, &cred2);
        client.submit_credential(&anchor, &cred3);

        let expiring = client.expiring_credentials(&2000);
        assert_eq!(expiring.len(), 1);
        assert_eq!(expiring.get(0).unwrap(), wallet1);
    }

    #[test]
    #[should_panic(expected = "HostError")]
    fn test_non_anchor_cannot_submit() {
        let (env, admin, _anchor, oracle) = setup_env();
        let client = deploy(&env, &admin, &_anchor, &oracle);
        let wallet = Address::generate(&env);
        let fake_anchor = Address::generate(&env);
        let cred = make_credential(&env, &wallet, &fake_anchor, 1, 9999999999, false);
        client.submit_credential(&fake_anchor, &cred);
    }

    #[test]
    #[should_panic(expected = "HostError")]
    fn test_non_anchor_cannot_revoke() {
        let (env, admin, _anchor, oracle) = setup_env();
        let client = deploy(&env, &admin, &_anchor, &oracle);
        let wallet = Address::generate(&env);
        let fake_anchor = Address::generate(&env);
        let reason: String = "test".into_val(&env);
        client.revoke_credential(&fake_anchor, &wallet, &reason);
    }

    #[test]
    #[should_panic(expected = "HostError")]
    fn test_non_oracle_cannot_flag_sanctioned() {
        let (env, admin, anchor, _oracle) = setup_env();
        let client = deploy(&env, &admin, &anchor, &_oracle);
        let wallet = Address::generate(&env);
        let fake_oracle = Address::generate(&env);
        client.flag_sanctioned(&fake_oracle, &wallet);
    }

    #[test]
    fn test_full_lifecycle() {
        let (env, admin, anchor, oracle) = setup_env();
        let client = deploy(&env, &admin, &anchor, &oracle);
        let wallet = Address::generate(&env);
        let cred = make_credential(&env, &wallet, &anchor, 3, 9999999999, false);

        client.submit_credential(&anchor, &cred);
        assert!(client.get_credential(&wallet).is_some());

        client.flag_sanctioned(&oracle, &wallet);
        assert!(client.is_sanctioned(&wallet));

        let retrieved = client.get_credential(&wallet).unwrap();
        assert!(retrieved.is_sanctioned);

        let reason: String = "sanctions".into_val(&env);
        client.revoke_credential(&anchor, &wallet, &reason);
        let revoked = client.get_credential(&wallet).unwrap();
        assert!(revoked.is_sanctioned);
    }
}
