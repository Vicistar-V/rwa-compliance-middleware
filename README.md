# ⚖️ Automated RWA Compliance Middleware
### *Jurisdiction-Aware Smart Contract Infrastructure for Tokenized Real-World Assets on Stellar & Soroban*

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Stellar](https://img.shields.io/badge/network-Stellar-black?logo=stellar)
![Soroban](https://img.shields.io/badge/smart--contracts-Soroban-purple)
![Rust](https://img.shields.io/badge/language-Rust-orange?logo=rust)
![Status](https://img.shields.io/badge/status-In%20Development-yellow)
![SEP](https://img.shields.io/badge/SEP--0008-Regulated%20Assets-blue)
![SEP](https://img.shields.io/badge/SEP--0012-KYC%2FAML-green)
![CI](https://github.com/Vicistar-V/rwa-compliance-middleware/actions/workflows/ci.yml/badge.svg)

---

## 📋 Table of Contents

1. [Overview](#-overview)
2. [Problem Statement](#-problem-statement)
3. [Solution Architecture](#-solution-architecture)
4. [How It Works](#-how-it-works)
5. [System Architecture Diagram](#-system-architecture-diagram)
6. [Jurisdiction Rule Engine](#-jurisdiction-rule-engine)
7. [Smart Contract Design](#-smart-contract-design)
8. [KYC/AML Integration (SEP-0012)](#-kycaml-integration-sep-0012)
9. [Asset Locking Mechanism](#-asset-locking-mechanism)
10. [Clawback Mechanism](#-clawback-mechanism)
11. [Compliance Event Lifecycle](#-compliance-event-lifecycle)
12. [Supported Asset Classes](#-supported-asset-classes)
13. [Stellar SEP Standards Used](#-stellar-sep-standards-used)
14. [Jurisdiction Coverage Matrix](#-jurisdiction-coverage-matrix)
15. [Security Model](#-security-model)
16. [Tech Stack](#-tech-stack)
17. [Repository Structure](#-repository-structure)
18. [Getting Started](#-getting-started)
19. [Contract Deployment](#-contract-deployment)
20. [API Reference](#-api-reference)
21. [Integration Guide for Asset Issuers](#-integration-guide-for-asset-issuers)
22. [Testing](#-testing)
23. [Governance & Rule Updates](#-governance--rule-updates)
24. [Legal & Regulatory Disclaimer](#-legal--regulatory-disclaimer)
25. [Roadmap](#-roadmap)
26. [Contributing](#-contributing)
27. [License](#-license)

---

## 🌐 Overview

The **Automated RWA Compliance Middleware (ARCM)** is a modular, jurisdiction-aware smart contract layer built on Stellar's Soroban platform. It enables issuers of tokenized real-world assets — including real estate, commodities, equities, and debt instruments — to enforce complex, country-specific regulatory rules **entirely on-chain**, without manual oversight or legal intermediaries.

ARCM intercepts every transfer of a regulated tokenized asset, evaluates it against a live rule set derived from the issuer's compliance configuration, verifies KYC/AML status via Stellar's SEP-0012 anchor protocol, and either **approves**, **rejects**, **flags**, **locks**, or **claws back** the asset — automatically and autonomously.

> ARCM is to tokenized asset compliance what Stripe Radar is to payment fraud: invisible, automatic, and always-on.

### Core Capabilities at a Glance

| Capability | Description |
|------------|-------------|
| 🌍 **Jurisdiction Engine** | Enforces per-country transfer rules in real-time |
| 🔐 **KYC/AML Gating** | Blocks transfers to unverified or sanctioned wallets |
| 🔒 **Asset Locking** | Freezes assets upon regulatory trigger or investigation |
| ↩️ **Clawback** | Automatically reclaims assets from non-compliant holders |
| 📋 **Whitelist/Blacklist** | Maintains on-chain registry of approved/banned addresses |
| 🧾 **Audit Trail** | Immutable on-chain log of every compliance decision |
| 🔌 **Plug-in Architecture** | Issuers compose only the rules they need |
| ⚡ **Zero Manual Intervention** | Fully autonomous enforcement via Soroban contracts |

---

## 🔴 Problem Statement

### The Regulatory Chaos of Tokenized RWAs

Tokenizing real-world assets creates enormous opportunity — but also enormous regulatory complexity. A single tokenized apartment building may be:

- **Legal to hold** in Germany (under MiCA framework)
- **Restricted** in the United States (unaccredited investors)
- **Fully banned** in China (cross-border capital controls)
- **Partially allowed** in Nigeria (SEC Nigeria sandbox rules)
- **Subject to clawback** if the holder becomes a sanctioned entity under OFAC

Today, compliance for tokenized RWAs is handled through:

| Current Approach | Problem |
|-----------------|---------|
| Off-chain legal agreements | Not enforceable on-chain; depends on human action |
| Manual KYC by issuers | Slow, expensive, error-prone, not scalable |
| Centralized blocklists | Single point of failure; opaque; can be circumvented |
| Per-asset custom contracts | No standardization; impossible to audit across issuers |
| No automated clawback | Regulators cannot enforce sanctions without court orders |

**The result:** Issuers either over-restrict (blocking legitimate users) or under-restrict (violating securities law). Neither is sustainable for institutional adoption of RWA tokenization.

### What's Missing

```
CURRENT STATE:
  Token Transfer → [No Compliance Check] → Transfer Executes
                          ⬆
                   Regulatory liability
                   sits with issuer

DESIRED STATE:
  Token Transfer → [ARCM Compliance Layer] → Approve / Reject / Lock / Claw
                          ⬆
                   Regulatory liability
                   enforced autonomously on-chain
```

---

## 🧩 Solution Architecture

ARCM is composed of five loosely coupled on-chain modules, each addressing a distinct compliance concern:

```
┌──────────────────────────────────────────────────────────────┐
│                  ARCM MODULE STACK                           │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  MODULE 1: TRANSFER INTERCEPT GATEWAY                │   │
│  │  Hooks into every SEP-0008 regulated asset transfer  │   │
│  └────────────────────┬─────────────────────────────────┘   │
│                       │                                      │
│  ┌────────────────────▼─────────────────────────────────┐   │
│  │  MODULE 2: JURISDICTION RULE ENGINE                  │   │
│  │  Evaluates sender/receiver against country rules     │   │
│  └────────────────────┬─────────────────────────────────┘   │
│                       │                                      │
│  ┌────────────────────▼─────────────────────────────────┐   │
│  │  MODULE 3: KYC/AML VERIFICATION ORACLE               │   │
│  │  Queries SEP-0012 anchor + on-chain credential store │   │
│  └────────────────────┬─────────────────────────────────┘   │
│                       │                                      │
│  ┌────────────────────▼─────────────────────────────────┐   │
│  │  MODULE 4: ENFORCEMENT ENGINE                        │   │
│  │  Executes: Approve / Reject / Lock / Clawback        │   │
│  └────────────────────┬─────────────────────────────────┘   │
│                       │                                      │
│  ┌────────────────────▼─────────────────────────────────┐   │
│  │  MODULE 5: AUDIT & COMPLIANCE LEDGER                 │   │
│  │  Immutable on-chain record of every decision         │   │
│  └──────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

### Design Principles

- **Modular composition:** Issuers activate only the modules relevant to their asset class and jurisdictions.
- **Non-custodial:** ARCM never holds assets. It only gates, locks, or initiates clawback through asset contract calls.
- **Deterministic:** Given the same inputs, ARCM always produces the same compliance decision. No ambiguity.
- **Upgradeable rules:** Jurisdiction rules are stored in upgradeable on-chain storage, updated via governance with timelock.
- **Fail-safe:** In the event of contract failure, the default posture is **block** (not allow) — protecting issuers from accidental non-compliance.

---

## ⚙️ How It Works

### The Compliance Decision Flow

```
USER initiates transfer of tokenized RWA
            │
            ▼
┌─────────────────────────────┐
│  TRANSFER INTERCEPT         │
│  SEP-0008 approval request  │
│  arrives at ARCM gateway    │
└────────────┬────────────────┘
             │
             ▼
┌─────────────────────────────┐       ┌─────────────────────┐
│  JURISDICTION LOOKUP        │──────▶│  Jurisdiction DB    │
│  Resolve sender +           │       │  (on-chain storage) │
│  receiver country codes     │◀──────│  ISO 3166-1 alpha-2 │
└────────────┬────────────────┘       └─────────────────────┘
             │
             ▼
┌─────────────────────────────┐
│  RULE EVALUATION            │
│  Check:                     │
│  • Transfer allowed?        │
│  • Max transfer amount?     │
│  • Holding period met?      │
│  • Investor accreditation?  │
│  • Sanctions check?         │
└────────────┬────────────────┘
             │
             ├── RULE FAILS ──────────────────────────────────▶ REJECT
             │
             ▼
┌─────────────────────────────┐       ┌─────────────────────┐
│  KYC/AML VERIFICATION       │──────▶│  SEP-0012 Anchor    │
│  Query credential status    │       │  Off-chain KYC      │
│  for sender + receiver      │◀──────│  Provider           │
└────────────┬────────────────┘       └─────────────────────┘
             │
             ├── KYC MISSING ─────────────────────────────────▶ REJECT (pending KYC)
             ├── KYC EXPIRED ─────────────────────────────────▶ LOCK + NOTIFY
             ├── SANCTIONED ──────────────────────────────────▶ LOCK + CLAWBACK
             │
             ▼
┌─────────────────────────────┐
│  WHITELIST CHECK            │
│  Is receiver in             │
│  issuer's approved list?    │
└────────────┬────────────────┘
             │
             ├── BLACKLISTED ─────────────────────────────────▶ REJECT + FLAG
             │
             ▼
┌─────────────────────────────┐
│  ENFORCEMENT DECISION       │
│  All checks passed          │
└────────────┬────────────────┘
             │
             ▼
          APPROVE
   (SEP-0008 approval memo
    returned to asset contract)
```

---

## 🏗️ System Architecture Diagram

```
                    ┌─────────────────────────────────────────────────────────┐
                    │                   STELLAR NETWORK                       │
                    │                                                         │
  ┌──────────┐      │  ┌──────────────────────────────────────────────────┐  │
  │  Sender  │─────▶│  │         TOKENIZED RWA CONTRACT                   │  │
  │  Wallet  │      │  │  (Real Estate / Commodity / Equity Token)        │  │
  └──────────┘      │  │                                                  │  │
                    │  │  transfer(from, to, amount)                      │  │
  ┌──────────┐      │  │   └─▶ Calls ARCM Gateway for approval           │  │
  │ Receiver │      │  └─────────────────┬────────────────────────────────┘  │
  │  Wallet  │      │                    │ SEP-0008                           │
  └──────────┘      │                    │ approval_request()                 │
                    │                    ▼                                    │
                    │  ┌──────────────────────────────────────────────────┐  │
                    │  │      ARCM TRANSFER GATEWAY CONTRACT              │  │
                    │  │      (gateway/src/lib.rs)                        │  │
                    │  │                                                  │  │
                    │  │  • Receives SEP-0008 approval requests           │  │
                    │  │  • Routes to Rule Engine                         │  │
                    │  │  • Returns: Approve / Reject / Revise            │  │
                    │  └──────────────────┬───────────────────────────────┘  │
                    │                     │                                   │
                    │          ┌──────────┴──────────┐                       │
                    │          │                     │                       │
                    │          ▼                     ▼                       │
                    │  ┌───────────────┐   ┌─────────────────────────────┐  │
                    │  │  JURISDICTION │   │    KYC/AML ORACLE           │  │
                    │  │  RULE ENGINE  │   │    CONTRACT                 │  │
                    │  │               │   │                             │  │
                    │  │ • Country DB  │   │  • Queries SEP-0012 Anchor  │  │
                    │  │ • Rule eval   │   │  • Checks credential expiry │  │
                    │  │ • Amount caps │   │  • OFAC/sanctions lookup    │  │
                    │  │ • Hold period │   │  • Stores KYC status hash   │  │
                    │  └───────┬───────┘   └──────────────┬──────────────┘  │
                    │          │                           │                 │
                    │          └──────────┬────────────────┘                 │
                    │                     │                                  │
                    │                     ▼                                  │
                    │  ┌──────────────────────────────────────────────────┐  │
                    │  │         ENFORCEMENT ENGINE CONTRACT              │  │
                    │  │         (enforcement/src/lib.rs)                 │  │
                    │  │                                                  │  │
                    │  │  APPROVE ──▶ Sign SEP-0008 approval              │  │
                    │  │  REJECT  ──▶ Return denial + reason code         │  │
                    │  │  LOCK    ──▶ Call asset.set_authorized(false)    │  │
                    │  │  CLAWBACK──▶ Call asset.clawback(holder, amt)    │  │
                    │  └──────────────────┬───────────────────────────────┘  │
                    │                     │                                  │
                    │                     ▼                                  │
                    │  ┌──────────────────────────────────────────────────┐  │
                    │  │         COMPLIANCE AUDIT LEDGER                  │  │
                    │  │         (audit/src/lib.rs)                       │  │
                    │  │                                                  │  │
                    │  │  • Immutable decision log (epoch-stamped)        │  │
                    │  │  • Reason codes per decision                     │  │
                    │  │  • Exportable for regulator reporting            │  │
                    │  └──────────────────────────────────────────────────┘  │
                    │                                                         │
                    │  ┌──────────────────────────────────────────────────┐  │
                    │  │          EXTERNAL INTEGRATIONS                   │  │
                    │  │                                                  │  │
                    │  │  SEP-0012 KYC Anchor  ◀──▶  Synaps / Persona    │  │
                    │  │  Sanctions Oracle     ◀──▶  Chainalysis / TRM   │  │
                    │  │  Issuer Dashboard     ◀──▶  ARCM Admin UI        │  │
                    │  │  Regulatory Export    ◀──▶  Regulator Portals    │  │
                    │  └──────────────────────────────────────────────────┘  │
                    └─────────────────────────────────────────────────────────┘
```

---

## 🌍 Jurisdiction Rule Engine

The heart of ARCM is the **Jurisdiction Rule Engine** — a deterministic, on-chain policy evaluator that applies country-specific compliance rules to every transfer attempt.

### Rule Schema

Each jurisdiction rule is stored as a structured record:

```rust
pub struct JurisdictionRule {
    /// ISO 3166-1 alpha-2 country code (e.g., "US", "DE", "NG")
    pub country_code: String,

    /// Asset class this rule applies to
    pub asset_class: AssetClass,  // RealEstate | Commodity | Equity | Debt

    /// Transfer permission level
    pub transfer_policy: TransferPolicy,

    /// Minimum KYC tier required (1 = basic ID, 2 = accredited, 3 = institutional)
    pub min_kyc_tier: u8,

    /// Maximum single transfer in USD equivalent (None = unlimited)
    pub max_transfer_amount: Option<u128>,

    /// Maximum total holdings per wallet in USD equivalent
    pub max_holding_amount: Option<u128>,

    /// Minimum holding period before resale (in seconds)
    pub min_holding_period: Option<u64>,

    /// Whether the country requires issuer pre-approval for each transfer
    pub requires_issuer_approval: bool,

    /// Automatic clawback on KYC expiry
    pub clawback_on_kyc_expiry: bool,

    /// Rule version — incremented on each governance update
    pub version: u32,

    /// Block timestamp of last update
    pub updated_at: u64,
}

pub enum TransferPolicy {
    Open,           // Unrestricted (all KYC tiers accepted)
    Restricted,     // Transfer allowed but with conditions (amount caps, hold period)
    AccreditedOnly, // Only KYC tier 2+ (accredited investors)
    InstitutionalOnly, // Only KYC tier 3 (institutional)
    Prohibited,     // No transfers in/out of this jurisdiction
    Sanctioned,     // Country under active sanctions — auto-reject + report
}

pub enum AssetClass {
    RealEstate,
    Commodity,
    Equity,
    Debt,
    Fund,
    Generic,
}
```

### Rule Evaluation Algorithm

```rust
pub fn evaluate_transfer(
    env: &Env,
    asset_contract: Address,
    asset_class: AssetClass,
    sender: Address,
    receiver: Address,
    amount: u128,
) -> ComplianceDecision {

    // 1. Resolve jurisdictions
    let sender_country = resolve_country(env, &sender);
    let receiver_country = resolve_country(env, &receiver);

    // 2. Load rules for both sides
    let sender_rule = load_rule(env, &sender_country, &asset_class);
    let receiver_rule = load_rule(env, &receiver_country, &asset_class);

    // 3. Check sanctioned countries (hard block — no override possible)
    if sender_rule.transfer_policy == TransferPolicy::Sanctioned
        || receiver_rule.transfer_policy == TransferPolicy::Sanctioned
    {
        emit_compliance_event(env, ComplianceAction::Reject, ReasonCode::SanctionedJurisdiction);
        return ComplianceDecision::Reject(ReasonCode::SanctionedJurisdiction);
    }

    // 4. Check prohibited jurisdictions
    if sender_rule.transfer_policy == TransferPolicy::Prohibited
        || receiver_rule.transfer_policy == TransferPolicy::Prohibited
    {
        return ComplianceDecision::Reject(ReasonCode::ProhibitedJurisdiction);
    }

    // 5. KYC tier check
    let receiver_kyc = get_kyc_status(env, &receiver);
    let required_tier = receiver_rule.min_kyc_tier.max(sender_rule.min_kyc_tier);
    if receiver_kyc.tier < required_tier {
        return ComplianceDecision::Reject(ReasonCode::InsufficientKycTier);
    }

    // 6. KYC expiry check
    if receiver_kyc.expires_at < env.ledger().timestamp() {
        if receiver_rule.clawback_on_kyc_expiry {
            return ComplianceDecision::Clawback(ReasonCode::KycExpired);
        }
        return ComplianceDecision::Lock(ReasonCode::KycExpired);
    }

    // 7. Amount cap check
    if let Some(max) = receiver_rule.max_transfer_amount {
        if amount > max {
            return ComplianceDecision::Reject(ReasonCode::AmountExceedsJurisdictionCap);
        }
    }

    // 8. Holding period check
    if let Some(min_period) = sender_rule.min_holding_period {
        let acquired_at = get_acquisition_timestamp(env, &sender, &asset_contract);
        let held_for = env.ledger().timestamp() - acquired_at;
        if held_for < min_period {
            return ComplianceDecision::Reject(ReasonCode::HoldingPeriodNotMet);
        }
    }

    // 9. Max holdings check
    if let Some(max_holdings) = receiver_rule.max_holding_amount {
        let current_holdings = get_usd_value_of_holdings(env, &receiver, &asset_contract);
        if current_holdings + amount > max_holdings {
            return ComplianceDecision::Reject(ReasonCode::HoldingsCapExceeded);
        }
    }

    // 10. Issuer pre-approval check
    if receiver_rule.requires_issuer_approval {
        return ComplianceDecision::PendingIssuerApproval;
    }

    ComplianceDecision::Approve
}
```

### Compliance Decision Types

```rust
pub enum ComplianceDecision {
    Approve,                          // Transfer proceeds immediately
    Reject(ReasonCode),               // Transfer blocked; reason recorded
    Lock(ReasonCode),                 // Asset frozen; transfer blocked pending resolution
    Clawback(ReasonCode),             // Asset automatically returned to issuer
    PendingIssuerApproval,            // Transfer paused awaiting issuer sign-off
    Revise { max_allowed: u128 },     // Transfer allowed at reduced amount (SEP-0008 revise)
}

pub enum ReasonCode {
    // Jurisdiction
    SanctionedJurisdiction,
    ProhibitedJurisdiction,
    AmountExceedsJurisdictionCap,
    HoldingPeriodNotMet,
    HoldingsCapExceeded,
    // KYC/AML
    InsufficientKycTier,
    KycExpired,
    KycNotFound,
    SanctionedAddress,
    // Issuer
    NotWhitelisted,
    IssuerApprovalRequired,
    // System
    AssetClassRestricted,
    ContractPaused,
}
```

---

## 📜 Smart Contract Design

### Contract Inventory

| Contract | Location | Responsibility |
|----------|----------|----------------|
| `arcm_gateway` | `contracts/gateway/` | SEP-0008 request handler; main entry point |
| `jurisdiction_engine` | `contracts/jurisdiction/` | Rule storage + evaluation |
| `kyc_oracle` | `contracts/kyc_oracle/` | KYC/AML credential verification |
| `enforcement_engine` | `contracts/enforcement/` | Lock, clawback, whitelist execution |
| `compliance_ledger` | `contracts/audit/` | Immutable decision log |
| `rule_governance` | `contracts/governance/` | Timelocked rule update management |
| `credential_registry` | `contracts/credentials/` | On-chain KYC hash store |
| `country_resolver` | `contracts/geo/` | Wallet → Country code mapping |

---

### 1. ARCM Gateway Contract (`contracts/gateway`)

The SEP-0008 regulated asset approval gateway. Every transfer of a registered RWA must pass through this contract.

```rust
/// Entry point for all SEP-0008 approval requests
pub fn approve(
    env: Env,
    source_account: Address,
    stellar_tx: StellarTransaction,   // The proposed transfer transaction
) -> ApprovalResponse;

/// Register a new RWA token with ARCM
pub fn register_asset(
    env: Env,
    issuer: Address,
    asset_contract: Address,
    asset_class: AssetClass,
    rule_config: IssuerRuleConfig,
);

/// Deregister an asset (issuer only)
pub fn deregister_asset(env: Env, issuer: Address, asset_contract: Address);

/// Returns compliance configuration for an asset
pub fn get_asset_config(env: Env, asset_contract: Address) -> IssuerRuleConfig;

pub struct ApprovalResponse {
    pub status: ApprovalStatus,           // Approved | Rejected | Revised | Pending
    pub reason_code: Option<ReasonCode>,
    pub revised_amount: Option<u128>,     // For SEP-0008 "revise" responses
    pub audit_ref: String,                // On-chain audit log reference ID
}
```

---

### 2. Jurisdiction Engine Contract (`contracts/jurisdiction`)

```rust
/// Load a jurisdiction rule for a country + asset class
pub fn get_rule(
    env: Env,
    country_code: String,
    asset_class: AssetClass,
) -> JurisdictionRule;

/// Propose a rule update (governance controlled)
pub fn propose_rule_update(
    env: Env,
    proposer: Address,
    country_code: String,
    asset_class: AssetClass,
    new_rule: JurisdictionRule,
) -> u64; // Returns proposal ID

/// Execute approved rule update (after timelock expires)
pub fn execute_rule_update(env: Env, proposal_id: u64);

/// List all rules for a given country
pub fn list_country_rules(env: Env, country_code: String) -> Vec<JurisdictionRule>;

/// Check if a country is sanctioned
pub fn is_sanctioned(env: Env, country_code: String) -> bool;
```

---

### 3. KYC/AML Oracle Contract (`contracts/kyc_oracle`)

Bridges off-chain KYC providers (SEP-0012 anchors) with on-chain credential verification.

```rust
pub struct KycCredential {
    pub wallet: Address,
    pub tier: u8,                    // 1 = Basic, 2 = Accredited, 3 = Institutional
    pub country_code: String,
    pub credential_hash: BytesN<32>, // SHA-256 of off-chain KYC record
    pub issued_at: u64,
    pub expires_at: u64,
    pub issuer_anchor: Address,      // SEP-0012 anchor that issued the credential
    pub is_sanctioned: bool,
    pub sanctions_lists_checked: Vec<String>, // ["OFAC", "EU", "UN"]
}

/// Submit a KYC credential (called by authorized SEP-0012 anchors only)
pub fn submit_credential(env: Env, anchor: Address, credential: KycCredential);

/// Get KYC status for a wallet
pub fn get_kyc_status(env: Env, wallet: Address) -> Option<KycCredential>;

/// Revoke a KYC credential (on sanctions hit)
pub fn revoke_credential(env: Env, anchor: Address, wallet: Address, reason: String);

/// Update sanctions status for a wallet
pub fn flag_sanctioned(env: Env, oracle_authority: Address, wallet: Address);

/// Returns whether a wallet's KYC is valid and unexpired
pub fn is_kyc_valid(env: Env, wallet: Address, required_tier: u8) -> bool;

/// Returns all wallets with expiring KYC in next N days (for keeper alerts)
pub fn expiring_credentials(env: Env, within_seconds: u64) -> Vec<Address>;
```

#### KYC Tier Definitions

| Tier | Name | Requirements | Typical User |
|------|------|-------------|--------------|
| 0 | Unverified | No KYC completed | Anonymous wallet |
| 1 | Basic | Government ID + liveness check | Retail investor |
| 2 | Accredited | Tier 1 + income/wealth verification | Accredited investor (SEC Rule 501) |
| 3 | Institutional | Tier 2 + entity docs + AML program | Fund, bank, family office |

---

### 4. Enforcement Engine Contract (`contracts/enforcement`)

Executes compliance actions against asset contracts.

```rust
/// Lock an asset for a specific wallet (freezes transfers)
pub fn lock_asset(
    env: Env,
    authority: Address,     // ARCM gateway or admin
    asset_contract: Address,
    wallet: Address,
    reason: ReasonCode,
    duration: Option<u64>,  // None = indefinite
);

/// Unlock a previously locked asset
pub fn unlock_asset(
    env: Env,
    authority: Address,
    asset_contract: Address,
    wallet: Address,
);

/// Initiate clawback of asset from non-compliant holder
pub fn execute_clawback(
    env: Env,
    authority: Address,
    asset_contract: Address,
    holder: Address,
    amount: Option<u128>,   // None = full balance
    reason: ReasonCode,
    destination: Address,   // Where clawed-back assets go (usually issuer)
);

/// Add wallet to asset whitelist
pub fn whitelist_address(
    env: Env,
    issuer: Address,
    asset_contract: Address,
    wallet: Address,
    tier_override: Option<u8>,
);

/// Remove wallet from whitelist or add to blacklist
pub fn blacklist_address(
    env: Env,
    issuer: Address,
    asset_contract: Address,
    wallet: Address,
    reason: String,
);

/// Return all locked wallets for an asset
pub fn get_locked_wallets(env: Env, asset_contract: Address) -> Vec<LockRecord>;

/// Return clawback history for an asset
pub fn get_clawback_history(env: Env, asset_contract: Address) -> Vec<ClawbackRecord>;
```

---

### 5. Compliance Audit Ledger (`contracts/audit`)

Append-only on-chain log of every compliance decision made by ARCM.

```rust
pub struct ComplianceEvent {
    pub event_id: u64,               // Sequential, never reused
    pub timestamp: u64,
    pub asset_contract: Address,
    pub action: ComplianceAction,    // Approve | Reject | Lock | Clawback | Whitelist | Blacklist
    pub sender: Address,
    pub receiver: Address,
    pub amount: u128,
    pub reason_code: Option<ReasonCode>,
    pub jurisdiction_sender: String, // ISO 3166-1 alpha-2
    pub jurisdiction_receiver: String,
    pub kyc_tier_sender: u8,
    pub kyc_tier_receiver: u8,
    pub rule_version: u32,           // Which version of rules was applied
    pub tx_hash: BytesN<32>,         // Stellar transaction hash
}

/// Append a compliance event (gateway only)
pub fn log_event(env: Env, caller: Address, event: ComplianceEvent) -> u64;

/// Query events for an asset (paginated)
pub fn query_events(
    env: Env,
    asset_contract: Address,
    from_id: u64,
    limit: u32,
) -> Vec<ComplianceEvent>;

/// Query events for a wallet (paginated)
pub fn query_wallet_events(
    env: Env,
    wallet: Address,
    from_id: u64,
    limit: u32,
) -> Vec<ComplianceEvent>;

/// Export full event log as structured data (for regulator reporting)
pub fn export_report(
    env: Env,
    asset_contract: Address,
    from_timestamp: u64,
    to_timestamp: u64,
) -> ComplianceReport;
```

---

## 🪪 KYC/AML Integration (SEP-0012)

### SEP-0012 Flow

ARCM integrates with Stellar's **SEP-0012 (KYC API)** standard, which defines how anchors collect and store user identity data.

```
STEP 1: User submits KYC documents to SEP-0012 Anchor
  User ──▶ Anchor KYC Portal ──▶ ID verification, liveness, AML screening

STEP 2: Anchor signs KYC credential
  Anchor ──▶ ARCM KYC Oracle ──▶ submit_credential(wallet, tier, hash, expiry)
  Note: Only the credential HASH is stored on-chain. PII stays off-chain.

STEP 3: ARCM caches credential status
  KYC Oracle stores: { wallet, tier, country, expiry, sanctioned_flag }
  
STEP 4: Transfer attempt triggers KYC check
  Gateway ──▶ KYC Oracle.is_kyc_valid(receiver, required_tier)
  
STEP 5: Credential renewal (before expiry)
  Anchor re-runs KYC ──▶ Calls submit_credential() with new expiry
  
STEP 6: Sanctions hit
  Chainalysis / TRM Oracle ──▶ Calls flag_sanctioned(wallet)
  ──▶ Enforcement Engine locks + initiates clawback
```

### Supported KYC Providers

| Provider | Type | Integration Method |
|----------|------|-------------------|
| Synaps | SEP-0012 Anchor | Direct Stellar anchor |
| Persona | KYC SaaS | Via anchor adapter |
| Onfido | KYC SaaS | Via anchor adapter |
| Jumio | KYC SaaS | Via anchor adapter |
| Chainalysis KYT | Blockchain analytics | Sanctions oracle |
| TRM Labs | Blockchain analytics | Sanctions oracle |
| Elliptic | Blockchain analytics | Sanctions oracle |

### On-Chain Privacy Model

```
WHAT IS STORED ON-CHAIN (public):
  ✅ Wallet address
  ✅ KYC tier (1/2/3)
  ✅ Country code
  ✅ Credential expiry timestamp
  ✅ Sanctions flag (bool)
  ✅ SHA-256 hash of off-chain KYC record (for integrity verification)

WHAT IS NEVER STORED ON-CHAIN:
  ❌ Full name
  ❌ Date of birth
  ❌ Passport / ID number
  ❌ Address
  ❌ Tax ID / SSN
  ❌ Photo / biometric data
  ❌ Bank account details
```

---

## 🔒 Asset Locking Mechanism

ARCM uses Stellar's native **authorization flags** on Classic assets and Soroban's **token authorization** on Soroban tokens.

### Lock Triggers

| Trigger | Automatic? | Lock Type | Resolution Path |
|---------|-----------|-----------|----------------|
| KYC expired | ✅ Yes | Soft lock (no transfers until renewal) | Re-submit KYC |
| Sanctions hit | ✅ Yes | Hard lock + clawback initiated | Issuer + regulator review |
| Jurisdiction violation | ✅ Yes | Transfer block (not full lock) | KYC upgrade or jurisdiction approval |
| Issuer request | Manual | Full lock | Issuer unlocks via admin |
| Court/regulatory order | Manual | Full lock + evidence log | Regulatory resolution |
| Suspicious activity | ✅ Yes (AML oracle) | Soft lock + investigation flag | AML review completion |

### Lock State Machine

```
                    ┌─────────┐
                    │ ACTIVE  │  ◀── Normal transferable state
                    └────┬────┘
                         │
             ┌───────────┼────────────┐
             │           │            │
             ▼           ▼            ▼
       ┌──────────┐ ┌─────────┐ ┌──────────┐
       │ SOFT     │ │  HARD   │ │ PENDING  │
       │ LOCKED   │ │ LOCKED  │ │ APPROVAL │
       │          │ │         │ │          │
       │ No sends │ │ No sends│ │ Transfer │
       │ Can recv │ │ No recv │ │ queued   │
       └────┬─────┘ └────┬────┘ └────┬─────┘
            │             │           │
            │ KYC renewed │ Regulatory│ Issuer
            │             │ clearance │ approves
            ▼             ▼           ▼
          ┌─────────────────────────────┐
          │           ACTIVE            │
          └─────────────────────────────┘
                         │
                    Clawback trigger
                         │
                         ▼
                  ┌────────────┐
                  │  CLAWBACK  │
                  │  EXECUTED  │  ── Terminal state (assets returned to issuer)
                  └────────────┘
```

---

## ↩️ Clawback Mechanism

Clawback is the most powerful — and most sensitive — capability in ARCM. It uses Stellar's native **CLAWBACK_ENABLED flag** on asset accounts to reclaim tokens from a non-compliant holder.

### Clawback Triggers

```
AUTOMATIC TRIGGERS (no human intervention required):
  1. Wallet flagged as sanctioned by OFAC/EU/UN oracle
  2. KYC expired AND clawback_on_kyc_expiry = true in jurisdiction rule
  3. Wallet detected in prohibited jurisdiction with active holdings
  4. Fraudulent credential detected (anchor revokes KYC)

MANUAL TRIGGERS (issuer or regulator authorizes):
  5. Court order / regulatory instruction
  6. AML investigation outcome
  7. Terms of service violation (issuer-initiated)
```

### Clawback Execution Flow

```rust
// Triggered automatically by enforcement engine
pub fn execute_clawback(
    env: Env,
    authority: Address,
    asset_contract: Address,
    holder: Address,
    amount: Option<u128>,
    reason: ReasonCode,
    destination: Address,
) {
    // 1. Verify authority (must be ARCM gateway or admin multisig)
    authority.require_auth();
    verify_arcm_authority(&env, &authority);

    // 2. Lock first (prevents front-running withdrawal)
    lock_asset(&env, &asset_contract, &holder);

    // 3. Determine clawback amount
    let clawback_amount = amount.unwrap_or_else(|| {
        get_full_balance(&env, &asset_contract, &holder)
    });

    // 4. Execute Stellar clawback via asset contract
    invoke_asset_clawback(&env, &asset_contract, &holder, clawback_amount, &destination);

    // 5. Log to audit ledger
    log_clawback_event(&env, &holder, &asset_contract, clawback_amount, &reason);

    // 6. Notify (emit event for off-chain watchers)
    env.events().publish(
        (Symbol::new(&env, "clawback_executed"),),
        ClawbackEvent { holder, asset_contract, amount: clawback_amount, reason }
    );
}
```

### Clawback Destination Routing

```
Clawback destination depends on the trigger type:

  SANCTIONS HIT    ──▶  Frozen Escrow Contract (awaiting regulatory guidance)
  KYC EXPIRED      ──▶  Issuer Reserve Wallet (re-distributable after KYC renewal)
  COURT ORDER      ──▶  Designated regulator/court wallet
  ISSUER REQUEST   ──▶  Issuer Reserve Wallet
```

---

## 📅 Compliance Event Lifecycle

```
DAY 0:   User completes KYC via SEP-0012 anchor
          └─▶ KYC Credential (Tier 2, expires 365 days) stored on-chain

DAY 1:   User purchases tokenized real estate token
          └─▶ ARCM Gateway approves (all checks pass)
          └─▶ Acquisition timestamp recorded

DAY 45:  User attempts to sell (holding period = 90 days)
          └─▶ ARCM Gateway rejects: HoldingPeriodNotMet
          └─▶ Audit log entry created

DAY 90:  User attempts to sell to a US wallet (accredited only zone)
          └─▶ ARCM checks receiver KYC tier → Tier 1 (not accredited)
          └─▶ Reject: InsufficientKycTier

DAY 120: Receiver upgrades KYC to Tier 2
          └─▶ Anchor submits new credential
          └─▶ Transfer retried → Approved ✅

DAY 340: OFAC adds holder's country to sanctions list
          └─▶ Sanctions Oracle flags wallet
          └─▶ Enforcement Engine: LOCK + CLAWBACK initiated
          └─▶ Assets returned to issuer frozen escrow

DAY 360: KYC renewal reminder (5 days before expiry)
          └─▶ Keeper bot emits notification event
          └─▶ Front-end notifies user to renew

DAY 366: KYC expired for another holder
          └─▶ ARCM auto-LOCKS (soft lock)
          └─▶ Transfers blocked until KYC renewed
```

---

## 🏢 Supported Asset Classes

| Asset Class | Examples | Key Compliance Rules | Typical Hold Period |
|-------------|----------|---------------------|-------------------|
| **Real Estate** | Tokenized property shares, REITs | Accredited investor thresholds, country ownership limits | 90–180 days |
| **Commodities** | Tokenized gold, oil, agricultural | CFTC reporting thresholds, position limits | No minimum |
| **Private Equity** | Startup equity tokens | Reg D/CF/A+ compliance, investor cap tables | 12 months |
| **Debt Instruments** | Corporate bonds, sovereign debt | Investor qualification, max holdings | 30–90 days |
| **Funds** | Tokenized hedge/PE fund units | AML risk rating, accredited investor only | Varies |
| **Carbon Credits** | Tokenized verified carbon offsets | Registry verification, country trading rules | No minimum |

---

## ⭐ Stellar SEP Standards Used

| SEP | Name | Role in ARCM |
|-----|------|-------------|
| **SEP-0001** | Stellar.toml | Protocol metadata, ARCM service discovery |
| **SEP-0008** | Regulated Assets | Core transfer approval/rejection mechanism |
| **SEP-0010** | Stellar Web Auth | Admin and issuer authentication |
| **SEP-0012** | KYC API | Off-chain KYC collection + on-chain credential anchoring |
| **SEP-0024** | Hosted Deposit/Withdrawal | Fiat on/off-ramp integration for compliant users |
| **SEP-0038** | Anchor RFQ | Rate quoting for compliant asset swaps |

### SEP-0008 Deep Dive

SEP-0008 is the backbone of ARCM's transfer interception. Here's how it works:

```
1. ASSET ISSUER configures their token with ARCM gateway as the
   "regulated asset approval server" in stellar.toml:

   [REGULATED_ASSETS.TOKEN_XYZ]
   approval_server = "https://arcm.yourprotocol.com/approve"
   approval_criteria = "KYC + jurisdiction check required"

2. WALLET submits transfer transaction to ARCM approval endpoint
   before broadcasting to Stellar network

3. ARCM evaluates → returns one of:
   { "status": "success" }              → Wallet broadcasts tx
   { "status": "revised", "tx": "..." } → Wallet broadcasts revised tx
   { "status": "pending", "message": "KYC required" }
   { "status": "rejected", "error": "Prohibited jurisdiction" }

4. Only approved/revised transactions accepted by asset contract
```

---

## 🗺️ Jurisdiction Coverage Matrix

### Current Rule Coverage (v1.0)

| Region | Country | Real Estate | Commodities | Equity | Debt | Status |
|--------|---------|------------|------------|--------|------|--------|
| **North America** | 🇺🇸 US | Accredited Only | Open | Reg D/CF | Accredited | ✅ Live |
| | 🇨🇦 Canada | Restricted | Open | Restricted | Open | ✅ Live |
| | 🇲🇽 Mexico | Restricted | Restricted | Restricted | Restricted | ✅ Live |
| **Europe** | 🇩🇪 Germany | Open (MiCA) | Open | Open | Open | ✅ Live |
| | 🇫🇷 France | Open (MiCA) | Open | Open | Open | ✅ Live |
| | 🇬🇧 UK | Restricted | Open | Restricted | Open | ✅ Live |
| | 🇨🇭 Switzerland | Open | Open | Open | Open | ✅ Live |
| **Africa** | 🇳🇬 Nigeria | Restricted | Open | Restricted | Open | ✅ Live |
| | 🇿🇦 South Africa | Restricted | Open | Restricted | Restricted | ✅ Live |
| | 🇰🇪 Kenya | Restricted | Open | Prohibited | Restricted | ✅ Live |
| | 🇬🇭 Ghana | Restricted | Open | Restricted | Restricted | ✅ Live |
| **Asia-Pacific** | 🇸🇬 Singapore | Open | Open | Open | Open | ✅ Live |
| | 🇯🇵 Japan | Restricted | Open | Restricted | Restricted | ✅ Live |
| | 🇦🇺 Australia | Open | Open | Open | Open | ✅ Live |
| | 🇮🇳 India | Prohibited | Restricted | Prohibited | Restricted | ✅ Live |
| | 🇨🇳 China | Prohibited | Prohibited | Prohibited | Prohibited | ✅ Live |
| **Sanctioned** | 🇷🇺 Russia | Sanctioned | Sanctioned | Sanctioned | Sanctioned | 🚫 Blocked |
| | 🇮🇷 Iran | Sanctioned | Sanctioned | Sanctioned | Sanctioned | 🚫 Blocked |
| | 🇰🇵 North Korea | Sanctioned | Sanctioned | Sanctioned | Sanctioned | 🚫 Blocked |

> **Note:** Rule coverage is continuously updated via governance. See `docs/jurisdiction-rules/` for full rule specs per country.

---

## 🔐 Security Model

### Threat Model

| Threat Vector | Attack Scenario | ARCM Mitigation |
|--------------|----------------|----------------|
| **KYC Spoofing** | Attacker submits fake KYC credential | Only authorized anchor addresses can call `submit_credential()` |
| **Jurisdiction Bypass** | Attacker uses VPN / fake wallet location | ARCM uses blockchain analytics for country resolution, not IP |
| **Front-Running Clawback** | Holder detects pending clawback and withdraws | Lock executes before clawback in same transaction |
| **Rule Manipulation** | Attacker tries to update jurisdiction rules | All rule updates require governance vote + 48hr timelock |
| **Gateway Bypass** | Asset transferred without ARCM approval | SEP-0008 asset contract rejects unapproved transactions at protocol level |
| **Oracle Manipulation** | Corrupt sanctions data pushed by bad oracle | Multiple oracle sources; majority-vote required for sanctions flag |
| **Admin Key Compromise** | Admin multisig compromised | 3-of-5 multisig + hardware wallet requirement |
| **Replay Attacks** | Approved transaction rebroadcast | SEP-0008 approval tokens are single-use with nonce |
| **Contract Upgrade Attack** | Malicious upgrade pushed | Upgrade requires governance + 72hr timelock + audit review |

### Access Control Matrix

```
ROLE                  │ CAPABILITIES
──────────────────────┼──────────────────────────────────────────────────────
SUPER_ADMIN (3-of-5)  │ Contract upgrades, emergency pause, admin management
                      │ Rule emergency freeze, anchor authorization
──────────────────────┼──────────────────────────────────────────────────────
RULE_GOVERNOR (DAO)   │ Propose + vote on jurisdiction rule updates
                      │ Add/remove countries, update transfer policies
──────────────────────┼──────────────────────────────────────────────────────
ISSUER                │ Register assets, configure per-asset rules
                      │ Manual lock/unlock, whitelist management
                      │ View asset-specific audit logs
──────────────────────┼──────────────────────────────────────────────────────
KYC_ANCHOR            │ Submit/revoke KYC credentials (authorized anchors only)
                      │ Update credential expiry
──────────────────────┼──────────────────────────────────────────────────────
SANCTIONS_ORACLE      │ Flag/unflag wallets as sanctioned
                      │ (Chainalysis, TRM, Elliptic — authorized oracles)
──────────────────────┼──────────────────────────────────────────────────────
ARCM_GATEWAY          │ Call lock, clawback, whitelist on enforcement engine
                      │ Write to audit ledger
──────────────────────┼──────────────────────────────────────────────────────
PUBLIC                │ Read audit logs, query KYC status (own wallet only)
                      │ Submit transfer requests, renew own KYC
```

### Audit & Verification

- [ ] Internal security review (pre-testnet)
- [ ] OtterSec / Halborn external audit (planned pre-mainnet)
- [ ] Formal verification of clawback logic
- [ ] Sanctions oracle multi-source consensus audit
- [ ] Bug bounty via Immunefi (post-audit)

---

## 🛠️ Tech Stack

| Layer | Technology |
|-------|-----------|
| Smart Contracts | Rust + Soroban SDK |
| Blockchain | Stellar (Mainnet / Testnet / Futurenet) |
| Transfer Gating | SEP-0008 Regulated Assets |
| KYC Standard | SEP-0012 KYC API |
| Auth Standard | SEP-0010 Stellar Web Auth |
| KYC Providers | Synaps, Persona, Onfido (via anchor adapters) |
| Sanctions Oracles | Chainalysis KYT, TRM Labs, Elliptic |
| Country Resolution | Chainalysis + on-chain geo registry |
| Frontend (Admin UI) | Next.js 14 + TypeScript |
| Frontend (Issuer Dashboard) | Next.js 14 + TypeScript |
| Stellar SDK | @stellar/stellar-sdk |
| Keeper Bot | Node.js + Soroban RPC |
| Governance | Custom DAO contract (Soroban) |
| Testing | Soroban test framework (Rust) + Jest |
| CI/CD | GitHub Actions |
| Monitoring | Datadog + Horizon event stream |
| Audit Export | REST API + PDF report generator |

---

## 📁 Repository Structure

```
automated-rwa-compliance-middleware/
│
├── contracts/                              # All Soroban smart contracts
│   │
│   ├── gateway/                            # SEP-0008 approval gateway
│   │   ├── src/
│   │   │   ├── lib.rs                      # Entry: approve(), register_asset()
│   │   │   ├── router.rs                   # Routes to rule engine + kyc oracle
│   │   │   ├── sep0008.rs                  # SEP-0008 response formatting
│   │   │   └── errors.rs
│   │   └── Cargo.toml
│   │
│   ├── jurisdiction/                       # Jurisdiction rule engine
│   │   ├── src/
│   │   │   ├── lib.rs                      # Rule CRUD + evaluation
│   │   │   ├── rules.rs                    # Rule schema + evaluation logic
│   │   │   ├── countries.rs                # ISO 3166-1 country data
│   │   │   ├── sanctions.rs                # Sanctioned country list
│   │   │   └── storage.rs
│   │   └── Cargo.toml
│   │
│   ├── kyc_oracle/                         # KYC/AML credential oracle
│   │   ├── src/
│   │   │   ├── lib.rs                      # Credential CRUD + validation
│   │   │   ├── credential.rs               # KycCredential schema
│   │   │   ├── anchors.rs                  # Authorized anchor management
│   │   │   ├── sanctions.rs                # Wallet sanctions flagging
│   │   │   └── expiry.rs                   # Expiry tracking + alerts
│   │   └── Cargo.toml
│   │
│   ├── enforcement/                        # Lock, clawback, whitelist engine
│   │   ├── src/
│   │   │   ├── lib.rs                      # Lock, unlock, clawback, whitelist
│   │   │   ├── lock.rs                     # Lock state machine
│   │   │   ├── clawback.rs                 # Clawback execution
│   │   │   ├── registry.rs                 # Whitelist/blacklist registry
│   │   │   └── destinations.rs             # Clawback routing
│   │   └── Cargo.toml
│   │
│   ├── audit/                              # Compliance audit ledger
│   │   ├── src/
│   │   │   ├── lib.rs                      # log_event(), query_events()
│   │   │   ├── events.rs                   # ComplianceEvent schema
│   │   │   ├── report.rs                   # Report export formatting
│   │   │   └── storage.rs
│   │   └── Cargo.toml
│   │
│   ├── governance/                         # Rule governance + timelock
│   │   ├── src/
│   │   │   ├── lib.rs                      # Propose, vote, execute
│   │   │   ├── proposals.rs                # Proposal schema + voting
│   │   │   ├── timelock.rs                 # 48hr timelock enforcement
│   │   │   └── multisig.rs                 # Admin multisig logic
│   │   └── Cargo.toml
│   │
│   ├── credentials/                        # On-chain credential registry
│   │   └── src/lib.rs
│   │
│   └── geo/                                # Wallet → Country code resolver
│       ├── src/
│       │   ├── lib.rs
│       │   ├── registry.rs                 # Issuer-submitted geo mappings
│       │   └── oracle.rs                   # On-chain country oracle interface
│       └── Cargo.toml
│
├── adapters/                               # KYC provider adapters
│   ├── synaps/                             # Synaps SEP-0012 anchor adapter
│   ├── persona/                            # Persona KYC adapter
│   ├── chainalysis/                        # Chainalysis sanctions oracle adapter
│   ├── trm_labs/                           # TRM Labs adapter
│   └── generic/                            # Template for new KYC providers
│
├── keeper/                                 # Automation bot
│   ├── src/
│   │   ├── index.ts                        # Main cron entry point
│   │   ├── kyc_expiry_monitor.ts           # Monitors expiring KYC credentials
│   │   ├── sanctions_sync.ts               # Syncs sanctions oracle data
│   │   ├── lock_enforcer.ts                # Triggers locks on violations
│   │   └── clawback_executor.ts            # Triggers clawback pipeline
│   └── package.json
│
├── frontend/
│   ├── issuer-dashboard/                   # Dashboard for RWA issuers
│   │   ├── src/
│   │   │   ├── app/
│   │   │   │   ├── page.tsx                # Asset overview + compliance status
│   │   │   │   ├── rules/page.tsx          # Jurisdiction rule viewer
│   │   │   │   ├── holders/page.tsx        # Holder KYC + lock status
│   │   │   │   └── audit/page.tsx          # Audit log explorer
│   │   │   └── components/
│   │   │       ├── ComplianceMap.tsx       # World map of rule coverage
│   │   │       ├── HolderTable.tsx         # KYC status per holder
│   │   │       ├── ClawbackHistory.tsx     # Past clawbacks
│   │   │       └── RuleEditor.tsx          # Propose rule changes
│   │   └── package.json
│   │
│   └── user-portal/                        # KYC status portal for holders
│       ├── src/
│       │   ├── app/
│       │   │   ├── page.tsx                # My KYC status
│       │   │   └── kyc/page.tsx            # KYC renewal flow
│       │   └── components/
│       │       ├── KycStatusCard.tsx
│       │       └── HoldingsCompliance.tsx
│       └── package.json
│
├── scripts/
│   ├── deploy_all.sh                       # Deploy full contract suite
│   ├── initialize.sh                       # Post-deploy contract linking
│   ├── seed_jurisdiction_rules.sh          # Load default rule set
│   ├── register_anchors.sh                 # Authorize KYC anchors
│   └── seed_sanctions.sh                   # Load initial sanctions lists
│
├── docs/
│   ├── architecture.md
│   ├── sep0008-integration.md
│   ├── sep0012-anchor-guide.md
│   ├── jurisdiction-rules/
│   │   ├── US.md
│   │   ├── EU.md
│   │   ├── NG.md
│   │   └── ...
│   ├── clawback-procedures.md
│   └── regulator-reporting-guide.md
│
├── tests/
│   ├── unit/
│   │   ├── jurisdiction_tests.rs
│   │   ├── kyc_oracle_tests.rs
│   │   ├── enforcement_tests.rs
│   │   └── audit_tests.rs
│   └── integration/
│       ├── full_compliance_flow_test.rs
│       ├── clawback_flow_test.rs
│       └── sanctions_hit_test.rs
│
├── Cargo.toml                              # Workspace manifest
└── README.md
```

---

## 🚀 Getting Started

### Prerequisites

```bash
# 1. Install Rust + Wasm target
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# 2. Install Soroban CLI
cargo install --locked soroban-cli --features opt

# 3. Install Node.js 18+ (keeper + frontend)
nvm install 18 && nvm use 18

# 4. Verify installation
soroban --version
node --version
```

### Environment Configuration

```bash
cp .env.example .env
vim .env
```

```env
# .env — ARCM Configuration

# ── NETWORK ──────────────────────────────────────────
STELLAR_NETWORK=testnet
SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
STELLAR_HORIZON_URL=https://horizon-testnet.stellar.org
NETWORK_PASSPHRASE="Test SDF Network ; September 2015"

# ── CONTRACT IDs (populated after deployment) ─────────
ARCM_GATEWAY_CONTRACT_ID=
JURISDICTION_ENGINE_CONTRACT_ID=
KYC_ORACLE_CONTRACT_ID=
ENFORCEMENT_ENGINE_CONTRACT_ID=
AUDIT_LEDGER_CONTRACT_ID=
GOVERNANCE_CONTRACT_ID=
CREDENTIAL_REGISTRY_CONTRACT_ID=
GEO_RESOLVER_CONTRACT_ID=

# ── ADMIN KEYS ────────────────────────────────────────
SUPER_ADMIN_KEY_1=S...      # Use hardware wallet in production
SUPER_ADMIN_KEY_2=S...
SUPER_ADMIN_KEY_3=S...

# ── KYC ANCHOR CONFIGURATION ─────────────────────────
SYNAPS_ANCHOR_ADDRESS=G...
PERSONA_ANCHOR_ADDRESS=G...
ONFIDO_ANCHOR_ADDRESS=G...

# ── SANCTIONS ORACLE CONFIGURATION ───────────────────
CHAINALYSIS_ORACLE_ADDRESS=G...
TRM_LABS_ORACLE_ADDRESS=G...
ELLIPTIC_ORACLE_ADDRESS=G...

# ── KEEPER BOT ────────────────────────────────────────
KEEPER_SECRET_KEY=S...
KYC_EXPIRY_ALERT_DAYS=14    # Alert this many days before KYC expiry
SANCTIONS_SYNC_INTERVAL=3600 # Seconds between sanctions list refresh
```

### Build Contracts

```bash
# Build all contracts
cargo build --target wasm32-unknown-unknown --release

# Optimize all Wasm binaries
for contract in gateway jurisdiction kyc_oracle enforcement audit governance; do
  soroban contract optimize \
    --wasm target/wasm32-unknown-unknown/release/${contract}.wasm \
    --wasm-out target/wasm32-unknown-unknown/release/${contract}.optimized.wasm
done

# Run tests
cargo test --workspace
```

---

## 🌐 Contract Deployment

### Full Suite Deployment

```bash
# 1. Generate and fund admin account
soroban keys generate arcm-admin --network testnet
soroban keys fund arcm-admin --network testnet

# 2. Run automated deployment script
./scripts/deploy_all.sh

# 3. Initialize contract relationships
./scripts/initialize.sh

# 4. Seed jurisdiction rules (loads default rule set from docs/jurisdiction-rules/)
./scripts/seed_jurisdiction_rules.sh

# 5. Register KYC anchors
./scripts/register_anchors.sh \
  --synaps $SYNAPS_ANCHOR_ADDRESS \
  --persona $PERSONA_ANCHOR_ADDRESS

# 6. Load initial sanctions lists
./scripts/seed_sanctions.sh
```

### Initialize Gateway

```bash
soroban contract invoke \
  --id $ARCM_GATEWAY_CONTRACT_ID \
  --source arcm-admin \
  --network testnet \
  -- \
  initialize \
  --admin $ADMIN_ADDRESS \
  --jurisdiction_engine $JURISDICTION_ENGINE_CONTRACT_ID \
  --kyc_oracle $KYC_ORACLE_CONTRACT_ID \
  --enforcement_engine $ENFORCEMENT_ENGINE_CONTRACT_ID \
  --audit_ledger $AUDIT_LEDGER_CONTRACT_ID \
  --geo_resolver $GEO_RESOLVER_CONTRACT_ID
```

### Register a Sample RWA Asset

```bash
soroban contract invoke \
  --id $ARCM_GATEWAY_CONTRACT_ID \
  --source issuer-key \
  --network testnet \
  -- \
  register_asset \
  --issuer $ISSUER_ADDRESS \
  --asset_contract $PROPERTY_TOKEN_CONTRACT_ID \
  --asset_class RealEstate \
  --min_kyc_tier 2 \
  --requires_issuer_approval false \
  --clawback_enabled true
```

---

## 📡 API Reference

### Gateway Contract

#### `approve` — Core SEP-0008 approval endpoint

```rust
pub fn approve(
    env: Env,
    source_account: Address,
    stellar_tx: StellarTransaction,
) -> ApprovalResponse

// Returns:
pub struct ApprovalResponse {
    pub status: ApprovalStatus,      // Approved | Rejected | Revised | Pending
    pub reason_code: Option<ReasonCode>,
    pub revised_amount: Option<u128>,
    pub audit_ref: String,           // Reference to audit log entry
    pub message: Option<String>,     // Human-readable explanation
}
```

#### `register_asset` — Register RWA for compliance gating

```rust
pub fn register_asset(
    env: Env,
    issuer: Address,             // Must authorize
    asset_contract: Address,
    asset_class: AssetClass,
    rule_config: IssuerRuleConfig,
)
```

### Jurisdiction Engine

#### `get_rule` — Fetch jurisdiction rule

```rust
pub fn get_rule(env: Env, country_code: String, asset_class: AssetClass) -> JurisdictionRule
```

#### `propose_rule_update` — Submit governance rule change

```rust
pub fn propose_rule_update(
    env: Env,
    proposer: Address,
    country_code: String,
    asset_class: AssetClass,
    new_rule: JurisdictionRule,
) -> u64  // Proposal ID
```

### KYC Oracle

#### `get_kyc_status` — Query wallet KYC

```rust
pub fn get_kyc_status(env: Env, wallet: Address) -> Option<KycCredential>
```

#### `is_kyc_valid` — Check tier + expiry in one call

```rust
pub fn is_kyc_valid(env: Env, wallet: Address, required_tier: u8) -> bool
```

### Enforcement Engine

#### `lock_asset` — Freeze a wallet's holdings

```rust
pub fn lock_asset(
    env: Env,
    authority: Address,
    asset_contract: Address,
    wallet: Address,
    reason: ReasonCode,
    duration: Option<u64>,  // Seconds; None = indefinite
)
```

#### `execute_clawback` — Reclaim assets from holder

```rust
pub fn execute_clawback(
    env: Env,
    authority: Address,
    asset_contract: Address,
    holder: Address,
    amount: Option<u128>,
    reason: ReasonCode,
    destination: Address,
)
```

### Audit Ledger

#### `query_events` — Paginated event query

```rust
pub fn query_events(
    env: Env,
    asset_contract: Address,
    from_id: u64,
    limit: u32,
) -> Vec<ComplianceEvent>
```

#### `export_report` — Generate compliance report

```rust
pub fn export_report(
    env: Env,
    asset_contract: Address,
    from_timestamp: u64,
    to_timestamp: u64,
) -> ComplianceReport
```

---

## 🔌 Integration Guide for Asset Issuers

### Step 1: Set Up Your stellar.toml

```toml
# stellar.toml
[[REGULATED_ASSETS]]
code = "PROP001"
issuer = "G..."
approval_server = "https://arcm.yourprotocol.com/v1/approve"
approval_criteria = """
Transfers require:
- KYC Tier 2 (accredited investor) for US holders
- KYC Tier 1 for most other jurisdictions
- 90-day minimum holding period
See full rules at https://yourprotocol.com/compliance
"""
```

### Step 2: Register Asset with ARCM

```typescript
import { Contract, Networks } from "@stellar/stellar-sdk";

const registerAsset = async (issuerKeypair, assetContractId) => {
  await arcmGateway.invoke("register_asset", {
    issuer: issuerKeypair.publicKey(),
    asset_contract: assetContractId,
    asset_class: "RealEstate",
    rule_config: {
      min_kyc_tier: 2,           // Accredited investors only
      requires_issuer_approval: false,
      clawback_enabled: true,
      custom_hold_period: 7776000, // 90 days in seconds
    }
  });
};
```

### Step 3: Implement Approval Server

ARCM provides a hosted approval server, but issuers can also self-host:

```typescript
// Express.js approval server wrapper (optional — use hosted ARCM instead)
app.post('/v1/approve', async (req, res) => {
  const { tx } = req.body;

  // Forward to ARCM Gateway contract
  const decision = await arcmGateway.call('approve', {
    source_account: extractSourceAccount(tx),
    stellar_tx: tx,
  });

  // Format as SEP-0008 response
  res.json(formatSep0008Response(decision));
});
```

### Step 4: Configure KYC Anchor

```bash
# Authorize a KYC anchor for your asset
soroban contract invoke \
  --id $KYC_ORACLE_CONTRACT_ID \
  --source arcm-admin \
  -- \
  authorize_anchor \
  --anchor $SYNAPS_ANCHOR_ADDRESS \
  --asset_contracts '["$PROPERTY_TOKEN_CONTRACT_ID"]'
```

### Step 5: Monitor Compliance Events

```typescript
// Subscribe to compliance events for your asset
const server = new SorobanRpc.Server(SOROBAN_RPC_URL);

const watchComplianceEvents = async (assetContractId: string) => {
  const events = await server.getEvents({
    startLedger: currentLedger,
    filters: [
      {
        type: "contract",
        contractIds: [AUDIT_LEDGER_CONTRACT_ID],
        topics: [["compliance_event", assetContractId]],
      },
    ],
  });

  for (const event of events.events) {
    const decoded = decodeComplianceEvent(event.value);
    if (decoded.action === "Clawback" || decoded.action === "Lock") {
      await notifyIssuers(decoded);
    }
  }
};
```

---

## 🧪 Testing

### Unit Tests

```bash
# All contract unit tests
cargo test --workspace

# Specific contract
cargo test -p jurisdiction_engine
cargo test -p kyc_oracle
cargo test -p enforcement

# With verbose output + no test capture
cargo test -- --nocapture
```

### Integration Tests

```bash
# Full compliance flow (deposit → transfer → KYC check → approve)
cargo test --test full_compliance_flow

# Clawback flow (sanctions hit → lock → clawback → audit log)
cargo test --test clawback_flow

# Sanctions hit scenario
cargo test --test sanctions_hit
```

### Key Test Scenarios

| Scenario | Test File | Validates |
|----------|-----------|-----------|
| Happy path transfer (all checks pass) | `full_compliance_flow` | End-to-end approval |
| Prohibited jurisdiction transfer | `jurisdiction_tests` | Rule evaluation accuracy |
| Expired KYC soft lock | `kyc_oracle_tests` | Expiry detection + lock |
| OFAC sanctions clawback | `sanctions_hit` | Clawback execution + audit |
| Holding period not met | `jurisdiction_tests` | Timestamp comparison |
| Accredited investor check (US) | `jurisdiction_tests` | Tier 2 KYC gate |
| Revised transfer (amount cap) | `full_compliance_flow` | SEP-0008 revise response |
| Governance rule update + timelock | `governance_tests` | Timelock enforcement |
| Multi-oracle sanctions consensus | `sanctions_hit` | Oracle quorum logic |

---

## 🏛️ Governance & Rule Updates

### Rule Update Lifecycle

```
STEP 1: PROPOSE
  Any governance token holder calls:
  jurisdiction_engine.propose_rule_update(country, asset_class, new_rule)
  → Proposal created with 48-hour voting window

STEP 2: VOTE
  Governance token holders vote: approve / reject
  Quorum: 10% of tokens must vote
  Threshold: 51% approval required

STEP 3: TIMELOCK
  If approved → 48-hour timelock begins
  During timelock: community can raise objections
  Emergency veto: 3-of-5 admin multisig can block

STEP 4: EXECUTE
  After timelock: any address can call execute_rule_update(proposal_id)
  → New rule takes effect for all future transfers
  → Old rule version preserved in audit history
```

### Emergency Rule Freeze

In the event of a regulatory emergency (e.g., sudden country sanctions):

```bash
# Admin multisig (3-of-5) can immediately freeze a country's rules
soroban contract invoke \
  --id $JURISDICTION_ENGINE_CONTRACT_ID \
  --source admin-multisig \
  -- \
  emergency_freeze_country \
  --country_code "XX" \
  --reason "OFAC executive order EO-XXXXX" \
  --duration 604800   # 7 days freeze while full rule update is prepared
```

---

## ⚠️ Legal & Regulatory Disclaimer

> **ARCM is infrastructure, not legal advice.**

ARCM enforces rules that **you as the issuer configure and are legally responsible for**. The protocol:

- Does **not** independently interpret securities law
- Does **not** guarantee full regulatory compliance in any jurisdiction
- Does **not** replace the need for qualified legal counsel per jurisdiction
- Does **not** store or process personal data (PII stays with KYC anchors)
- **Does** provide a technical enforcement layer for rules you have determined apply to your asset

**Issuers must independently verify jurisdiction rules with local legal counsel before deploying any tokenized RWA.**

The clawback mechanism requires the underlying Stellar asset to have `AUTHORIZATION_REQUIRED`, `AUTHORIZATION_REVOCABLE`, and `CLAWBACK_ENABLED` flags set at issuance. These flags cannot be added after the asset is created.

---

## 🗺️ Roadmap

### Phase 1 — Core Infrastructure
- [x] Architecture design + SEP analysis
- [x] Gateway + Jurisdiction Engine contracts
- [x] KYC Oracle contract + SEP-0012 integration
- [ ] Enforcement Engine (lock + clawback)
- [ ] Audit Ledger contract
- [ ] Testnet deployment

### Phase 2 — KYC & Sanctions
- [ ] Synaps SEP-0012 anchor adapter
- [ ] Chainalysis KYT sanctions oracle integration
- [ ] TRM Labs adapter
- [ ] KYC expiry keeper bot
- [ ] Issuer dashboard (Alpha)

### Phase 3 — Jurisdiction Expansion
- [ ] Full G20 jurisdiction rule set
- [ ] Africa-specific rule library (NG, KE, GH, ZA)
- [ ] MiCA compliance module (EU-wide)
- [ ] Regulation D / Regulation CF (US) module
- [ ] Governance contract + DAO token

### Phase 4 — Mainnet & Integrations
- [ ] Security audit (OtterSec / Halborn)
- [ ] Mainnet deployment
- [ ] Real estate token issuer pilots
- [ ] Commodity token issuer pilots
- [ ] Regulator reporting API

### Phase 5 — Ecosystem
- [ ] Cross-chain compliance bridge (Ethereum ↔ Stellar)
- [ ] ARCM SDK for third-party issuers
- [ ] Compliance-as-a-Service API
- [ ] Insurance integration (DeFi coverage for compliant assets)
- [ ] Automated regulatory report submission

---

## 🤝 Contributing

### Development Workflow

```bash
# Fork and clone
git clone https://github.com/your-org/automated-rwa-compliance-middleware.git

# Create feature branch
git checkout -b feature/your-feature-name

# Make changes + test
cargo test --workspace

# Submit PR against main
```

### Contribution Areas

- 🌍 **Jurisdiction rules** — Add or improve rules for new countries in `docs/jurisdiction-rules/`
- 🔌 **KYC adapters** — Build new anchor adapters in `adapters/`
- 🧪 **Test coverage** — Add edge case tests for enforcement + clawback logic
- 📖 **Documentation** — Improve integration guides, add real-world examples
- 🔐 **Security** — Review contract logic, submit security issues via private disclosure

### Security Disclosure

For security vulnerabilities, **do not open a public GitHub issue**. Email `security@yourprotocol.com` with:
- Description of the vulnerability
- Steps to reproduce
- Potential impact assessment
- Suggested fix (if any)

---

## 📄 License

This project is licensed under the **MIT License** — see the [LICENSE](./LICENSE) file for details.

---

## 🙏 Acknowledgements

- [Stellar Development Foundation](https://stellar.org) — SEP-0008 regulated assets standard + Soroban
- [Circle](https://circle.com) — Native USDC infrastructure on Stellar
- [Chainalysis](https://chainalysis.com) — Sanctions oracle and KYT tooling
- [Synaps](https://synaps.io) — SEP-0012 KYC anchor infrastructure
- [Franklin Templeton](https://franklintempleton.com) — Pioneering tokenized assets on Stellar
- The global RWA tokenization community pushing the legal and technical frontier

---

<div align="center">

**Built with ⚖️ for a compliant tokenized future on Stellar**

[Website](#) · [Documentation](#) · [Discord](#) · [Twitter](#) · [Security](#)

*Autonomous. Jurisdiction-aware. Unstoppable compliance.*

</div>
