#![no_std]

use soroban_sdk::{contracttype, Address, BytesN, String, Vec};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Defines the permissible transfer policies for a jurisdiction/asset pair.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransferPolicy {
    /// No restrictions on transfers.
    Open,
    /// Transfers are restricted but may be allowed under certain conditions.
    Restricted,
    /// Only accredited investors may receive or hold the asset.
    AccreditedOnly,
    /// Only institutional entities may receive or hold the asset.
    InstitutionalOnly,
    /// Transfers are completely prohibited for this jurisdiction.
    Prohibited,
    /// The jurisdiction is sanctioned; transfers are rejected outright.
    Sanctioned,
}

/// Broad category of the real-world asset.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetClass {
    RealEstate,
    Commodity,
    Equity,
    Debt,
    Fund,
    Generic,
}

/// Machine-readable reason for a compliance decision.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReasonCode {
    None,
    SanctionedJurisdiction,
    ProhibitedJurisdiction,
    AmountExceedsJurisdictionCap,
    HoldingPeriodNotMet,
    HoldingsCapExceeded,
    InsufficientKycTier,
    KycExpired,
    KycNotFound,
    SanctionedAddress,
    NotWhitelisted,
    IssuerApprovalRequired,
    AssetClassRestricted,
    ContractPaused,
}

/// Outcome of a compliance check on a proposed transfer.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComplianceDecision {
    /// Transfer is approved.
    Approve,
    /// Transfer is rejected for the given reason.
    Reject(ReasonCode),
    /// Assets are locked for the given reason.
    Lock(ReasonCode),
    /// Assets are clawed back for the given reason.
    Clawback(ReasonCode),
    /// Issuer must pre-approve before the transfer can proceed.
    PendingIssuerApproval,
    /// Transfer amount is revised to the given value.
    Revise(u128),
}

/// Action recorded in a compliance event log.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComplianceAction {
    Approve,
    Reject,
    Lock,
    Clawback,
    Whitelist,
    Blacklist,
}

/// Status of an issuer approval request.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApprovalStatus {
    Approved,
    Rejected,
    Revised,
    Pending,
}

/// Severity and nature of an asset lock.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LockType {
    /// Informational lock; can be lifted by admin.
    Soft,
    /// Cryptographic lock; cannot be lifted without a clawback event.
    Hard,
    /// Temporary lock while awaiting issuer approval.
    PendingApproval,
}

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

/// Compliance rule scoped to a country-code and asset-class pair.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JurisdictionRule {
    pub country_code: String,
    pub asset_class: AssetClass,
    pub transfer_policy: TransferPolicy,
    pub min_kyc_tier: u32,
    pub max_transfer_amount: Option<u128>,
    pub max_holding_amount: Option<u128>,
    pub min_holding_period: Option<u64>,
    pub requires_issuer_approval: bool,
    pub clawback_on_kyc_expiry: bool,
    pub version: u32,
    pub updated_at: u64,
}

/// KYC / AML credential issued by an anchor for a wallet.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KycCredential {
    pub wallet: Address,
    pub tier: u32,
    pub country_code: String,
    pub credential_hash: BytesN<32>,
    pub issued_at: u64,
    pub expires_at: u64,
    pub issuer_anchor: Address,
    pub is_sanctioned: bool,
    pub sanctions_lists_checked: Vec<String>,
}

/// A single compliance-check event recorded during a transfer attempt.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComplianceEvent {
    pub event_id: u64,
    pub timestamp: u64,
    pub asset_contract: Address,
    pub action: ComplianceAction,
    pub sender: Address,
    pub receiver: Address,
    pub amount: u128,
    pub reason_code: ReasonCode,
    pub jurisdiction_sender: String,
    pub jurisdiction_receiver: String,
    pub kyc_tier_sender: u32,
    pub kyc_tier_receiver: u32,
    pub rule_version: u32,
    pub tx_hash: BytesN<32>,
}

/// Response from the issuer approval flow.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApprovalResponse {
    pub status: ApprovalStatus,
    pub reason_code: ReasonCode,
    pub revised_amount: Option<u128>,
    pub audit_ref: String,
}

/// Configuration for issuer-level rule enforcement.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IssuerRuleConfig {
    pub asset_class: AssetClass,
    pub jurisdictions: Vec<String>,
    pub require_kyc: bool,
    pub require_whitelist: bool,
    pub clawback_enabled: bool,
}

/// Event emitted when assets are clawed back from a holder.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClawbackEvent {
    pub holder: Address,
    pub asset_contract: Address,
    pub amount: u128,
    pub reason: ReasonCode,
}

/// Persistent record of an asset lock applied to a wallet.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockRecord {
    pub wallet: Address,
    pub asset_contract: Address,
    pub locked_at: u64,
    pub reason: ReasonCode,
    pub duration: Option<u64>,
    pub lock_type: LockType,
}

/// Persistent record of an executed clawback.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClawbackRecord {
    pub event_id: u64,
    pub holder: Address,
    pub asset_contract: Address,
    pub amount: u128,
    pub reason: ReasonCode,
    pub destination: Address,
    pub executed_at: u64,
}

/// Report summarising compliance events over a time window.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComplianceReport {
    pub asset_contract: Address,
    pub from_timestamp: u64,
    pub to_timestamp: u64,
    pub total_events: u64,
    pub events: Vec<ComplianceEvent>,
}

// ---------------------------------------------------------------------------
// Impl blocks
// ---------------------------------------------------------------------------

impl ReasonCode {
    /// Returns a human-readable description of this reason code.
    pub fn description(&self) -> &'static str {
        match self {
            ReasonCode::None => "No reason code",
            ReasonCode::SanctionedJurisdiction => {
                "Sender or receiver is in a sanctioned jurisdiction"
            }
            ReasonCode::ProhibitedJurisdiction => "Transfer involves a prohibited jurisdiction",
            ReasonCode::AmountExceedsJurisdictionCap => "Transfer amount exceeds jurisdiction cap",
            ReasonCode::HoldingPeriodNotMet => "Minimum holding period not yet satisfied",
            ReasonCode::HoldingsCapExceeded => "Receiver would exceed maximum holdings cap",
            ReasonCode::InsufficientKycTier => "KYC tier is below the minimum required",
            ReasonCode::KycExpired => "KYC credential has expired",
            ReasonCode::KycNotFound => "No KYC credential found for wallet",
            ReasonCode::SanctionedAddress => "Wallet address is under sanctions",
            ReasonCode::NotWhitelisted => "Receiver is not on the issuer whitelist",
            ReasonCode::IssuerApprovalRequired => {
                "Issuer pre-approval is required for this transfer"
            }
            ReasonCode::AssetClassRestricted => "Asset class is restricted for this jurisdiction",
            ReasonCode::ContractPaused => "ARCM contract is currently paused",
        }
    }
}

impl ComplianceDecision {
    /// Returns `true` if the decision is [`Approve`](ComplianceDecision::Approve).
    pub fn is_approve(&self) -> bool {
        matches!(self, ComplianceDecision::Approve)
    }

    /// Returns `true` if the decision is [`Reject`](ComplianceDecision::Reject).
    pub fn is_reject(&self) -> bool {
        matches!(self, ComplianceDecision::Reject(_))
    }

    /// Returns `true` if the decision is [`Lock`](ComplianceDecision::Lock).
    pub fn is_lock(&self) -> bool {
        matches!(self, ComplianceDecision::Lock(_))
    }

    /// Returns `true` if the decision is [`Clawback`](ComplianceDecision::Clawback).
    pub fn is_clawback(&self) -> bool {
        matches!(self, ComplianceDecision::Clawback(_))
    }

    /// Returns `true` if the decision is [`PendingIssuerApproval`](ComplianceDecision::PendingIssuerApproval).
    pub fn is_pending(&self) -> bool {
        matches!(self, ComplianceDecision::PendingIssuerApproval)
    }

    /// Returns `true` if the decision is [`Revise`](ComplianceDecision::Revise).
    pub fn is_revise(&self) -> bool {
        matches!(self, ComplianceDecision::Revise(_))
    }

    /// Returns the inner [`ReasonCode`] for `Reject`, `Lock`, or `Clawback` variants.
    pub fn reason_code(&self) -> Option<&ReasonCode> {
        match self {
            ComplianceDecision::Reject(r)
            | ComplianceDecision::Lock(r)
            | ComplianceDecision::Clawback(r) => Some(r),
            _ => None,
        }
    }
}

impl JurisdictionRule {
    /// Creates a new [`JurisdictionRule`] with the given parameters.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        country_code: String,
        asset_class: AssetClass,
        transfer_policy: TransferPolicy,
        min_kyc_tier: u32,
        max_transfer_amount: Option<u128>,
        max_holding_amount: Option<u128>,
        min_holding_period: Option<u64>,
        requires_issuer_approval: bool,
        clawback_on_kyc_expiry: bool,
        version: u32,
        updated_at: u64,
    ) -> Self {
        JurisdictionRule {
            country_code,
            asset_class,
            transfer_policy,
            min_kyc_tier,
            max_transfer_amount,
            max_holding_amount,
            min_holding_period,
            requires_issuer_approval,
            clawback_on_kyc_expiry,
            version,
            updated_at,
        }
    }
}

impl KycCredential {
    /// Returns `true` if the credential has expired by `current_timestamp`.
    pub fn is_expired(&self, current_timestamp: u64) -> bool {
        self.expires_at < current_timestamp
    }

    /// Returns `true` if the credential's tier meets `required_tier` and the holder is not sanctioned.
    pub fn meets_tier_requirement(&self, required_tier: u32) -> bool {
        self.tier >= required_tier && !self.is_sanctioned
    }
}

impl ComplianceEvent {
    /// Creates a new [`ComplianceEvent`] with the given parameters.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        event_id: u64,
        timestamp: u64,
        asset_contract: Address,
        action: ComplianceAction,
        sender: Address,
        receiver: Address,
        amount: u128,
        reason_code: ReasonCode,
        jurisdiction_sender: String,
        jurisdiction_receiver: String,
        kyc_tier_sender: u32,
        kyc_tier_receiver: u32,
        rule_version: u32,
        tx_hash: BytesN<32>,
    ) -> Self {
        ComplianceEvent {
            event_id,
            timestamp,
            asset_contract,
            action,
            sender,
            receiver,
            amount,
            reason_code,
            jurisdiction_sender,
            jurisdiction_receiver,
            kyc_tier_sender,
            kyc_tier_receiver,
            rule_version,
            tx_hash,
        }
    }
}

// ---------------------------------------------------------------------------
// evaluate_transfer — pure function, no storage access
// ---------------------------------------------------------------------------

/// Pure-function compliance check. Returns a [`ComplianceDecision`] based on the
/// sender/receiver jurisdiction rules, KYC state, holding period, and caps.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_transfer(
    sender_rule: &JurisdictionRule,
    receiver_rule: &JurisdictionRule,
    amount: u128,
    _sender_kyc_tier: u32,
    receiver_kyc_tier: u32,
    receiver_kyc_expires_at: u64,
    current_timestamp: u64,
    sender_acquisition_timestamp: Option<u64>,
    receiver_current_holdings: Option<u128>,
) -> ComplianceDecision {
    if sender_rule.transfer_policy == TransferPolicy::Sanctioned
        || receiver_rule.transfer_policy == TransferPolicy::Sanctioned
    {
        return ComplianceDecision::Reject(ReasonCode::SanctionedJurisdiction);
    }

    if sender_rule.transfer_policy == TransferPolicy::Prohibited
        || receiver_rule.transfer_policy == TransferPolicy::Prohibited
    {
        return ComplianceDecision::Reject(ReasonCode::ProhibitedJurisdiction);
    }

    let required_tier = sender_rule.min_kyc_tier.max(receiver_rule.min_kyc_tier);
    if receiver_kyc_tier < required_tier {
        return ComplianceDecision::Reject(ReasonCode::InsufficientKycTier);
    }

    if receiver_kyc_expires_at < current_timestamp {
        if receiver_rule.clawback_on_kyc_expiry {
            return ComplianceDecision::Clawback(ReasonCode::KycExpired);
        }
        return ComplianceDecision::Lock(ReasonCode::KycExpired);
    }

    if let Some(max) = receiver_rule.max_transfer_amount {
        if amount > max {
            return ComplianceDecision::Reject(ReasonCode::AmountExceedsJurisdictionCap);
        }
    }

    if let Some(min_period) = sender_rule.min_holding_period {
        if let Some(acquired_at) = sender_acquisition_timestamp {
            let held_for = current_timestamp.saturating_sub(acquired_at);
            if held_for < min_period {
                return ComplianceDecision::Reject(ReasonCode::HoldingPeriodNotMet);
            }
        }
    }

    if let Some(max_holdings) = receiver_rule.max_holding_amount {
        if let Some(current_holdings) = receiver_current_holdings {
            if current_holdings + amount > max_holdings {
                return ComplianceDecision::Reject(ReasonCode::HoldingsCapExceeded);
            }
        }
    }

    if receiver_rule.requires_issuer_approval {
        return ComplianceDecision::PendingIssuerApproval;
    }

    ComplianceDecision::Approve
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{vec, Env, IntoVal, String};

    fn create_test_env() -> Env {
        Env::default()
    }

    fn make_country_string(env: &Env, code: &str) -> String {
        code.into_val(env)
    }

    fn zero_hash(env: &Env) -> BytesN<32> {
        BytesN::from_array(env, &[0u8; 32])
    }

    fn one_hash(env: &Env) -> BytesN<32> {
        let mut arr = [0u8; 32];
        arr[0] = 1;
        BytesN::from_array(env, &arr)
    }

    // --- TransferPolicy tests ---

    #[test]
    fn test_transfer_policy_variants() {
        let policies = [
            TransferPolicy::Open,
            TransferPolicy::Restricted,
            TransferPolicy::AccreditedOnly,
            TransferPolicy::InstitutionalOnly,
            TransferPolicy::Prohibited,
            TransferPolicy::Sanctioned,
        ];
        assert_eq!(policies.len(), 6);
        assert!(TransferPolicy::Open != TransferPolicy::Prohibited);
    }

    // --- AssetClass tests ---

    #[test]
    fn test_asset_class_variants() {
        let classes = [
            AssetClass::RealEstate,
            AssetClass::Commodity,
            AssetClass::Equity,
            AssetClass::Debt,
            AssetClass::Fund,
            AssetClass::Generic,
        ];
        assert_eq!(classes.len(), 6);
    }

    // --- ReasonCode tests ---

    #[test]
    fn test_reason_code_descriptions() {
        assert!(ReasonCode::SanctionedJurisdiction
            .description()
            .contains("sanctioned"));
        assert!(ReasonCode::KycExpired.description().contains("expired"));
        assert!(ReasonCode::InsufficientKycTier
            .description()
            .contains("KYC tier"));
        assert!(ReasonCode::NotWhitelisted
            .description()
            .contains("whitelist"));
        assert!(ReasonCode::ContractPaused.description().contains("paused"));
    }

    #[test]
    fn test_reason_code_all_variants_have_descriptions() {
        let codes = [
            ReasonCode::None,
            ReasonCode::SanctionedJurisdiction,
            ReasonCode::ProhibitedJurisdiction,
            ReasonCode::AmountExceedsJurisdictionCap,
            ReasonCode::HoldingPeriodNotMet,
            ReasonCode::HoldingsCapExceeded,
            ReasonCode::InsufficientKycTier,
            ReasonCode::KycExpired,
            ReasonCode::KycNotFound,
            ReasonCode::SanctionedAddress,
            ReasonCode::NotWhitelisted,
            ReasonCode::IssuerApprovalRequired,
            ReasonCode::AssetClassRestricted,
            ReasonCode::ContractPaused,
        ];
        for code in &codes {
            assert!(
                !code.description().is_empty(),
                "ReasonCode variant {:?} has no description",
                code
            );
        }
    }

    // --- ComplianceDecision tests ---

    #[test]
    fn test_compliance_decision_approve() {
        let d = ComplianceDecision::Approve;
        assert!(d.is_approve());
        assert!(!d.is_reject());
        assert!(!d.is_lock());
        assert!(!d.is_clawback());
        assert!(!d.is_pending());
        assert!(!d.is_revise());
        assert_eq!(d.reason_code(), None);
    }

    #[test]
    fn test_compliance_decision_reject() {
        let d = ComplianceDecision::Reject(ReasonCode::KycNotFound);
        assert!(d.is_reject());
        assert!(!d.is_approve());
        assert_eq!(d.reason_code(), Some(&ReasonCode::KycNotFound));
    }

    #[test]
    fn test_compliance_decision_lock() {
        let d = ComplianceDecision::Lock(ReasonCode::KycExpired);
        assert!(d.is_lock());
        assert_eq!(d.reason_code(), Some(&ReasonCode::KycExpired));
    }

    #[test]
    fn test_compliance_decision_clawback() {
        let d = ComplianceDecision::Clawback(ReasonCode::SanctionedAddress);
        assert!(d.is_clawback());
        assert_eq!(d.reason_code(), Some(&ReasonCode::SanctionedAddress));
    }

    #[test]
    fn test_compliance_decision_pending() {
        let d = ComplianceDecision::PendingIssuerApproval;
        assert!(d.is_pending());
        assert_eq!(d.reason_code(), None);
    }

    #[test]
    fn test_compliance_decision_revise() {
        let d = ComplianceDecision::Revise(5000);
        assert!(d.is_revise());
        assert_eq!(d.reason_code(), None);
        if let ComplianceDecision::Revise(amount) = d {
            assert_eq!(amount, 5000);
        } else {
            panic!("Expected Revise variant");
        }
    }

    // --- JurisdictionRule tests ---

    #[test]
    fn test_jurisdiction_rule_new() {
        let env = create_test_env();
        let rule = JurisdictionRule::new(
            make_country_string(&env, "US"),
            AssetClass::Equity,
            TransferPolicy::AccreditedOnly,
            2,
            Some(1_000_000),
            Some(10_000_000),
            Some(90 * 86400),
            false,
            true,
            1,
            1_700_000_000,
        );
        assert_eq!(rule.country_code, make_country_string(&env, "US"));
        assert_eq!(rule.asset_class, AssetClass::Equity);
        assert_eq!(rule.transfer_policy, TransferPolicy::AccreditedOnly);
        assert_eq!(rule.min_kyc_tier, 2);
        assert_eq!(rule.max_transfer_amount, Some(1_000_000));
        assert_eq!(rule.max_holding_amount, Some(10_000_000));
        assert_eq!(rule.min_holding_period, Some(90 * 86400));
        assert!(!rule.requires_issuer_approval);
        assert!(rule.clawback_on_kyc_expiry);
        assert_eq!(rule.version, 1);
        assert_eq!(rule.updated_at, 1_700_000_000);
    }

    #[test]
    fn test_jurisdiction_rule_default_values() {
        let env = create_test_env();
        let rule = JurisdictionRule::new(
            make_country_string(&env, "DE"),
            AssetClass::RealEstate,
            TransferPolicy::Open,
            1,
            None,
            None,
            None,
            false,
            false,
            0,
            0,
        );
        assert_eq!(rule.country_code, make_country_string(&env, "DE"));
        assert_eq!(rule.transfer_policy, TransferPolicy::Open);
        assert_eq!(rule.max_transfer_amount, None);
        assert_eq!(rule.min_holding_period, None);
    }

    // --- KycCredential tests ---

    #[test]
    fn test_kyc_credential_is_expired() {
        let env = create_test_env();
        let wallet = Address::generate(&env);
        let anchor = Address::generate(&env);
        let checklists = vec![&env, make_country_string(&env, "OFAC")];

        let credential = KycCredential {
            wallet: wallet.clone(),
            tier: 2,
            country_code: make_country_string(&env, "US"),
            credential_hash: zero_hash(&env),
            issued_at: 1_700_000_000,
            expires_at: 1_731_600_000,
            issuer_anchor: anchor,
            is_sanctioned: false,
            sanctions_lists_checked: checklists,
        };
        assert!(credential.is_expired(1_731_600_001));
        assert!(!credential.is_expired(1_700_000_000));
        assert!(!credential.is_expired(1_731_600_000));
    }

    #[test]
    fn test_kyc_credential_meets_tier_requirement() {
        let env = create_test_env();
        let wallet = Address::generate(&env);
        let anchor = Address::generate(&env);

        let credential = KycCredential {
            wallet: wallet.clone(),
            tier: 2,
            country_code: make_country_string(&env, "US"),
            credential_hash: zero_hash(&env),
            issued_at: 1_700_000_000,
            expires_at: 1_731_600_000,
            issuer_anchor: anchor,
            is_sanctioned: false,
            sanctions_lists_checked: vec![&env],
        };
        assert!(credential.meets_tier_requirement(1));
        assert!(credential.meets_tier_requirement(2));
        assert!(!credential.meets_tier_requirement(3));
    }

    #[test]
    fn test_kyc_credential_sanctioned_fails_tier_check() {
        let env = create_test_env();
        let wallet = Address::generate(&env);
        let anchor = Address::generate(&env);

        let credential = KycCredential {
            wallet,
            tier: 3,
            country_code: make_country_string(&env, "US"),
            credential_hash: zero_hash(&env),
            issued_at: 1_700_000_000,
            expires_at: 1_731_600_000,
            issuer_anchor: anchor,
            is_sanctioned: true,
            sanctions_lists_checked: vec![&env],
        };
        assert!(!credential.meets_tier_requirement(1));
        assert!(!credential.meets_tier_requirement(3));
    }

    // --- ApprovalResponse tests ---

    #[test]
    fn test_approval_response_approved() {
        let env = create_test_env();
        let response = ApprovalResponse {
            status: ApprovalStatus::Approved,
            reason_code: ReasonCode::None,
            revised_amount: None,
            audit_ref: make_country_string(&env, "AUDIT-001"),
        };
        assert_eq!(response.status, ApprovalStatus::Approved);
        assert_eq!(response.reason_code, ReasonCode::None);
        assert_eq!(response.audit_ref, make_country_string(&env, "AUDIT-001"));
    }

    #[test]
    fn test_approval_response_rejected() {
        let env = create_test_env();
        let response = ApprovalResponse {
            status: ApprovalStatus::Rejected,
            reason_code: ReasonCode::SanctionedJurisdiction,
            revised_amount: None,
            audit_ref: make_country_string(&env, "AUDIT-002"),
        };
        assert_eq!(response.status, ApprovalStatus::Rejected);
        assert_eq!(response.reason_code, ReasonCode::SanctionedJurisdiction);
    }

    #[test]
    fn test_approval_response_revised() {
        let env = create_test_env();
        let response = ApprovalResponse {
            status: ApprovalStatus::Revised,
            reason_code: ReasonCode::None,
            revised_amount: Some(5000),
            audit_ref: make_country_string(&env, "AUDIT-003"),
        };
        assert_eq!(response.status, ApprovalStatus::Revised);
        assert_eq!(response.revised_amount, Some(5000));
    }

    // --- ComplianceEvent tests ---

    #[test]
    fn test_compliance_event_new() {
        let env = create_test_env();
        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);
        let asset = Address::generate(&env);

        let event = ComplianceEvent::new(
            1,
            1_700_000_000,
            asset.clone(),
            ComplianceAction::Approve,
            sender.clone(),
            receiver.clone(),
            500_000,
            ReasonCode::None,
            make_country_string(&env, "US"),
            make_country_string(&env, "DE"),
            2,
            2,
            1,
            zero_hash(&env),
        );
        assert_eq!(event.event_id, 1);
        assert_eq!(event.asset_contract, asset);
        assert_eq!(event.action, ComplianceAction::Approve);
        assert_eq!(event.amount, 500_000);
        assert_eq!(event.reason_code, ReasonCode::None);
        assert_eq!(event.jurisdiction_sender, make_country_string(&env, "US"));
    }

    #[test]
    fn test_compliance_event_with_rejection() {
        let env = create_test_env();
        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);
        let asset = Address::generate(&env);

        let event = ComplianceEvent::new(
            2,
            1_700_000_001,
            asset,
            ComplianceAction::Reject,
            sender,
            receiver,
            0,
            ReasonCode::ProhibitedJurisdiction,
            make_country_string(&env, "IR"),
            make_country_string(&env, "US"),
            0,
            2,
            1,
            one_hash(&env),
        );
        assert_eq!(event.action, ComplianceAction::Reject);
        assert_eq!(event.reason_code, ReasonCode::ProhibitedJurisdiction);
        assert_eq!(event.jurisdiction_sender, make_country_string(&env, "IR"));
    }

    // --- IssuerRuleConfig tests ---

    #[test]
    fn test_issuer_rule_config() {
        let env = create_test_env();
        let jurs = vec![
            &env,
            make_country_string(&env, "US"),
            make_country_string(&env, "DE"),
        ];

        let config = IssuerRuleConfig {
            asset_class: AssetClass::RealEstate,
            jurisdictions: jurs,
            require_kyc: true,
            require_whitelist: false,
            clawback_enabled: true,
        };
        assert_eq!(config.asset_class, AssetClass::RealEstate);
        assert!(config.require_kyc);
        assert!(!config.require_whitelist);
        assert!(config.clawback_enabled);
    }

    // --- LockRecord tests ---
    #[test]
    fn test_lock_record() {
        let env = create_test_env();
        let wallet = Address::generate(&env);
        let asset = Address::generate(&env);

        let record = LockRecord {
            wallet: wallet.clone(),
            asset_contract: asset.clone(),
            locked_at: 1_700_000_000,
            reason: ReasonCode::KycExpired,
            duration: None,
            lock_type: LockType::Soft,
        };
        assert_eq!(record.wallet, wallet);
        assert_eq!(record.lock_type, LockType::Soft);
        assert_eq!(record.duration, None);
    }

    #[test]
    fn test_lock_record_hard_lock() {
        let env = create_test_env();
        let wallet = Address::generate(&env);

        let record = LockRecord {
            wallet,
            asset_contract: Address::generate(&env),
            locked_at: 1_700_000_000,
            reason: ReasonCode::SanctionedAddress,
            duration: Some(7 * 86400),
            lock_type: LockType::Hard,
        };
        assert_eq!(record.lock_type, LockType::Hard);
        assert_eq!(record.duration, Some(7 * 86400));
    }

    // --- ClawbackRecord tests ---
    #[test]
    fn test_clawback_record() {
        let env = create_test_env();
        let holder = Address::generate(&env);
        let asset = Address::generate(&env);
        let dest = Address::generate(&env);

        let record = ClawbackRecord {
            event_id: 1,
            holder: holder.clone(),
            asset_contract: asset.clone(),
            amount: 1_000_000,
            reason: ReasonCode::SanctionedAddress,
            destination: dest.clone(),
            executed_at: 1_700_000_000,
        };
        assert_eq!(record.event_id, 1);
        assert_eq!(record.holder, holder);
        assert_eq!(record.amount, 1_000_000);
        assert_eq!(record.destination, dest);
    }

    // --- ClawbackEvent tests ---
    #[test]
    fn test_clawback_event() {
        let env = create_test_env();
        let holder = Address::generate(&env);
        let asset = Address::generate(&env);

        let event = ClawbackEvent {
            holder: holder.clone(),
            asset_contract: asset.clone(),
            amount: 2_000_000,
            reason: ReasonCode::KycExpired,
        };
        assert_eq!(event.holder, holder);
        assert_eq!(event.amount, 2_000_000);
        assert_eq!(event.reason, ReasonCode::KycExpired);
    }

    // --- ComplianceAction tests ---
    #[test]
    fn test_compliance_action_variants() {
        let actions = [
            ComplianceAction::Approve,
            ComplianceAction::Reject,
            ComplianceAction::Lock,
            ComplianceAction::Clawback,
            ComplianceAction::Whitelist,
            ComplianceAction::Blacklist,
        ];
        assert_eq!(actions.len(), 6);
        assert!(ComplianceAction::Approve != ComplianceAction::Reject);
    }

    // --- ApprovalStatus tests ---
    #[test]
    fn test_approval_status_variants() {
        let statuses = [
            ApprovalStatus::Approved,
            ApprovalStatus::Rejected,
            ApprovalStatus::Revised,
            ApprovalStatus::Pending,
        ];
        assert_eq!(statuses.len(), 4);
    }

    // --- LockType tests ---
    #[test]
    fn test_lock_type_variants() {
        let types = [LockType::Soft, LockType::Hard, LockType::PendingApproval];
        assert_eq!(types.len(), 3);
    }

    // --- ComplianceReport tests ---
    #[test]
    fn test_compliance_report() {
        let env = create_test_env();
        let asset = Address::generate(&env);
        let events = vec![&env];

        let report = ComplianceReport {
            asset_contract: asset.clone(),
            from_timestamp: 1_700_000_000,
            to_timestamp: 1_731_600_000,
            total_events: 0,
            events,
        };
        assert_eq!(report.asset_contract, asset);
        assert_eq!(report.total_events, 0);
    }

    // --- Equality / round-trip tests ---
    #[test]
    fn test_compliance_decision_equality() {
        assert_eq!(
            ComplianceDecision::Reject(ReasonCode::KycExpired),
            ComplianceDecision::Reject(ReasonCode::KycExpired)
        );
        assert_ne!(
            ComplianceDecision::Reject(ReasonCode::KycExpired),
            ComplianceDecision::Lock(ReasonCode::KycExpired)
        );
        assert_eq!(ComplianceDecision::Approve, ComplianceDecision::Approve);
    }

    #[test]
    fn test_reason_code_equality() {
        assert_eq!(ReasonCode::KycExpired, ReasonCode::KycExpired);
        assert_ne!(ReasonCode::KycExpired, ReasonCode::KycNotFound);
    }
}
