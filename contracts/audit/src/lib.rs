#![no_std]
use arcm_types::{ComplianceEvent, ComplianceReport};
use soroban_sdk::{contract, contractimpl, contracttype, vec, Address, Env, Vec};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Gateway,
    Event(u64),
    NextEventId,
    AssetEventIds(Address),
    WalletEventIds(Address),
}

#[contract]
pub struct AuditLedgerContract;

#[contractimpl]
impl AuditLedgerContract {
    /// Initializes the contract with an admin and a gateway address.
    /// Only the gateway can log compliance events.
    pub fn __constructor(env: Env, admin: Address, gateway: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Gateway, &gateway);
        env.storage().instance().set(&DataKey::NextEventId, &1u64);
    }

    /// Records a compliance event, indexes it by asset and wallet, and returns the assigned event ID.
    pub fn log_event(env: Env, caller: Address, event: ComplianceEvent) -> u64 {
        Self::check_gateway(&env, &caller);

        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextEventId)
            .unwrap_or(1);
        env.storage()
            .instance()
            .set(&DataKey::NextEventId, &(id + 1));

        let mut stored_event = event;
        stored_event.event_id = id;
        env.storage()
            .instance()
            .set(&DataKey::Event(id), &stored_event);

        let mut asset_ids: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::AssetEventIds(stored_event.asset_contract.clone()))
            .unwrap_or(vec![&env]);
        asset_ids.push_back(id);
        env.storage().instance().set(
            &DataKey::AssetEventIds(stored_event.asset_contract.clone()),
            &asset_ids,
        );

        let mut sender_ids: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::WalletEventIds(stored_event.sender.clone()))
            .unwrap_or(vec![&env]);
        sender_ids.push_back(id);
        env.storage().instance().set(
            &DataKey::WalletEventIds(stored_event.sender.clone()),
            &sender_ids,
        );

        let mut receiver_ids: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::WalletEventIds(stored_event.receiver.clone()))
            .unwrap_or(vec![&env]);
        receiver_ids.push_back(id);
        env.storage().instance().set(
            &DataKey::WalletEventIds(stored_event.receiver),
            &receiver_ids,
        );

        id
    }

    /// Queries compliance events for an asset with pagination (`from_id`, `limit`).
    pub fn query_events(
        env: Env,
        asset_contract: Address,
        from_id: u64,
        limit: u32,
    ) -> Vec<ComplianceEvent> {
        let ids: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::AssetEventIds(asset_contract))
            .unwrap_or(vec![&env]);

        let mut results: Vec<ComplianceEvent> = vec![&env];
        let mut count = 0u32;
        for i in 0..ids.len() {
            if count >= limit {
                break;
            }
            let event_id = ids.get(i).unwrap();
            if event_id >= from_id {
                if let Some(event) = env
                    .storage()
                    .instance()
                    .get::<DataKey, ComplianceEvent>(&DataKey::Event(event_id))
                {
                    results.push_back(event);
                    count += 1;
                }
            }
        }
        results
    }

    /// Queries compliance events involving a specific wallet with pagination (`from_id`, `limit`).
    pub fn query_wallet_events(
        env: Env,
        wallet: Address,
        from_id: u64,
        limit: u32,
    ) -> Vec<ComplianceEvent> {
        let ids: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::WalletEventIds(wallet))
            .unwrap_or(vec![&env]);

        let mut results: Vec<ComplianceEvent> = vec![&env];
        let mut count = 0u32;
        for i in 0..ids.len() {
            if count >= limit {
                break;
            }
            let event_id = ids.get(i).unwrap();
            if event_id >= from_id {
                if let Some(event) = env
                    .storage()
                    .instance()
                    .get::<DataKey, ComplianceEvent>(&DataKey::Event(event_id))
                {
                    results.push_back(event);
                    count += 1;
                }
            }
        }
        results
    }

    /// Generates a compliance report for an asset over a timestamp range, including all matching events.
    pub fn export_report(
        env: Env,
        asset_contract: Address,
        from_timestamp: u64,
        to_timestamp: u64,
    ) -> ComplianceReport {
        let ids: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::AssetEventIds(asset_contract.clone()))
            .unwrap_or(vec![&env]);

        let mut events: Vec<ComplianceEvent> = vec![&env];
        for i in 0..ids.len() {
            let event_id = ids.get(i).unwrap();
            if let Some(event) = env
                .storage()
                .instance()
                .get::<DataKey, ComplianceEvent>(&DataKey::Event(event_id))
            {
                if event.timestamp >= from_timestamp && event.timestamp <= to_timestamp {
                    events.push_back(event);
                }
            }
        }

        ComplianceReport {
            asset_contract,
            from_timestamp,
            to_timestamp,
            total_events: events.len() as u64,
            events,
        }
    }

    pub fn get_event_count(env: Env) -> u64 {
        let next: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextEventId)
            .unwrap_or(1);
        next - 1
    }

    fn check_gateway(env: &Env, caller: &Address) {
        let stored: Address = env
            .storage()
            .instance()
            .get(&DataKey::Gateway)
            .expect("gateway not set");
        if stored != *caller {
            panic!("not authorized");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use arcm_types::{ComplianceAction, ReasonCode};
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{BytesN, Env, IntoVal};

    fn zero_hash(env: &Env) -> BytesN<32> {
        BytesN::from_array(env, &[0u8; 32])
    }

    #[allow(clippy::too_many_arguments)]
    fn make_event(
        env: &Env,
        id: u64,
        asset: &Address,
        action: ComplianceAction,
        sender: &Address,
        receiver: &Address,
        timestamp: u64,
        reason: ReasonCode,
    ) -> ComplianceEvent {
        ComplianceEvent::new(
            id,
            timestamp,
            asset.clone(),
            action,
            sender.clone(),
            receiver.clone(),
            1000,
            reason,
            "US".into_val(env),
            "DE".into_val(env),
            2,
            2,
            1,
            zero_hash(env),
        )
    }

    fn setup_env() -> (Env, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let gateway = Address::generate(&env);
        (env, admin, gateway)
    }

    fn deploy<'a>(
        env: &'a Env,
        admin: &Address,
        gateway: &Address,
    ) -> AuditLedgerContractClient<'a> {
        let contract_id = env.register(AuditLedgerContract, (admin.clone(), gateway.clone()));
        AuditLedgerContractClient::new(env, &contract_id)
    }

    #[test]
    fn test_log_event() {
        let (env, admin, gateway) = setup_env();
        let client = deploy(&env, &admin, &gateway);
        let asset = Address::generate(&env);
        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);

        let event = make_event(
            &env,
            0,
            &asset,
            ComplianceAction::Approve,
            &sender,
            &receiver,
            1_700_000_000,
            ReasonCode::None,
        );
        let event_id = client.log_event(&gateway, &event);
        assert_eq!(event_id, 1);
        assert_eq!(client.get_event_count(), 1);
    }

    #[test]
    fn test_log_multiple_events_sequential_ids() {
        let (env, admin, gateway) = setup_env();
        let client = deploy(&env, &admin, &gateway);
        let asset = Address::generate(&env);
        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);

        let e1 = make_event(
            &env,
            0,
            &asset,
            ComplianceAction::Approve,
            &sender,
            &receiver,
            1_700_000_000,
            ReasonCode::None,
        );
        let e2 = make_event(
            &env,
            0,
            &asset,
            ComplianceAction::Reject,
            &sender,
            &receiver,
            1_700_000_001,
            ReasonCode::KycExpired,
        );

        let id1 = client.log_event(&gateway, &e1);
        let id2 = client.log_event(&gateway, &e2);
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(client.get_event_count(), 2);
    }

    #[test]
    fn test_query_events_pagination() {
        let (env, admin, gateway) = setup_env();
        let client = deploy(&env, &admin, &gateway);
        let asset = Address::generate(&env);
        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);

        for i in 0..10 {
            let event = make_event(
                &env,
                0,
                &asset,
                ComplianceAction::Approve,
                &sender,
                &receiver,
                1_700_000_000 + i as u64,
                ReasonCode::None,
            );
            client.log_event(&gateway, &event);
        }

        let first_5 = client.query_events(&asset, &1, &5);
        assert_eq!(first_5.len(), 5);
        assert_eq!(first_5.get(0).unwrap().event_id, 1);
        assert_eq!(first_5.get(4).unwrap().event_id, 5);

        let next_5 = client.query_events(&asset, &6, &5);
        assert_eq!(next_5.len(), 5);
        assert_eq!(next_5.get(0).unwrap().event_id, 6);
    }

    #[test]
    fn test_query_wallet_events() {
        let (env, admin, gateway) = setup_env();
        let client = deploy(&env, &admin, &gateway);
        let asset = Address::generate(&env);
        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);

        let e1 = make_event(
            &env,
            0,
            &asset,
            ComplianceAction::Approve,
            &sender,
            &receiver,
            1_700_000_000,
            ReasonCode::None,
        );
        let e2 = make_event(
            &env,
            0,
            &asset,
            ComplianceAction::Reject,
            &sender,
            &receiver,
            1_700_000_001,
            ReasonCode::KycExpired,
        );
        client.log_event(&gateway, &e1);
        client.log_event(&gateway, &e2);

        let sender_events = client.query_wallet_events(&sender, &1, &10);
        assert_eq!(sender_events.len(), 2);

        let receiver_events = client.query_wallet_events(&receiver, &1, &10);
        assert_eq!(receiver_events.len(), 2);
    }

    #[test]
    fn test_export_report() {
        let (env, admin, gateway) = setup_env();
        let client = deploy(&env, &admin, &gateway);
        let asset = Address::generate(&env);
        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);

        let e1 = make_event(
            &env,
            0,
            &asset,
            ComplianceAction::Approve,
            &sender,
            &receiver,
            1_700_000_000,
            ReasonCode::None,
        );
        let e2 = make_event(
            &env,
            0,
            &asset,
            ComplianceAction::Reject,
            &sender,
            &receiver,
            1_800_000_000,
            ReasonCode::KycExpired,
        );
        client.log_event(&gateway, &e1);
        client.log_event(&gateway, &e2);

        let report = client.export_report(&asset, &1_700_000_000, &1_750_000_000);
        assert_eq!(report.total_events, 1);
        assert_eq!(report.events.len(), 1);
        assert_eq!(report.events.get(0).unwrap().event_id, 1);
    }

    #[test]
    fn test_export_report_empty_range() {
        let (env, admin, gateway) = setup_env();
        let client = deploy(&env, &admin, &gateway);
        let asset = Address::generate(&env);
        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);

        let event = make_event(
            &env,
            0,
            &asset,
            ComplianceAction::Approve,
            &sender,
            &receiver,
            1_700_000_000,
            ReasonCode::None,
        );
        client.log_event(&gateway, &event);

        let report = client.export_report(&asset, &1_800_000_000, &1_900_000_000);
        assert_eq!(report.total_events, 0);
        assert_eq!(report.events.len(), 0);
    }

    #[test]
    fn test_multiple_assets_separate_events() {
        let (env, admin, gateway) = setup_env();
        let client = deploy(&env, &admin, &gateway);
        let asset1 = Address::generate(&env);
        let asset2 = Address::generate(&env);
        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);

        let e1 = make_event(
            &env,
            0,
            &asset1,
            ComplianceAction::Approve,
            &sender,
            &receiver,
            1_700_000_000,
            ReasonCode::None,
        );
        let e2 = make_event(
            &env,
            0,
            &asset2,
            ComplianceAction::Reject,
            &sender,
            &receiver,
            1_700_000_001,
            ReasonCode::KycExpired,
        );
        client.log_event(&gateway, &e1);
        client.log_event(&gateway, &e2);

        let asset1_events = client.query_events(&asset1, &1, &10);
        assert_eq!(asset1_events.len(), 1);

        let asset2_events = client.query_events(&asset2, &1, &10);
        assert_eq!(asset2_events.len(), 1);
    }

    #[test]
    fn test_empty_query_returns_empty() {
        let (env, admin, gateway) = setup_env();
        let client = deploy(&env, &admin, &gateway);
        let asset = Address::generate(&env);

        let events = client.query_events(&asset, &1, &10);
        assert_eq!(events.len(), 0);
    }

    #[test]
    #[should_panic(expected = "HostError")]
    fn test_non_gateway_cannot_log() {
        let (env, admin, _gateway) = setup_env();
        let client = deploy(&env, &admin, &_gateway);
        let fake = Address::generate(&env);
        let asset = Address::generate(&env);
        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);

        let event = make_event(
            &env,
            0,
            &asset,
            ComplianceAction::Approve,
            &sender,
            &receiver,
            1_700_000_000,
            ReasonCode::None,
        );
        client.log_event(&fake, &event);
    }

    #[test]
    fn test_append_only_no_deletes() {
        let (env, admin, gateway) = setup_env();
        let client = deploy(&env, &admin, &gateway);
        let asset = Address::generate(&env);
        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);

        let e1 = make_event(
            &env,
            0,
            &asset,
            ComplianceAction::Approve,
            &sender,
            &receiver,
            1_700_000_000,
            ReasonCode::None,
        );
        let e2 = make_event(
            &env,
            0,
            &asset,
            ComplianceAction::Reject,
            &sender,
            &receiver,
            1_700_000_001,
            ReasonCode::KycExpired,
        );
        client.log_event(&gateway, &e1);
        client.log_event(&gateway, &e2);

        assert_eq!(client.get_event_count(), 2);

        let all = client.query_events(&asset, &1, &100);
        assert_eq!(all.len(), 2);
    }
}
