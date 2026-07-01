#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, vec, Address, Env, String, Vec};
use arcm_types::{ClawbackRecord, LockRecord, LockType, ReasonCode};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Authority,
    LockRecord(Address, Address),
    LockedWallets(Address),
    ClawbackRecord(u64),
    NextClawbackId,
    ClawbackHistory(Address),
    Whitelisted(Address, Address),
    Blacklisted(Address, Address),
}

#[contract]
pub struct EnforcementEngineContract;

#[contractimpl]
impl EnforcementEngineContract {
    pub fn __constructor(env: Env, admin: Address, authority: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::Authority, &authority);
        env.storage()
            .instance()
            .set(&DataKey::NextClawbackId, &1u64);
    }

    pub fn lock_asset(
        env: Env,
        authority: Address,
        asset_contract: Address,
        wallet: Address,
        reason: ReasonCode,
        duration: Option<u64>,
    ) {
        authority.require_auth();
        Self::check_authority(&env, &authority);

        let record = LockRecord {
            wallet: wallet.clone(),
            asset_contract: asset_contract.clone(),
            locked_at: env.ledger().timestamp(),
            reason: reason.clone(),
            duration,
            lock_type: if duration.is_some() {
                LockType::Hard
            } else {
                LockType::Soft
            },
        };

        env.storage()
            .instance()
            .set(&DataKey::LockRecord(asset_contract.clone(), wallet.clone()), &record);

        let mut wallets: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::LockedWallets(asset_contract.clone()))
            .unwrap_or(vec![&env]);

        let exists = (0..wallets.len())
            .any(|i| wallets.get(i).unwrap() == wallet);
        if !exists {
            wallets.push_back(wallet);
            env.storage()
                .instance()
                .set(&DataKey::LockedWallets(asset_contract), &wallets);
        }
    }

    pub fn unlock_asset(
        env: Env,
        authority: Address,
        asset_contract: Address,
        wallet: Address,
    ) {
        authority.require_auth();
        Self::check_authority(&env, &authority);

        env.storage()
            .instance()
            .remove(&DataKey::LockRecord(asset_contract.clone(), wallet.clone()));

        let wallets: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::LockedWallets(asset_contract.clone()))
            .unwrap_or(vec![&env]);

        let mut new_wallets: Vec<Address> = vec![&env];
        for i in 0..wallets.len() {
            let w = wallets.get(i).unwrap();
            if w != wallet {
                new_wallets.push_back(w);
            }
        }
        env.storage()
            .instance()
            .set(&DataKey::LockedWallets(asset_contract), &new_wallets);
    }

    pub fn execute_clawback(
        env: Env,
        authority: Address,
        asset_contract: Address,
        holder: Address,
        amount: u128,
        reason: ReasonCode,
        destination: Address,
    ) {
        authority.require_auth();
        Self::check_authority(&env, &authority);

        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextClawbackId)
            .unwrap_or(1);
        env.storage()
            .instance()
            .set(&DataKey::NextClawbackId, &(id + 1));

        let record = ClawbackRecord {
            event_id: id,
            holder: holder.clone(),
            asset_contract: asset_contract.clone(),
            amount,
            reason: reason.clone(),
            destination: destination.clone(),
            executed_at: env.ledger().timestamp(),
        };

        env.storage()
            .instance()
            .set(&DataKey::ClawbackRecord(id), &record);

        let mut ids: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::ClawbackHistory(asset_contract.clone()))
            .unwrap_or(vec![&env]);
        ids.push_back(id);
        env.storage()
            .instance()
            .set(&DataKey::ClawbackHistory(asset_contract.clone()), &ids);

        env.storage()
            .instance()
            .remove(&DataKey::LockRecord(asset_contract.clone(), holder.clone()));

        let wallets: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::LockedWallets(asset_contract.clone()))
            .unwrap_or(vec![&env]);

        let mut new_wallets: Vec<Address> = vec![&env];
        for i in 0..wallets.len() {
            let w = wallets.get(i).unwrap();
            if w != holder {
                new_wallets.push_back(w);
            }
        }
        env.storage()
            .instance()
            .set(&DataKey::LockedWallets(asset_contract), &new_wallets);
    }

    pub fn whitelist_address(
        env: Env,
        issuer: Address,
        asset_contract: Address,
        wallet: Address,
        _tier_override: u32,
    ) {
        issuer.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::Whitelisted(asset_contract, wallet), &true);
    }

    pub fn blacklist_address(
        env: Env,
        issuer: Address,
        asset_contract: Address,
        wallet: Address,
        _reason: String,
    ) {
        issuer.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::Blacklisted(asset_contract, wallet), &true);
    }

    pub fn is_whitelisted(env: Env, asset_contract: Address, wallet: Address) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Whitelisted(asset_contract, wallet))
            .unwrap_or(false)
    }

    pub fn is_blacklisted(env: Env, asset_contract: Address, wallet: Address) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Blacklisted(asset_contract, wallet))
            .unwrap_or(false)
    }

    pub fn get_locked_wallets(env: Env, asset_contract: Address) -> Vec<LockRecord> {
        let wallets: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::LockedWallets(asset_contract.clone()))
            .unwrap_or(vec![&env]);

        let mut records: Vec<LockRecord> = vec![&env];
        for i in 0..wallets.len() {
            let wallet = wallets.get(i).unwrap();
            if let Some(record) = env
                .storage()
                .instance()
                .get(&DataKey::LockRecord(asset_contract.clone(), wallet))
            {
                records.push_back(record);
            }
        }
        records
    }

    pub fn get_clawback_history(env: Env, asset_contract: Address) -> Vec<ClawbackRecord> {
        let ids: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::ClawbackHistory(asset_contract))
            .unwrap_or(vec![&env]);

        let mut records: Vec<ClawbackRecord> = vec![&env];
        for i in 0..ids.len() {
            let id = ids.get(i).unwrap();
            if let Some(record) = env
                .storage()
                .instance()
                .get(&DataKey::ClawbackRecord(id))
            {
                records.push_back(record);
            }
        }
        records
    }

    fn check_authority(env: &Env, authority: &Address) {
        let stored: Address = env
            .storage()
            .instance()
            .get(&DataKey::Authority)
            .expect("authority not set");
        if stored != *authority {
            panic!("not authorized");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Env, IntoVal};

    fn setup_env() -> (Env, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let authority = Address::generate(&env);
        (env, admin, authority)
    }

    fn deploy<'a>(
        env: &'a Env,
        admin: &Address,
        authority: &Address,
    ) -> EnforcementEngineContractClient<'a> {
        let contract_id = env.register(
            EnforcementEngineContract,
            (admin.clone(), authority.clone()),
        );
        EnforcementEngineContractClient::new(env, &contract_id)
    }

    #[test]
    fn test_lock_asset() {
        let (env, admin, authority) = setup_env();
        let client = deploy(&env, &admin, &authority);
        let asset = Address::generate(&env);
        let wallet = Address::generate(&env);

        client.lock_asset(&authority, &asset, &wallet, &ReasonCode::SanctionedAddress, &None);

        let locked = client.get_locked_wallets(&asset);
        assert_eq!(locked.len(), 1);
        assert_eq!(locked.get(0).unwrap().wallet, wallet);
        assert_eq!(locked.get(0).unwrap().lock_type, LockType::Soft);
    }

    #[test]
    fn test_lock_asset_hard_lock() {
        let (env, admin, authority) = setup_env();
        let client = deploy(&env, &admin, &authority);
        let asset = Address::generate(&env);
        let wallet = Address::generate(&env);

        client.lock_asset(&authority, &asset, &wallet, &ReasonCode::KycExpired, &Some(86400u64));

        let locked = client.get_locked_wallets(&asset);
        assert_eq!(locked.len(), 1);
        assert_eq!(locked.get(0).unwrap().lock_type, LockType::Hard);
        assert_eq!(locked.get(0).unwrap().duration, Some(86400));
    }

    #[test]
    fn test_unlock_asset() {
        let (env, admin, authority) = setup_env();
        let client = deploy(&env, &admin, &authority);
        let asset = Address::generate(&env);
        let wallet = Address::generate(&env);

        client.lock_asset(&authority, &asset, &wallet, &ReasonCode::SanctionedAddress, &None);
        assert_eq!(client.get_locked_wallets(&asset).len(), 1);

        client.unlock_asset(&authority, &asset, &wallet);
        assert_eq!(client.get_locked_wallets(&asset).len(), 0);
    }

    #[test]
    fn test_lock_then_unlock_removes_record() {
        let (env, admin, authority) = setup_env();
        let client = deploy(&env, &admin, &authority);
        let asset = Address::generate(&env);
        let wallet = Address::generate(&env);

        client.lock_asset(&authority, &asset, &wallet, &ReasonCode::SanctionedAddress, &None);
        client.unlock_asset(&authority, &asset, &wallet);
        client.unlock_asset(&authority, &asset, &wallet);

        assert_eq!(client.get_locked_wallets(&asset).len(), 0);
    }

    #[test]
    fn test_execute_clawback() {
        let (env, admin, authority) = setup_env();
        let client = deploy(&env, &admin, &authority);
        let asset = Address::generate(&env);
        let holder = Address::generate(&env);
        let dest = Address::generate(&env);

        client.lock_asset(&authority, &asset, &holder, &ReasonCode::SanctionedAddress, &None);
        client.execute_clawback(&authority, &asset, &holder, &1000, &ReasonCode::SanctionedAddress, &dest);

        let history = client.get_clawback_history(&asset);
        assert_eq!(history.len(), 1);
        assert_eq!(history.get(0).unwrap().holder, holder);
        assert_eq!(history.get(0).unwrap().amount, 1000);
        assert_eq!(history.get(0).unwrap().destination, dest);

        let locked = client.get_locked_wallets(&asset);
        assert_eq!(locked.len(), 0);
    }

    #[test]
    fn test_get_locked_wallets_multiple() {
        let (env, admin, authority) = setup_env();
        let client = deploy(&env, &admin, &authority);
        let asset = Address::generate(&env);
        let wallet1 = Address::generate(&env);
        let wallet2 = Address::generate(&env);

        client.lock_asset(&authority, &asset, &wallet1, &ReasonCode::KycExpired, &None);
        client.lock_asset(&authority, &asset, &wallet2, &ReasonCode::SanctionedAddress, &None);

        let locked = client.get_locked_wallets(&asset);
        assert_eq!(locked.len(), 2);
    }

    #[test]
    fn test_whitelist_address() {
        let (env, admin, _authority) = setup_env();
        let client = deploy(&env, &admin, &_authority);
        let asset = Address::generate(&env);
        let wallet = Address::generate(&env);
        let issuer = Address::generate(&env);

        assert!(!client.is_whitelisted(&asset, &wallet));
        client.whitelist_address(&issuer, &asset, &wallet, &1);
        assert!(client.is_whitelisted(&asset, &wallet));
    }

    #[test]
    fn test_blacklist_address() {
        let (env, admin, _authority) = setup_env();
        let client = deploy(&env, &admin, &_authority);
        let asset = Address::generate(&env);
        let wallet = Address::generate(&env);
        let issuer = Address::generate(&env);
        let reason: String = "fraud".into_val(&env);

        assert!(!client.is_blacklisted(&asset, &wallet));
        client.blacklist_address(&issuer, &asset, &wallet, &reason);
        assert!(client.is_blacklisted(&asset, &wallet));
    }

    #[test]
    fn test_get_clawback_history_multiple() {
        let (env, admin, authority) = setup_env();
        let client = deploy(&env, &admin, &authority);
        let asset = Address::generate(&env);
        let holder1 = Address::generate(&env);
        let holder2 = Address::generate(&env);
        let dest = Address::generate(&env);

        client.execute_clawback(&authority, &asset, &holder1, &500, &ReasonCode::SanctionedAddress, &dest);
        client.execute_clawback(&authority, &asset, &holder2, &1000, &ReasonCode::KycExpired, &dest);

        let history = client.get_clawback_history(&asset);
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_multiple_assets_separate_tracking() {
        let (env, admin, authority) = setup_env();
        let client = deploy(&env, &admin, &authority);
        let asset1 = Address::generate(&env);
        let asset2 = Address::generate(&env);
        let wallet = Address::generate(&env);

        client.lock_asset(&authority, &asset1, &wallet, &ReasonCode::KycExpired, &None);
        assert_eq!(client.get_locked_wallets(&asset1).len(), 1);
        assert_eq!(client.get_locked_wallets(&asset2).len(), 0);
    }

    #[test]
    #[should_panic(expected = "HostError")]
    fn test_non_authority_cannot_lock() {
        let (env, admin, _authority) = setup_env();
        let client = deploy(&env, &admin, &_authority);
        let fake = Address::generate(&env);
        let asset = Address::generate(&env);
        let wallet = Address::generate(&env);

        client.lock_asset(&fake, &asset, &wallet, &ReasonCode::KycExpired, &None);
    }

    #[test]
    #[should_panic(expected = "HostError")]
    fn test_non_authority_cannot_unlock() {
        let (env, admin, _authority) = setup_env();
        let client = deploy(&env, &admin, &_authority);
        let fake = Address::generate(&env);
        let asset = Address::generate(&env);
        let wallet = Address::generate(&env);

        client.unlock_asset(&fake, &asset, &wallet);
    }

    #[test]
    #[should_panic(expected = "HostError")]
    fn test_non_authority_cannot_clawback() {
        let (env, admin, _authority) = setup_env();
        let client = deploy(&env, &admin, &_authority);
        let fake = Address::generate(&env);
        let asset = Address::generate(&env);
        let holder = Address::generate(&env);
        let dest = Address::generate(&env);

        client.execute_clawback(&fake, &asset, &holder, &1000, &ReasonCode::SanctionedAddress, &dest);
    }
}
