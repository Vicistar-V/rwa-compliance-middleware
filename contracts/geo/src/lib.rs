#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    WalletCountry(Address),
}

#[contract]
pub struct CountryResolverContract;

#[contractimpl]
impl CountryResolverContract {
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn resolve_country(env: Env, address: Address) -> String {
        env.storage()
            .instance()
            .get(&DataKey::WalletCountry(address))
            .unwrap_or(String::from_str(&env, "XX"))
    }

    pub fn set_country(env: Env, admin: Address, address: Address, country_code: String) {
        admin.require_auth();
        Self::check_admin(&env, &admin);
        env.storage()
            .instance()
            .set(&DataKey::WalletCountry(address), &country_code);
    }

    pub fn remove_country(env: Env, admin: Address, address: Address) {
        admin.require_auth();
        Self::check_admin(&env, &admin);
        env.storage()
            .instance()
            .remove(&DataKey::WalletCountry(address));
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
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Env, IntoVal};

    fn setup_env<'a>() -> (Env, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        (env, admin)
    }

    fn deploy<'a>(
        env: &'a Env,
        admin: &Address,
    ) -> CountryResolverContractClient<'a> {
        let contract_id = env.register(CountryResolverContract, (admin.clone(),));
        CountryResolverContractClient::new(env, &contract_id)
    }

    #[test]
    fn test_resolve_unknown_wallet() {
        let (env, _admin) = setup_env();
        let wallet = Address::generate(&env);

        let cid = env.register(CountryResolverContract, (Address::generate(&env),));
        let c = CountryResolverContractClient::new(&env, &cid);

        let country = c.resolve_country(&wallet);
        assert_eq!(country, String::from_str(&env, "XX"));
    }

    #[test]
    fn test_set_and_resolve_round_trip() {
        let (env, admin) = setup_env();
        let client = deploy(&env, &admin);
        let wallet = Address::generate(&env);
        let country_code: String = "US".into_val(&env);

        client.set_country(&admin, &wallet, &country_code);
        let resolved = client.resolve_country(&wallet);
        assert_eq!(resolved, country_code);
    }

    #[test]
    fn test_remove_country() {
        let (env, admin) = setup_env();
        let client = deploy(&env, &admin);
        let wallet = Address::generate(&env);
        let country_code: String = "DE".into_val(&env);

        client.set_country(&admin, &wallet, &country_code);
        assert_eq!(client.resolve_country(&wallet), country_code);

        client.remove_country(&admin, &wallet);
        assert_eq!(
            client.resolve_country(&wallet),
            String::from_str(&env, "XX")
        );
    }

    #[test]
    #[should_panic(expected = "not authorized")]
    fn test_non_admin_cannot_set_country() {
        let (env, admin) = setup_env();
        let client = deploy(&env, &admin);
        let attacker = Address::generate(&env);
        let wallet = Address::generate(&env);
        let country_code: String = "US".into_val(&env);

        client.set_country(&attacker, &wallet, &country_code);
    }

    #[test]
    #[should_panic(expected = "not authorized")]
    fn test_non_admin_cannot_remove_country() {
        let (env, admin) = setup_env();
        let client = deploy(&env, &admin);
        let attacker = Address::generate(&env);
        let wallet = Address::generate(&env);

        client.remove_country(&attacker, &wallet);
    }

    #[test]
    fn test_resolve_multiple_wallets() {
        let (env, admin) = setup_env();
        let client = deploy(&env, &admin);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let us: String = "US".into_val(&env);
        let de: String = "DE".into_val(&env);

        client.set_country(&admin, &alice, &us);
        client.set_country(&admin, &bob, &de);

        assert_eq!(client.resolve_country(&alice), us);
        assert_eq!(client.resolve_country(&bob), de);
    }
}
