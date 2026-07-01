#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, vec, Address, BytesN, Env, String, Vec};

use arcm_types::{
    evaluate_transfer, ApprovalResponse, ApprovalStatus, AssetClass, ComplianceAction,
    ComplianceDecision, ComplianceEvent, IssuerRuleConfig, ReasonCode,
};

fn u64_to_string(env: &Env, n: u64) -> String {
    if n == 0 {
        return String::from_str(env, "0");
    }
    let mut buf = [0u8; 20];
    let mut i = buf.len();
    let mut x = n;
    loop {
        i -= 1;
        buf[i] = b'0' + (x % 10) as u8;
        x /= 10;
        if x == 0 {
            break;
        }
    }
    let s = core::str::from_utf8(&buf[i..]).expect("valid utf8");
    String::from_str(env, s)
}

use arcm_audit::AuditLedgerContractClient;
use arcm_enforcement::EnforcementEngineContractClient;
use arcm_geo::CountryResolverContractClient;
use arcm_jurisdiction::JurisdictionEngineContractClient;
use arcm_kyc_oracle::KycOracleContractClient;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    JurisdictionEngine,
    KycOracle,
    EnforcementEngine,
    EnforcementAuthority,
    AuditLedger,
    GeoResolver,
    AssetConfig(Address),
    RegisteredAssets,
}

#[contract]
pub struct GatewayContract;

#[contractimpl]
impl GatewayContract {
    #[allow(clippy::too_many_arguments)]
    pub fn __constructor(
        env: Env,
        admin: Address,
        jurisdiction_engine: Address,
        kyc_oracle: Address,
        enforcement_engine: Address,
        enforcement_authority: Address,
        audit_ledger: Address,
        geo_resolver: Address,
    ) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::JurisdictionEngine, &jurisdiction_engine);
        env.storage()
            .instance()
            .set(&DataKey::KycOracle, &kyc_oracle);
        env.storage()
            .instance()
            .set(&DataKey::EnforcementEngine, &enforcement_engine);
        env.storage()
            .instance()
            .set(&DataKey::EnforcementAuthority, &enforcement_authority);
        env.storage()
            .instance()
            .set(&DataKey::AuditLedger, &audit_ledger);
        env.storage()
            .instance()
            .set(&DataKey::GeoResolver, &geo_resolver);
    }

    pub fn approve(
        env: Env,
        sender: Address,
        receiver: Address,
        asset_contract: Address,
        amount: u128,
        tx_hash: BytesN<32>,
    ) -> ApprovalResponse {
        let geo: Address = env
            .storage()
            .instance()
            .get(&DataKey::GeoResolver)
            .expect("geo not set");
        let geo_client = CountryResolverContractClient::new(&env, &geo);

        let sender_country = geo_client.resolve_country(&sender);
        let receiver_country = geo_client.resolve_country(&receiver);

        let config: IssuerRuleConfig = env
            .storage()
            .instance()
            .get(&DataKey::AssetConfig(asset_contract.clone()))
            .expect("asset not registered");

        let jurisdiction: Address = env
            .storage()
            .instance()
            .get(&DataKey::JurisdictionEngine)
            .expect("jurisdiction not set");
        let jur_client = JurisdictionEngineContractClient::new(&env, &jurisdiction);

        let sender_rule = jur_client.get_rule(&sender_country, &config.asset_class);
        let receiver_rule = jur_client.get_rule(&receiver_country, &config.asset_class);

        let kyc_oracle: Address = env
            .storage()
            .instance()
            .get(&DataKey::KycOracle)
            .expect("kyc oracle not set");
        let kyc_client = KycOracleContractClient::new(&env, &kyc_oracle);

        let sender_cred = kyc_client.get_kyc_status(&sender);
        let receiver_cred = kyc_client.get_kyc_status(&receiver);

        let sender_kyc_tier = match &sender_cred {
            Some(c) => c.tier,
            None => 0,
        };
        let receiver_kyc_tier = match &receiver_cred {
            Some(c) => c.tier,
            None => 0,
        };
        let receiver_kyc_expires_at = match &receiver_cred {
            Some(c) => c.expires_at,
            None => u64::MAX,
        };

        if config.require_whitelist {
            let enforcement: Address = env
                .storage()
                .instance()
                .get(&DataKey::EnforcementEngine)
                .expect("enforcement not set");
            let enf_client = EnforcementEngineContractClient::new(&env, &enforcement);
            if !enf_client.is_whitelisted(&asset_contract, &receiver) {
                return Self::log_and_return(
                    &env,
                    &asset_contract,
                    &sender,
                    &receiver,
                    amount,
                    tx_hash,
                    sender_country,
                    receiver_country,
                    sender_kyc_tier,
                    receiver_kyc_tier,
                    sender_rule.version.max(receiver_rule.version),
                    ApprovalResponse {
                        status: ApprovalStatus::Rejected,
                        reason_code: ReasonCode::NotWhitelisted,
                        revised_amount: None,
                        audit_ref: String::from_str(&env, ""),
                    },
                );
            }
        }

        let timestamp = env.ledger().timestamp();
        let decision = evaluate_transfer(
            &sender_rule,
            &receiver_rule,
            amount,
            sender_kyc_tier,
            receiver_kyc_tier,
            receiver_kyc_expires_at,
            timestamp,
            None,
            None,
        );

        match &decision {
            ComplianceDecision::Approve => Self::log_and_return(
                &env,
                &asset_contract,
                &sender,
                &receiver,
                amount,
                tx_hash,
                sender_country,
                receiver_country,
                sender_kyc_tier,
                receiver_kyc_tier,
                sender_rule.version.max(receiver_rule.version),
                ApprovalResponse {
                    status: ApprovalStatus::Approved,
                    reason_code: ReasonCode::None,
                    revised_amount: None,
                    audit_ref: String::from_str(&env, ""),
                },
            ),
            ComplianceDecision::Reject(code) => Self::log_and_return(
                &env,
                &asset_contract,
                &sender,
                &receiver,
                amount,
                tx_hash,
                sender_country,
                receiver_country,
                sender_kyc_tier,
                receiver_kyc_tier,
                sender_rule.version.max(receiver_rule.version),
                ApprovalResponse {
                    status: ApprovalStatus::Rejected,
                    reason_code: code.clone(),
                    revised_amount: None,
                    audit_ref: String::from_str(&env, ""),
                },
            ),
            ComplianceDecision::Lock(code) => {
                let enforcement: Address = env
                    .storage()
                    .instance()
                    .get(&DataKey::EnforcementEngine)
                    .expect("enforcement not set");
                let enf_client = EnforcementEngineContractClient::new(&env, &enforcement);
                let enf_authority: Address = env
                    .storage()
                    .instance()
                    .get(&DataKey::EnforcementAuthority)
                    .expect("enforcement authority not set");
                enf_client.lock_asset(&enf_authority, &asset_contract, &receiver, code, &None);

                Self::log_and_return(
                    &env,
                    &asset_contract,
                    &sender,
                    &receiver,
                    amount,
                    tx_hash,
                    sender_country,
                    receiver_country,
                    sender_kyc_tier,
                    receiver_kyc_tier,
                    sender_rule.version.max(receiver_rule.version),
                    ApprovalResponse {
                        status: ApprovalStatus::Rejected,
                        reason_code: code.clone(),
                        revised_amount: None,
                        audit_ref: String::from_str(&env, ""),
                    },
                )
            }
            ComplianceDecision::Clawback(code) => {
                let enforcement: Address = env
                    .storage()
                    .instance()
                    .get(&DataKey::EnforcementEngine)
                    .expect("enforcement not set");
                let enf_client = EnforcementEngineContractClient::new(&env, &enforcement);
                let enf_authority: Address = env
                    .storage()
                    .instance()
                    .get(&DataKey::EnforcementAuthority)
                    .expect("enforcement authority not set");
                enf_client.execute_clawback(
                    &enf_authority,
                    &asset_contract,
                    &receiver,
                    &amount,
                    code,
                    &sender,
                );

                Self::log_and_return(
                    &env,
                    &asset_contract,
                    &sender,
                    &receiver,
                    amount,
                    tx_hash,
                    sender_country,
                    receiver_country,
                    sender_kyc_tier,
                    receiver_kyc_tier,
                    sender_rule.version.max(receiver_rule.version),
                    ApprovalResponse {
                        status: ApprovalStatus::Rejected,
                        reason_code: code.clone(),
                        revised_amount: None,
                        audit_ref: String::from_str(&env, ""),
                    },
                )
            }
            ComplianceDecision::PendingIssuerApproval => Self::log_and_return(
                &env,
                &asset_contract,
                &sender,
                &receiver,
                amount,
                tx_hash,
                sender_country,
                receiver_country,
                sender_kyc_tier,
                receiver_kyc_tier,
                sender_rule.version.max(receiver_rule.version),
                ApprovalResponse {
                    status: ApprovalStatus::Pending,
                    reason_code: ReasonCode::IssuerApprovalRequired,
                    revised_amount: None,
                    audit_ref: String::from_str(&env, ""),
                },
            ),
            ComplianceDecision::Revise(max_amount) => Self::log_and_return(
                &env,
                &asset_contract,
                &sender,
                &receiver,
                amount,
                tx_hash,
                sender_country,
                receiver_country,
                sender_kyc_tier,
                receiver_kyc_tier,
                sender_rule.version.max(receiver_rule.version),
                ApprovalResponse {
                    status: ApprovalStatus::Revised,
                    reason_code: ReasonCode::AmountExceedsJurisdictionCap,
                    revised_amount: Some(*max_amount),
                    audit_ref: String::from_str(&env, ""),
                },
            ),
        }
    }

    pub fn register_asset(
        env: Env,
        issuer: Address,
        asset_contract: Address,
        _asset_class: AssetClass,
        rule_config: IssuerRuleConfig,
    ) {
        issuer.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::AssetConfig(asset_contract.clone()), &rule_config);

        let existing: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::RegisteredAssets)
            .unwrap_or(vec![&env]);
        let exists = (0..existing.len()).any(|i| existing.get(i).unwrap() == asset_contract);
        if !exists {
            let mut assets = existing;
            assets.push_back(asset_contract);
            env.storage()
                .instance()
                .set(&DataKey::RegisteredAssets, &assets);
        }
    }

    pub fn deregister_asset(env: Env, issuer: Address, asset_contract: Address) {
        issuer.require_auth();
        env.storage()
            .instance()
            .remove(&DataKey::AssetConfig(asset_contract.clone()));

        let assets: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::RegisteredAssets)
            .unwrap_or(vec![&env]);
        let mut new_assets: Vec<Address> = vec![&env];
        for i in 0..assets.len() {
            let a = assets.get(i).unwrap();
            if a != asset_contract {
                new_assets.push_back(a);
            }
        }
        env.storage()
            .instance()
            .set(&DataKey::RegisteredAssets, &new_assets);
    }

    pub fn get_asset_config(env: Env, asset_contract: Address) -> IssuerRuleConfig {
        env.storage()
            .instance()
            .get(&DataKey::AssetConfig(asset_contract))
            .expect("asset not registered")
    }

    #[allow(clippy::too_many_arguments)]
    fn log_and_return(
        env: &Env,
        asset_contract: &Address,
        sender: &Address,
        receiver: &Address,
        amount: u128,
        tx_hash: BytesN<32>,
        sender_country: String,
        receiver_country: String,
        sender_kyc_tier: u32,
        receiver_kyc_tier: u32,
        rule_version: u32,
        response: ApprovalResponse,
    ) -> ApprovalResponse {
        let audit: Address = env
            .storage()
            .instance()
            .get(&DataKey::AuditLedger)
            .expect("audit not set");
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("admin not set");
        let audit_client = AuditLedgerContractClient::new(env, &audit);

        let action = match response.status {
            ApprovalStatus::Approved => ComplianceAction::Approve,
            ApprovalStatus::Rejected => ComplianceAction::Reject,
            ApprovalStatus::Revised => ComplianceAction::Approve,
            ApprovalStatus::Pending => ComplianceAction::Approve,
        };

        let event = ComplianceEvent::new(
            0,
            env.ledger().timestamp(),
            asset_contract.clone(),
            action,
            sender.clone(),
            receiver.clone(),
            amount,
            response.reason_code.clone(),
            sender_country,
            receiver_country,
            sender_kyc_tier,
            receiver_kyc_tier,
            rule_version,
            tx_hash,
        );

        let event_id = audit_client.log_event(&admin, &event);

        let mut resp = response;
        resp.audit_ref = u64_to_string(env, event_id);
        resp
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::{BytesN, Env, IntoVal};

    use arcm_audit::AuditLedgerContract;
    use arcm_credentials::{CredentialRegistryContract, CredentialRegistryContractClient};
    use arcm_enforcement::EnforcementEngineContract;
    use arcm_geo::CountryResolverContract;
    use arcm_jurisdiction::JurisdictionEngineContract;
    use arcm_kyc_oracle::KycOracleContract;
    use arcm_types::{JurisdictionRule, TransferPolicy};

    fn zero_hash(env: &Env) -> BytesN<32> {
        BytesN::from_array(env, &[0u8; 32])
    }

    fn make_country(env: &Env, code: &str) -> String {
        code.into_val(env)
    }

    fn make_rule(
        env: &Env,
        country: &str,
        policy: TransferPolicy,
        min_kyc: u32,
    ) -> JurisdictionRule {
        JurisdictionRule::new(
            make_country(env, country),
            AssetClass::Generic,
            policy,
            min_kyc,
            None,
            None,
            None,
            false,
            false,
            1,
            1_700_000_000,
        )
    }

    #[allow(dead_code)]
    struct TestEnv {
        env: Env,
        admin: Address,
        anchor: Address,
        oracle_auth: Address,
        authority: Address,
        geo_id: Address,
        jur_id: Address,
        cred_id: Address,
        kyc_id: Address,
        enf_id: Address,
        audit_id: Address,
        gateway_id: Address,
        gateway_client: GatewayContractClient<'static>,
    }

    fn setup_full_env() -> TestEnv {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        env.ledger().set_timestamp(1_700_000_000);
        let admin = Address::generate(&env);
        let anchor = Address::generate(&env);
        let oracle_auth = Address::generate(&env);
        let authority = Address::generate(&env);

        let geo_id = env.register(CountryResolverContract, (admin.clone(),));

        let jur_id = env.register(JurisdictionEngineContract, (admin.clone(),));

        let cred_id = env.register(
            CredentialRegistryContract,
            (admin.clone(), anchor.clone(), oracle_auth.clone()),
        );

        let kyc_id = env.register(KycOracleContract, (admin.clone(), cred_id.clone()));

        let enf_id = env.register(
            EnforcementEngineContract,
            (admin.clone(), authority.clone()),
        );

        let audit_id = env.register(AuditLedgerContract, (admin.clone(), admin.clone()));

        let gateway_id = env.register(
            GatewayContract,
            (
                admin.clone(),
                jur_id.clone(),
                kyc_id.clone(),
                enf_id.clone(),
                authority.clone(),
                audit_id.clone(),
                geo_id.clone(),
            ),
        );

        let gateway_client = GatewayContractClient::new(&env, &gateway_id);

        TestEnv {
            env,
            admin,
            anchor,
            oracle_auth,
            authority,
            geo_id,
            jur_id,
            cred_id,
            kyc_id,
            enf_id,
            audit_id,
            gateway_id,
            gateway_client,
        }
    }

    #[test]
    fn test_register_asset() {
        let te = setup_full_env();
        let issuer = Address::generate(&te.env);
        let asset = Address::generate(&te.env);
        let jurs = vec![&te.env, make_country(&te.env, "US")];
        let config = IssuerRuleConfig {
            asset_class: AssetClass::Generic,
            jurisdictions: jurs,
            require_kyc: false,
            require_whitelist: false,
            clawback_enabled: false,
        };

        te.gateway_client
            .register_asset(&issuer, &asset, &AssetClass::Generic, &config);
        let retrieved = te.gateway_client.get_asset_config(&asset);
        assert_eq!(retrieved.asset_class, AssetClass::Generic);
    }

    #[test]
    fn test_deregister_asset() {
        let te = setup_full_env();
        let issuer = Address::generate(&te.env);
        let asset = Address::generate(&te.env);
        let jurs = vec![&te.env, make_country(&te.env, "US")];
        let config = IssuerRuleConfig {
            asset_class: AssetClass::Generic,
            jurisdictions: jurs,
            require_kyc: false,
            require_whitelist: false,
            clawback_enabled: false,
        };

        te.gateway_client
            .register_asset(&issuer, &asset, &AssetClass::Generic, &config);
        let _registered = te.gateway_client.get_asset_config(&asset);
        assert_eq!(_registered.asset_class, AssetClass::Generic);

        te.gateway_client.deregister_asset(&issuer, &asset);
    }

    #[test]
    fn test_approve_happy_path() {
        let te = setup_full_env();
        let issuer = Address::generate(&te.env);
        let sender = Address::generate(&te.env);
        let receiver = Address::generate(&te.env);
        let asset = Address::generate(&te.env);

        let us_code: String = make_country(&te.env, "US");
        let de_code: String = make_country(&te.env, "DE");

        let geo_client = CountryResolverContractClient::new(&te.env, &te.geo_id);
        geo_client.set_country(&te.admin, &sender, &us_code);
        geo_client.set_country(&te.admin, &receiver, &de_code);

        let jur_client = JurisdictionEngineContractClient::new(&te.env, &te.jur_id);
        jur_client.set_rule(
            &te.admin,
            &make_country(&te.env, "US"),
            &AssetClass::Generic,
            &make_rule(&te.env, "US", TransferPolicy::Open, 0),
        );
        jur_client.set_rule(
            &te.admin,
            &make_country(&te.env, "DE"),
            &AssetClass::Generic,
            &make_rule(&te.env, "DE", TransferPolicy::Open, 0),
        );

        let jurs = vec![
            &te.env,
            make_country(&te.env, "US"),
            make_country(&te.env, "DE"),
        ];
        let config = IssuerRuleConfig {
            asset_class: AssetClass::Generic,
            jurisdictions: jurs,
            require_kyc: false,
            require_whitelist: false,
            clawback_enabled: false,
        };
        te.gateway_client
            .register_asset(&issuer, &asset, &AssetClass::Generic, &config);

        let response =
            te.gateway_client
                .approve(&sender, &receiver, &asset, &1000, &zero_hash(&te.env));
        assert_eq!(response.status, ApprovalStatus::Approved);
        assert!(!response.audit_ref.is_empty());
    }

    #[test]
    fn test_approve_rejected_no_kyc() {
        let te = setup_full_env();
        let issuer = Address::generate(&te.env);
        let sender = Address::generate(&te.env);
        let receiver = Address::generate(&te.env);
        let asset = Address::generate(&te.env);

        let us_code: String = make_country(&te.env, "US");
        let de_code: String = make_country(&te.env, "DE");

        let geo_client = CountryResolverContractClient::new(&te.env, &te.geo_id);
        geo_client.set_country(&te.admin, &sender, &us_code);
        geo_client.set_country(&te.admin, &receiver, &de_code);

        let jur_client = JurisdictionEngineContractClient::new(&te.env, &te.jur_id);
        jur_client.set_rule(
            &te.admin,
            &make_country(&te.env, "US"),
            &AssetClass::Generic,
            &make_rule(&te.env, "US", TransferPolicy::Open, 3),
        );
        jur_client.set_rule(
            &te.admin,
            &make_country(&te.env, "DE"),
            &AssetClass::Generic,
            &make_rule(&te.env, "DE", TransferPolicy::Open, 3),
        );

        let jurs = vec![
            &te.env,
            make_country(&te.env, "US"),
            make_country(&te.env, "DE"),
        ];
        let config = IssuerRuleConfig {
            asset_class: AssetClass::Generic,
            jurisdictions: jurs,
            require_kyc: false,
            require_whitelist: false,
            clawback_enabled: false,
        };
        te.gateway_client
            .register_asset(&issuer, &asset, &AssetClass::Generic, &config);

        let response =
            te.gateway_client
                .approve(&sender, &receiver, &asset, &1000, &zero_hash(&te.env));
        assert_eq!(response.status, ApprovalStatus::Rejected);
        assert_eq!(response.reason_code, ReasonCode::InsufficientKycTier);
    }

    #[test]
    fn test_approve_sanctioned_jurisdiction() {
        let te = setup_full_env();
        let issuer = Address::generate(&te.env);
        let sender = Address::generate(&te.env);
        let receiver = Address::generate(&te.env);
        let asset = Address::generate(&te.env);

        let geo_client = CountryResolverContractClient::new(&te.env, &te.geo_id);
        geo_client.set_country(&te.admin, &sender, &make_country(&te.env, "IR"));
        geo_client.set_country(&te.admin, &receiver, &make_country(&te.env, "US"));

        let jur_client = JurisdictionEngineContractClient::new(&te.env, &te.jur_id);
        jur_client.set_rule(
            &te.admin,
            &make_country(&te.env, "IR"),
            &AssetClass::Generic,
            &make_rule(&te.env, "IR", TransferPolicy::Sanctioned, 1),
        );
        jur_client.set_rule(
            &te.admin,
            &make_country(&te.env, "US"),
            &AssetClass::Generic,
            &make_rule(&te.env, "US", TransferPolicy::Open, 1),
        );

        let jurs = vec![&te.env, make_country(&te.env, "US")];
        let config = IssuerRuleConfig {
            asset_class: AssetClass::Generic,
            jurisdictions: jurs,
            require_kyc: false,
            require_whitelist: false,
            clawback_enabled: false,
        };
        te.gateway_client
            .register_asset(&issuer, &asset, &AssetClass::Generic, &config);

        let response =
            te.gateway_client
                .approve(&sender, &receiver, &asset, &1000, &zero_hash(&te.env));
        assert_eq!(response.status, ApprovalStatus::Rejected);
        assert_eq!(response.reason_code, ReasonCode::SanctionedJurisdiction);
    }

    #[test]
    fn test_approve_kyc_expired_triggers_lock() {
        let te = setup_full_env();
        let issuer = Address::generate(&te.env);
        let sender = Address::generate(&te.env);
        let receiver = Address::generate(&te.env);
        let asset = Address::generate(&te.env);

        let geo_client = CountryResolverContractClient::new(&te.env, &te.geo_id);
        geo_client.set_country(&te.admin, &sender, &make_country(&te.env, "US"));
        geo_client.set_country(&te.admin, &receiver, &make_country(&te.env, "DE"));

        let jur_client = JurisdictionEngineContractClient::new(&te.env, &te.jur_id);
        jur_client.set_rule(
            &te.admin,
            &make_country(&te.env, "US"),
            &AssetClass::Generic,
            &make_rule(&te.env, "US", TransferPolicy::Open, 1),
        );
        jur_client.set_rule(
            &te.admin,
            &make_country(&te.env, "DE"),
            &AssetClass::Generic,
            &make_rule(&te.env, "DE", TransferPolicy::Open, 1),
        );

        let cred_client = CredentialRegistryContractClient::new(&te.env, &te.cred_id);
        let wallet = receiver.clone();
        let kyc_cred = arcm_types::KycCredential {
            wallet: wallet.clone(),
            tier: 1,
            country_code: make_country(&te.env, "DE"),
            credential_hash: zero_hash(&te.env),
            issued_at: 1_600_000_000,
            expires_at: 1_650_000_000,
            issuer_anchor: te.anchor.clone(),
            is_sanctioned: false,
            sanctions_lists_checked: vec![&te.env, "OFAC".into_val(&te.env)],
        };
        cred_client.submit_credential(&te.anchor, &kyc_cred);

        let jurs = vec![
            &te.env,
            make_country(&te.env, "US"),
            make_country(&te.env, "DE"),
        ];
        let config = IssuerRuleConfig {
            asset_class: AssetClass::Generic,
            jurisdictions: jurs,
            require_kyc: false,
            require_whitelist: false,
            clawback_enabled: false,
        };
        te.gateway_client
            .register_asset(&issuer, &asset, &AssetClass::Generic, &config);

        let response =
            te.gateway_client
                .approve(&sender, &receiver, &asset, &1000, &zero_hash(&te.env));
        assert_eq!(response.status, ApprovalStatus::Rejected);
        assert_eq!(response.reason_code, ReasonCode::KycExpired);
    }

    #[test]
    fn test_approve_audit_event_logged() {
        let te = setup_full_env();
        let issuer = Address::generate(&te.env);
        let sender = Address::generate(&te.env);
        let receiver = Address::generate(&te.env);
        let asset = Address::generate(&te.env);

        let geo_client = CountryResolverContractClient::new(&te.env, &te.geo_id);
        geo_client.set_country(&te.admin, &sender, &make_country(&te.env, "US"));
        geo_client.set_country(&te.admin, &receiver, &make_country(&te.env, "DE"));

        let jur_client = JurisdictionEngineContractClient::new(&te.env, &te.jur_id);
        jur_client.set_rule(
            &te.admin,
            &make_country(&te.env, "US"),
            &AssetClass::Generic,
            &make_rule(&te.env, "US", TransferPolicy::Open, 0),
        );
        jur_client.set_rule(
            &te.admin,
            &make_country(&te.env, "DE"),
            &AssetClass::Generic,
            &make_rule(&te.env, "DE", TransferPolicy::Open, 0),
        );

        let jurs = vec![
            &te.env,
            make_country(&te.env, "US"),
            make_country(&te.env, "DE"),
        ];
        let config = IssuerRuleConfig {
            asset_class: AssetClass::Generic,
            jurisdictions: jurs,
            require_kyc: false,
            require_whitelist: false,
            clawback_enabled: false,
        };
        te.gateway_client
            .register_asset(&issuer, &asset, &AssetClass::Generic, &config);

        te.gateway_client
            .approve(&sender, &receiver, &asset, &1000, &zero_hash(&te.env));

        let audit_client = AuditLedgerContractClient::new(&te.env, &te.audit_id);
        let events = audit_client.query_events(&asset, &1, &10);
        assert_eq!(events.len(), 1);
        assert_eq!(events.get(0).unwrap().action, ComplianceAction::Approve);
    }

    #[test]
    fn test_non_issuer_cannot_deregister() {
        let te = setup_full_env();
        let issuer = Address::generate(&te.env);
        let attacker = Address::generate(&te.env);
        let asset = Address::generate(&te.env);
        let jurs = vec![&te.env, make_country(&te.env, "US")];
        let config = IssuerRuleConfig {
            asset_class: AssetClass::Generic,
            jurisdictions: jurs,
            require_kyc: false,
            require_whitelist: false,
            clawback_enabled: false,
        };

        te.gateway_client
            .register_asset(&issuer, &asset, &AssetClass::Generic, &config);
        te.gateway_client.deregister_asset(&attacker, &asset);
    }

    #[test]
    fn test_approve_with_whitelist_requirement() {
        let te = setup_full_env();
        let issuer = Address::generate(&te.env);
        let sender = Address::generate(&te.env);
        let receiver = Address::generate(&te.env);
        let asset = Address::generate(&te.env);

        let geo_client = CountryResolverContractClient::new(&te.env, &te.geo_id);
        geo_client.set_country(&te.admin, &sender, &make_country(&te.env, "US"));
        geo_client.set_country(&te.admin, &receiver, &make_country(&te.env, "DE"));

        let jur_client = JurisdictionEngineContractClient::new(&te.env, &te.jur_id);
        jur_client.set_rule(
            &te.admin,
            &make_country(&te.env, "US"),
            &AssetClass::Generic,
            &make_rule(&te.env, "US", TransferPolicy::Open, 1),
        );
        jur_client.set_rule(
            &te.admin,
            &make_country(&te.env, "DE"),
            &AssetClass::Generic,
            &make_rule(&te.env, "DE", TransferPolicy::Open, 1),
        );

        let jurs = vec![
            &te.env,
            make_country(&te.env, "US"),
            make_country(&te.env, "DE"),
        ];
        let config = IssuerRuleConfig {
            asset_class: AssetClass::Generic,
            jurisdictions: jurs,
            require_kyc: false,
            require_whitelist: true,
            clawback_enabled: false,
        };
        te.gateway_client
            .register_asset(&issuer, &asset, &AssetClass::Generic, &config);

        let response =
            te.gateway_client
                .approve(&sender, &receiver, &asset, &1000, &zero_hash(&te.env));
        assert_eq!(response.status, ApprovalStatus::Rejected);
        assert_eq!(response.reason_code, ReasonCode::NotWhitelisted);
    }

    #[test]
    fn test_full_lifecycle_integration() {
        let te = setup_full_env();
        let issuer = Address::generate(&te.env);
        let sender = Address::generate(&te.env);
        let receiver = Address::generate(&te.env);
        let asset = Address::generate(&te.env);

        let geo_client = CountryResolverContractClient::new(&te.env, &te.geo_id);
        geo_client.set_country(&te.admin, &sender, &make_country(&te.env, "US"));
        geo_client.set_country(&te.admin, &receiver, &make_country(&te.env, "DE"));

        let jur_client = JurisdictionEngineContractClient::new(&te.env, &te.jur_id);
        jur_client.set_rule(
            &te.admin,
            &make_country(&te.env, "US"),
            &AssetClass::Generic,
            &make_rule(&te.env, "US", TransferPolicy::Open, 1),
        );
        jur_client.set_rule(
            &te.admin,
            &make_country(&te.env, "DE"),
            &AssetClass::Generic,
            &make_rule(&te.env, "DE", TransferPolicy::Open, 1),
        );

        let cred_client = CredentialRegistryContractClient::new(&te.env, &te.cred_id);
        let kyc_cred = arcm_types::KycCredential {
            wallet: receiver.clone(),
            tier: 2,
            country_code: make_country(&te.env, "DE"),
            credential_hash: zero_hash(&te.env),
            issued_at: 1_600_000_000,
            expires_at: 1_800_000_000,
            issuer_anchor: te.anchor.clone(),
            is_sanctioned: false,
            sanctions_lists_checked: vec![&te.env, "OFAC".into_val(&te.env)],
        };
        cred_client.submit_credential(&te.anchor, &kyc_cred);

        let jurs = vec![
            &te.env,
            make_country(&te.env, "US"),
            make_country(&te.env, "DE"),
        ];
        let config = IssuerRuleConfig {
            asset_class: AssetClass::Generic,
            jurisdictions: jurs,
            require_kyc: false,
            require_whitelist: false,
            clawback_enabled: false,
        };
        te.gateway_client
            .register_asset(&issuer, &asset, &AssetClass::Generic, &config);

        let response =
            te.gateway_client
                .approve(&sender, &receiver, &asset, &1000, &zero_hash(&te.env));
        assert_eq!(response.status, ApprovalStatus::Approved);
        assert!(!response.audit_ref.is_empty());

        let audit_client = AuditLedgerContractClient::new(&te.env, &te.audit_id);
        let events = audit_client.query_events(&asset, &1, &10);
        assert_eq!(events.len(), 1);
        assert_eq!(events.get(0).unwrap().action, ComplianceAction::Approve);
        assert_eq!(events.get(0).unwrap().kyc_tier_receiver, 2);
    }
}
