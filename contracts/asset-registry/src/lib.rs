use near_contract_standards::non_fungible_token::{
    NonFungibleToken, Token, TokenId,
    core::NonFungibleTokenCore,
    metadata::{NFT_METADATA_SPEC, NFTContractMetadata, TokenMetadata},
};
use near_sdk::borsh::BorshSerialize;
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk::store::{LookupMap, LookupSet};
use near_sdk::{
    AccountId, BorshStorageKey, Gas, NearToken, PanicOnDefault, Promise, env, near, require,
};

const MIN_WORK_ORDER_DEPOSIT: u128 = 10_000_000_000_000_000_000_000; // 0.01 NEAR for durable payout records
const MIN_WORK_ORDER_STORAGE_DEPOSIT: u128 = 50_000_000_000_000_000_000_000; // 0.05 NEAR per persistent work order
const MIN_NFT_STORAGE_DEPOSIT: u128 = 100_000_000_000_000_000_000_000; // 0.1 NEAR for registry/NFT storage
const ASSET_ACCOUNT_OPERATIONAL_BUFFER: u128 = 250_000_000_000_000_000_000_000; // 0.25 NEAR after code storage
const ASSET_ACCOUNT_CODE: &[u8] = include_bytes!("../res/asset-account.wasm");

/// The asset creator, not the shared registry treasury, funds the child account.
fn child_account_initial_balance() -> u128 {
    let code_storage_cost = (ASSET_ACCOUNT_CODE.len() as u128)
        .checked_mul(env::storage_byte_cost().as_yoctonear())
        .unwrap_or_else(|| env::panic_str("Asset account code storage cost overflow"));
    code_storage_cost
        .checked_add(ASSET_ACCOUNT_OPERATIONAL_BUFFER)
        .unwrap_or_else(|| env::panic_str("Asset account initial balance overflow"))
}

fn required_asset_creation_deposit() -> u128 {
    child_account_initial_balance()
        .checked_add(MIN_NFT_STORAGE_DEPOSIT)
        .unwrap_or_else(|| env::panic_str("Asset creation deposit overflow"))
}

/// Keep create and retry on the same provisioning path so their callback semantics cannot drift.
fn provision_asset_account(
    asset_account_id: AccountId,
    registry_id: AccountId,
    token_id: TokenId,
    reserve_bps: u16,
    initial_balance: u128,
) -> Promise {
    let init_args = json!({
        "registry_id": registry_id.clone(),
        "token_id": token_id.clone(),
        "reserve_bps": reserve_bps
    })
    .to_string()
    .into_bytes();
    let callback_args = json!({ "token_id": token_id }).to_string().into_bytes();

    Promise::new(asset_account_id)
        .create_account()
        .transfer(NearToken::from_yoctonear(initial_balance))
        .deploy_contract(ASSET_ACCOUNT_CODE.to_vec())
        .function_call(
            "new_vault".to_string(),
            init_args,
            NearToken::from_yoctonear(0),
            Gas::from_tgas(30),
        )
        .then(Promise::new(registry_id).function_call(
            "on_asset_created".to_string(),
            callback_args,
            NearToken::from_yoctonear(0),
            Gas::from_tgas(15),
        ))
}

#[derive(BorshStorageKey, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
enum StorageKey {
    NftOwnerById,
    NftMetadataById,
    Assets,
    PendingWorkOrders,
    PaidWorkOrders,
    ProvisioningCredit,
    WorkOrders,
    WorkOrderIds,
    OwnershipHistory,
    OwnershipStorageCredit,
}

// Stored independently to preserve the existing serialized contract state layout.
fn provisioning_credits() -> LookupMap<TokenId, u128> {
    LookupMap::new(StorageKey::ProvisioningCredit)
}

/// Stored independently so adding ownership history does not change the serialized
/// Contract state layout or require migration of an already-deployed registry.
fn ownership_history() -> LookupMap<TokenId, Vec<OwnershipEvent>> {
    LookupMap::new(StorageKey::OwnershipHistory)
}

fn ownership_storage_credits() -> LookupMap<TokenId, u128> {
    LookupMap::new(StorageKey::OwnershipStorageCredit)
}

fn assert_provisioning_gas() {
    require!(
        env::prepaid_gas().as_gas() >= Gas::from_tgas(100).as_gas(),
        "Attach at least 100 Tgas for provisioning"
    );
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: NFTContractMetadata,
    assets: LookupMap<TokenId, AssetRecord>,
    pending_work_orders: LookupSet<String>,
    paid_work_orders: LookupSet<String>,
    work_orders: LookupMap<String, WorkOrder>,
    work_order_ids: LookupMap<TokenId, Vec<String>>,
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct AssetRecord {
    pub token_id: String,
    pub name: String,
    pub asset_type: String,
    pub asset_account_id: AccountId,
    pub reserve_bps: u16,
    pub created_by: AccountId,
    pub created_at_ns: u64,
    pub status: String,
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct OwnershipEvent {
    pub owner_id: AccountId,
    pub previous_owner_id: Option<AccountId>,
    pub timestamp_ns: u64,
    pub memo: Option<String>,
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct WorkOrder {
    pub work_order_id: String,
    pub token_id: String,
    pub observation: String,
    pub observation_evidence_hash: Option<String>,
    pub diagnosis: String,
    pub diagnosis_source: String,
    pub recommended_action: String,
    pub priority: String,
    pub created_by: AccountId,
    pub created_at_ns: u64,
    pub status: String,
    pub technician_id: Option<AccountId>,
    pub quote_amount_yocto: Option<String>,
    pub quote_note: Option<String>,
    pub quoted_at_ns: Option<u64>,
    pub approved_by: Option<AccountId>,
    pub approved_at_ns: Option<u64>,
    pub completion_note: Option<String>,
    pub completion_evidence_hash: Option<String>,
    pub completion_evidence_uri: Option<String>,
    pub completed_at_ns: Option<u64>,
    pub paid_amount_yocto: Option<String>,
    pub paid_at_ns: Option<u64>,
}

fn work_order_key(token_id: &str, work_order_id: &str) -> String {
    format!("{}:{}", token_id, work_order_id)
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|c| c.is_ascii_hexdigit())
}

#[near]
impl Contract {
    #[init]
    pub fn new() -> Self {
        let current = env::current_account_id();
        Self {
            tokens: NonFungibleToken::new(
                StorageKey::NftOwnerById,
                current,
                Some(StorageKey::NftMetadataById),
                None::<StorageKey>,
                None::<StorageKey>,
            ),
            metadata: NFTContractMetadata {
                spec: NFT_METADATA_SPEC.to_string(),
                name: "Asset Operating Accounts".to_string(),
                symbol: "ASSET".to_string(),
                icon: None,
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
            assets: LookupMap::new(StorageKey::Assets),
            pending_work_orders: LookupSet::new(StorageKey::PendingWorkOrders),
            paid_work_orders: LookupSet::new(StorageKey::PaidWorkOrders),
            work_orders: LookupMap::new(StorageKey::WorkOrders),
            work_order_ids: LookupMap::new(StorageKey::WorkOrderIds),
        }
    }

    /// Mint an NEP-171 asset passport and create its dedicated child account.
    /// The creator funds child-account provisioning and bounded registry storage.
    #[payable]
    pub fn create_asset(
        &mut self,
        token_id: TokenId,
        name: String,
        asset_type: String,
        reserve_bps: u16,
    ) -> Promise {
        assert_provisioning_gas();
        let storage_before = env::storage_usage();
        let initial_balance = child_account_initial_balance();
        require!(
            env::attached_deposit().as_yoctonear() == required_asset_creation_deposit(),
            "Attach exactly the amount returned by get_asset_creation_cost"
        );
        require!(
            token_id.len() >= 3 && token_id.len() <= 24,
            "Asset token id must be 3-24 characters"
        );
        require!(
            !token_id.contains('.')
                && token_id
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            "Asset token id must be a lowercase sub-account label"
        );
        require!(
            !name.trim().is_empty() && name.len() <= 128,
            "Asset name must be 1-128 bytes"
        );
        require!(
            !asset_type.trim().is_empty() && asset_type.len() <= 64,
            "Asset type must be 1-64 bytes"
        );
        require!(
            reserve_bps <= 10_000,
            "Reserve percentage cannot exceed 100%"
        );
        require!(
            self.assets.get(&token_id).is_none(),
            "Asset token id already exists"
        );

        let registry_id = env::current_account_id();
        let asset_account_id: AccountId = format!("{}.{}", token_id, registry_id)
            .parse()
            .unwrap_or_else(|_| env::panic_str("Could not derive a valid asset sub-account"));
        let owner_id = env::predecessor_account_id();

        let token_metadata = TokenMetadata {
            title: Some(name.clone()),
            description: Some(format!("{} operating account", asset_type)),
            media: None,
            media_hash: None,
            copies: Some(1),
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: Some(
                json!({
                    "asset_account_id": asset_account_id.to_string(),
                    "asset_type": asset_type.clone(),
                    "reserve_bps": reserve_bps,
                    "purpose": "physical_asset_operating_account"
                })
                .to_string(),
            ),
            reference: None,
            reference_hash: None,
        };

        // internal_mint would refund the child funding as excess NFT storage deposit.
        self.tokens.internal_mint_with_refund(
            token_id.clone(),
            owner_id.clone(),
            Some(token_metadata),
            None,
        );
        near_contract_standards::non_fungible_token::events::NftMint {
            owner_id: &owner_id,
            token_ids: &[&token_id],
            memo: None,
        }
        .emit();

        ownership_history().insert(
            token_id.clone(),
            vec![OwnershipEvent {
                owner_id: owner_id.clone(),
                previous_owner_id: None,
                timestamp_ns: env::block_timestamp(),
                memo: Some("Initial mint".to_string()),
            }],
        );

        self.assets.insert(
            token_id.clone(),
            AssetRecord {
                token_id: token_id.clone(),
                name,
                asset_type,
                asset_account_id: asset_account_id.clone(),
                reserve_bps,
                created_by: owner_id,
                created_at_ns: env::block_timestamp(),
                status: "PROVISIONING".to_string(),
            },
        );

        ownership_storage_credits().insert(token_id.clone(), 0);
        provisioning_credits().insert(token_id.clone(), initial_balance);
        self.assets.flush();
        let storage_cost = u128::from(env::storage_usage() - storage_before)
            .checked_mul(env::storage_byte_cost().as_yoctonear())
            .unwrap_or_else(|| env::panic_str("Registry storage cost overflow"));
        require!(
            storage_cost <= MIN_NFT_STORAGE_DEPOSIT,
            "Registry storage allowance exceeded"
        );
        // Unused creator-paid registry storage stays assigned to this passport.
        ownership_storage_credits()
            .insert(token_id.clone(), MIN_NFT_STORAGE_DEPOSIT - storage_cost);

        // The caller funds the child account. The registry never subsidizes permissionless mints.
        provision_asset_account(
            asset_account_id,
            registry_id,
            token_id,
            reserve_bps,
            initial_balance,
        )
    }

    /// Private callback records whether the child account was successfully created and initialized.
    #[private]
    pub fn on_asset_created(&mut self, token_id: TokenId) -> bool {
        let mut record = self
            .assets
            .get(&token_id)
            .cloned()
            .unwrap_or_else(|| env::panic_str("Pending asset record not found"));
        require!(
            record.status == "PROVISIONING",
            "Asset is not awaiting provisioning"
        );
        require!(
            env::promise_results_count() == 1,
            "Expected exactly one provisioning result"
        );
        // SDK initializers write state and return no bytes. Only a definite receipt
        // failure makes the deposit reusable; unexpected successful output must not
        // trigger another transfer from the registry treasury.
        let (success, status) = match env::promise_result_checked(0, 0) {
            Ok(_) => {
                provisioning_credits().remove(&token_id);
                (true, "ACTIVE")
            }
            Err(near_sdk::PromiseError::Failed) => (false, "FAILED"),
            Err(_) => {
                provisioning_credits().remove(&token_id);
                (false, "PROVISIONING_UNKNOWN")
            }
        };
        record.status = status.to_string();
        self.assets.insert(token_id, record);
        success
    }

    pub fn get_work_order_creation_cost(&self) -> U128 {
        U128(MIN_WORK_ORDER_STORAGE_DEPOSIT)
    }

    /// Owner records an observed issue and the bounded coordinator's diagnosis.
    /// The coordinator cannot approve work or move funds.
    #[payable]
    pub fn create_work_order(
        &mut self,
        token_id: TokenId,
        work_order_id: String,
        observation: String,
        observation_evidence_hash: Option<String>,
        diagnosis: String,
        diagnosis_source: String,
        recommended_action: String,
        priority: String,
    ) -> WorkOrder {
        let owner_id = self
            .tokens
            .owner_by_id
            .get(&token_id)
            .unwrap_or_else(|| env::panic_str("Asset NFT not found"));
        require!(
            owner_id == env::predecessor_account_id(),
            "Only the current NFT owner can create a work order"
        );
        let asset = self
            .assets
            .get(&token_id)
            .cloned()
            .unwrap_or_else(|| env::panic_str("Asset account record not found"));
        require!(asset.status == "ACTIVE", "Asset account is not active");
        require!(
            env::attached_deposit().as_yoctonear() == MIN_WORK_ORDER_STORAGE_DEPOSIT,
            "Attach exactly the amount returned by get_work_order_creation_cost"
        );
        require!(
            work_order_id.len() >= 3
                && work_order_id.len() <= 64
                && !work_order_id.contains(':')
                && work_order_id
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-'),
            "Work order id must be 3-64 alphanumeric or dash characters"
        );
        require!(
            observation.trim().len() >= 10 && observation.len() <= 1000,
            "Observation must be 10-1000 bytes"
        );
        require!(
            diagnosis.trim().len() >= 5 && diagnosis.len() <= 500,
            "Diagnosis must be 5-500 bytes"
        );
        require!(
            recommended_action.trim().len() >= 5 && recommended_action.len() <= 500,
            "Recommended action must be 5-500 bytes"
        );
        require!(
            diagnosis_source.trim().len() > 0 && diagnosis_source.len() <= 64,
            "Diagnosis source must be 1-64 bytes"
        );
        require!(
            ["LOW", "MEDIUM", "HIGH", "CRITICAL"].contains(&priority.as_str()),
            "Priority must be LOW, MEDIUM, HIGH or CRITICAL"
        );
        if let Some(hash) = observation_evidence_hash.as_ref() {
            require!(
                valid_sha256(hash),
                "Observation evidence must be a SHA-256 hex digest"
            );
        }
        let key = work_order_key(&token_id, &work_order_id);
        require!(
            self.work_orders.get(&key).is_none(),
            "Work order already exists"
        );
        let order = WorkOrder {
            work_order_id: work_order_id.clone(),
            token_id: token_id.clone(),
            observation,
            observation_evidence_hash,
            diagnosis,
            diagnosis_source,
            recommended_action,
            priority,
            created_by: owner_id,
            created_at_ns: env::block_timestamp(),
            status: "DIAGNOSED".to_string(),
            technician_id: None,
            quote_amount_yocto: None,
            quote_note: None,
            quoted_at_ns: None,
            approved_by: None,
            approved_at_ns: None,
            completion_note: None,
            completion_evidence_hash: None,
            completion_evidence_uri: None,
            completed_at_ns: None,
            paid_amount_yocto: None,
            paid_at_ns: None,
        };
        let mut ids = self
            .work_order_ids
            .get(&token_id)
            .cloned()
            .unwrap_or_default();
        ids.push(work_order_id.clone());
        self.work_order_ids.insert(token_id, ids);
        self.work_orders.insert(key, order.clone());
        env::log_str(
            &near_sdk::serde_json::json!({
                "event": "maintenance_work_order_created",
                "token_id": order.token_id,
                "work_order_id": order.work_order_id,
                "priority": order.priority,
                "diagnosis_source": order.diagnosis_source
            })
            .to_string(),
        );
        order
    }

    /// The current owner chooses the technician account; there is no open marketplace.
    pub fn assign_technician(
        &mut self,
        token_id: TokenId,
        work_order_id: String,
        technician_id: AccountId,
    ) -> WorkOrder {
        let owner_id = self
            .tokens
            .owner_by_id
            .get(&token_id)
            .unwrap_or_else(|| env::panic_str("Asset NFT not found"));
        require!(
            owner_id == env::predecessor_account_id(),
            "Only the current NFT owner can assign a technician"
        );
        require!(
            technician_id != owner_id,
            "Technician must differ from the asset owner"
        );
        let key = work_order_key(&token_id, &work_order_id);
        let mut order = self
            .work_orders
            .get(&key)
            .cloned()
            .unwrap_or_else(|| env::panic_str("Work order not found"));
        require!(
            order.status == "DIAGNOSED",
            "Work order is not awaiting technician assignment"
        );
        order.technician_id = Some(technician_id);
        order.status = "AWAITING_QUOTE".to_string();
        self.work_orders.insert(key, order.clone());
        order
    }

    /// Only the assigned technician can submit a quote.
    pub fn submit_technician_quote(
        &mut self,
        token_id: TokenId,
        work_order_id: String,
        quote_amount: U128,
        quote_note: String,
    ) -> WorkOrder {
        let key = work_order_key(&token_id, &work_order_id);
        let mut order = self
            .work_orders
            .get(&key)
            .cloned()
            .unwrap_or_else(|| env::panic_str("Work order not found"));
        require!(
            order.technician_id.as_ref() == Some(&env::predecessor_account_id()),
            "Only the assigned technician can submit this quote"
        );
        require!(
            order.status == "AWAITING_QUOTE",
            "Work order is not awaiting a quote"
        );
        require!(quote_amount.0 > 0, "Quote must be greater than zero");
        require!(
            quote_note.trim().len() >= 5 && quote_note.len() <= 500,
            "Quote note must be 5-500 bytes"
        );
        order.quote_amount_yocto = Some(quote_amount.0.to_string());
        order.quote_note = Some(quote_note);
        order.quoted_at_ns = Some(env::block_timestamp());
        order.status = "QUOTED".to_string();
        self.work_orders.insert(key, order.clone());
        order
    }

    /// The owner must explicitly approve the technician's quote.
    pub fn approve_work_order_quote(
        &mut self,
        token_id: TokenId,
        work_order_id: String,
    ) -> WorkOrder {
        let owner_id = self
            .tokens
            .owner_by_id
            .get(&token_id)
            .unwrap_or_else(|| env::panic_str("Asset NFT not found"));
        require!(
            owner_id == env::predecessor_account_id(),
            "Only the current NFT owner can approve a quote"
        );
        let key = work_order_key(&token_id, &work_order_id);
        let mut order = self
            .work_orders
            .get(&key)
            .cloned()
            .unwrap_or_else(|| env::panic_str("Work order not found"));
        require!(order.status == "QUOTED", "Work order has no pending quote");
        order.approved_by = Some(owner_id);
        order.approved_at_ns = Some(env::block_timestamp());
        order.status = "APPROVED".to_string();
        self.work_orders.insert(key, order.clone());
        order
    }

    /// Assigned technician submits a completion report and a SHA-256 evidence reference.
    pub fn submit_work_order_completion(
        &mut self,
        token_id: TokenId,
        work_order_id: String,
        completion_note: String,
        completion_evidence_hash: String,
        completion_evidence_uri: String,
    ) -> WorkOrder {
        let key = work_order_key(&token_id, &work_order_id);
        let mut order = self
            .work_orders
            .get(&key)
            .cloned()
            .unwrap_or_else(|| env::panic_str("Work order not found"));
        require!(
            order.technician_id.as_ref() == Some(&env::predecessor_account_id()),
            "Only the assigned technician can submit completion evidence"
        );
        require!(
            order.status == "APPROVED",
            "Work order is not approved for completion"
        );
        require!(
            completion_note.trim().len() >= 5 && completion_note.len() <= 500,
            "Completion note must be 5-500 bytes"
        );
        require!(
            valid_sha256(&completion_evidence_hash),
            "Completion evidence must be a SHA-256 hex digest"
        );
        require!(
            !completion_evidence_uri.trim().is_empty() && completion_evidence_uri.len() <= 512,
            "Completion evidence reference must be 1-512 bytes"
        );
        order.completion_note = Some(completion_note);
        order.completion_evidence_hash = Some(completion_evidence_hash);
        order.completion_evidence_uri = Some(completion_evidence_uri);
        order.completed_at_ns = Some(env::block_timestamp());
        order.status = "COMPLETION_SUBMITTED".to_string();
        self.work_orders.insert(key, order.clone());
        order
    }

    /// Only the current NFT owner can confirm the completed job and trigger the reserve payout.
    #[payable]
    pub fn confirm_work_order_completion(
        &mut self,
        token_id: TokenId,
        work_order_id: String,
    ) -> Promise {
        let key = work_order_key(&token_id, &work_order_id);
        let order = self
            .work_orders
            .get(&key)
            .cloned()
            .unwrap_or_else(|| env::panic_str("Work order not found"));
        require!(
            order.status == "COMPLETION_SUBMITTED" || order.status == "PAYMENT_FAILED",
            "Work order has no completed job awaiting payment"
        );
        let recipient_id = order
            .technician_id
            .clone()
            .unwrap_or_else(|| env::panic_str("Work order has no assigned technician"));
        let amount = order
            .quote_amount_yocto
            .as_ref()
            .and_then(|v| v.parse::<u128>().ok())
            .unwrap_or_else(|| env::panic_str("Work order has no valid approved quote"));
        self.start_work_order_payout(token_id, recipient_id, U128(amount), work_order_id)
    }

    pub fn get_work_order(&self, token_id: TokenId, work_order_id: String) -> Option<WorkOrder> {
        self.work_orders
            .get(&work_order_key(&token_id, &work_order_id))
            .cloned()
    }

    pub fn get_asset_work_orders(&self, token_id: TokenId) -> Vec<WorkOrder> {
        self.work_order_ids
            .get(&token_id)
            .cloned()
            .unwrap_or_default()
            .iter()
            .filter_map(|id| {
                self.work_orders
                    .get(&work_order_key(&token_id, id))
                    .cloned()
            })
            .collect()
    }

    fn start_work_order_payout(
        &mut self,
        token_id: TokenId,
        recipient_id: AccountId,
        amount: U128,
        work_order_id: String,
    ) -> Promise {
        require!(
            env::attached_deposit().as_yoctonear() == MIN_WORK_ORDER_DEPOSIT,
            "Attach exactly the amount returned by get_maintenance_payout_cost"
        );
        require!(
            env::prepaid_gas().as_gas() >= Gas::from_tgas(100).as_gas(),
            "Attach at least 100 Tgas for payout"
        );
        require!(amount.0 > 0, "Payout must be greater than zero");
        let owner_id = self
            .tokens
            .owner_by_id
            .get(&token_id)
            .unwrap_or_else(|| env::panic_str("Asset NFT not found"));
        require!(
            owner_id == env::predecessor_account_id(),
            "Only the current NFT owner can approve this payout"
        );
        let asset = self
            .assets
            .get(&token_id)
            .cloned()
            .unwrap_or_else(|| env::panic_str("Asset account record not found"));
        require!(asset.status == "ACTIVE", "Asset account is not active");
        let work_key = work_order_key(&token_id, &work_order_id);
        let mut order = self
            .work_orders
            .get(&work_key)
            .cloned()
            .unwrap_or_else(|| env::panic_str("Work order not found"));
        require!(
            order.status == "COMPLETION_SUBMITTED" || order.status == "PAYMENT_FAILED",
            "Work order completion must be submitted before payout"
        );
        require!(
            order.technician_id.as_ref() == Some(&recipient_id),
            "Payout recipient must be the assigned technician"
        );
        require!(
            order.quote_amount_yocto.as_deref() == Some(amount.0.to_string().as_str()),
            "Payout must exactly match the approved technician quote"
        );
        require!(
            order.approved_by.is_some(),
            "Technician quote has not been approved"
        );
        require!(
            order.completion_evidence_hash.is_some(),
            "Completion evidence is required before payout"
        );
        require!(
            !self.pending_work_orders.contains(&work_key),
            "Work order payout is already pending"
        );
        require!(
            !self.paid_work_orders.contains(&work_key),
            "Work order payout has already been paid"
        );
        self.pending_work_orders.insert(work_key.clone());
        order.status = "PAYOUT_PENDING".to_string();
        self.work_orders.insert(work_key.clone(), order);

        let payout_args = near_sdk::serde_json::json!({
            "recipient_id": recipient_id,
            "amount": amount,
            "work_order_id": work_order_id
        })
        .to_string()
        .into_bytes();
        let callback_args = near_sdk::serde_json::json!({ "work_key": work_key })
            .to_string()
            .into_bytes();
        let registry_id = env::current_account_id();
        Promise::new(asset.asset_account_id)
            .function_call(
                "release_maintenance".to_string(),
                payout_args,
                NearToken::from_yoctonear(0),
                Gas::from_tgas(30),
            )
            .then(Promise::new(registry_id).function_call(
                "on_payout_result".to_string(),
                callback_args,
                NearToken::from_yoctonear(0),
                Gas::from_tgas(10),
            ))
    }

    pub fn get_maintenance_payout_cost(&self) -> U128 {
        U128(MIN_WORK_ORDER_DEPOSIT)
    }

    /// Only the current NFT owner may approve a payout from the asset's protected maintenance reserve.
    #[payable]
    pub fn approve_maintenance_payout(
        &mut self,
        token_id: TokenId,
        recipient_id: AccountId,
        amount: U128,
        work_order_id: String,
    ) -> Promise {
        self.start_work_order_payout(token_id, recipient_id, amount, work_order_id)
    }

    #[private]
    pub fn on_payout_result(&mut self, work_key: String) -> bool {
        // A failed/invalid downstream callback does not prove the external
        // transfer failed: funds may already have been paid. Keep its replay lock
        // until reconciled rather than allowing a second payment for the same job.
        let result = if env::promise_results_count() == 1 {
            env::promise_result_checked(0, 16)
                .ok()
                .and_then(|bytes| near_sdk::serde_json::from_slice::<bool>(&bytes).ok())
        } else {
            None
        };
        match result {
            Some(true) => {
                self.pending_work_orders.remove(&work_key);
                self.paid_work_orders.insert(work_key.clone());
                if let Some(mut order) = self.work_orders.get(&work_key).cloned() {
                    order.status = "PAID".to_string();
                    order.paid_amount_yocto = order.quote_amount_yocto.clone();
                    order.paid_at_ns = Some(env::block_timestamp());
                    self.work_orders.insert(work_key, order);
                }
                true
            }
            Some(false) => {
                // The child's transfer callback explicitly confirmed reserve restoration.
                self.pending_work_orders.remove(&work_key);
                if let Some(mut order) = self.work_orders.get(&work_key).cloned() {
                    order.status = "PAYMENT_FAILED".to_string();
                    self.work_orders.insert(work_key, order);
                }
                false
            }
            None => {
                env::log_str(
                    "Payout outcome unknown; work order remains locked pending reconciliation",
                );
                if let Some(mut order) = self.work_orders.get(&work_key).cloned() {
                    order.status = "PAYOUT_UNKNOWN".to_string();
                    self.work_orders.insert(work_key, order);
                }
                false
            }
        }
    }

    /// Exact creator-paid amount required for a new asset: child code storage + buffer + registry storage.
    pub fn get_asset_creation_cost(&self) -> U128 {
        U128(required_asset_creation_deposit())
    }

    /// Exact retry deposit: shortfall against this asset's refundable funding,
    /// or one authorization yoctoNEAR if no top-up is necessary.
    pub fn get_asset_retry_cost(&self, token_id: TokenId) -> U128 {
        let record = self
            .assets
            .get(&token_id)
            .expect("Asset account record not found");
        require!(
            record.status == "FAILED",
            "Asset account is not in a failed provisioning state"
        );
        // Older registry versions did not record credit and could refund child
        // funding during mint. Never infer a claim on the shared treasury: the
        // owner can still recover a legacy NFT by funding the full quoted child.
        let credit = provisioning_credits().get(&token_id).copied().unwrap_or(0);
        U128(
            child_account_initial_balance()
                .saturating_sub(credit)
                .max(1),
        )
    }

    /// Retry using only this asset's credit and its exact owner-funded shortfall.
    /// Failed atomic child receipts refund their transfer to the registry.
    #[payable]
    pub fn retry_asset_provisioning(&mut self, token_id: TokenId) -> Promise {
        assert_provisioning_gas();
        let owner_id = self
            .tokens
            .owner_by_id
            .get(&token_id)
            .unwrap_or_else(|| env::panic_str("Asset NFT not found"));
        require!(
            owner_id == env::predecessor_account_id(),
            "Only the current NFT owner can retry asset-account provisioning"
        );

        let mut record = self
            .assets
            .get(&token_id)
            .cloned()
            .unwrap_or_else(|| env::panic_str("Asset account record not found"));
        require!(
            record.status == "FAILED",
            "Asset account is not in a failed provisioning state"
        );

        let deposit = env::attached_deposit().as_yoctonear();
        require!(
            deposit == self.get_asset_retry_cost(token_id.clone()).0,
            "Attach exactly the amount returned by get_asset_retry_cost"
        );
        let credit = provisioning_credits().get(&token_id).copied().unwrap_or(0);
        let target = child_account_initial_balance();
        // Code may grow or shrink after recovery. Preserve any existing surplus in
        // the asset account, and forward every owner-attached yoctoNEAR.
        let funding = credit
            .checked_add(deposit)
            .unwrap_or_else(|| env::panic_str("Retry funding overflow"));
        require!(funding >= target, "Insufficient asset provisioning funds");
        provisioning_credits().insert(token_id.clone(), funding);
        record.status = "PROVISIONING".to_string();
        self.assets.insert(token_id.clone(), record.clone());

        provision_asset_account(
            record.asset_account_id,
            env::current_account_id(),
            token_id,
            record.reserve_bps,
            funding,
        )
    }

    pub fn get_asset(&self, token_id: TokenId) -> Option<AssetRecord> {
        self.assets.get(&token_id).cloned()
    }

    pub fn get_asset_owner(&self, token_id: TokenId) -> Option<AccountId> {
        self.tokens.owner_by_id.get(&token_id)
    }

    /// Append-only ownership history for this operational passport.
    pub fn get_asset_ownership_history(&self, token_id: TokenId) -> Vec<OwnershipEvent> {
        ownership_history()
            .get(&token_id)
            .cloned()
            .unwrap_or_default()
    }

    /// True while a payout is in flight or its result needs reconciliation.
    pub fn is_work_order_pending(&self, token_id: TokenId, work_order_id: String) -> bool {
        self.pending_work_orders
            .contains(&format!("{}:{}", token_id, work_order_id))
    }

    pub fn is_work_order_paid(&self, token_id: TokenId, work_order_id: String) -> bool {
        self.paid_work_orders
            .contains(&format!("{}:{}", token_id, work_order_id))
    }

    pub fn nft_token(&self, token_id: TokenId) -> Option<Token> {
        self.tokens.nft_token(token_id)
    }

    pub fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.clone()
    }

    /// Recommended owner-funded history top-up (0.01 NEAR). It does not buy revenue or reserve funds.
    pub fn get_ownership_storage_cost(&self) -> U128 {
        U128(MIN_WORK_ORDER_DEPOSIT)
    }

    pub fn get_ownership_storage_balance(&self, token_id: TokenId) -> U128 {
        U128(
            ownership_storage_credits()
                .get(&token_id)
                .copied()
                .unwrap_or(0),
        )
    }

    #[payable]
    pub fn fund_ownership_storage(&mut self, token_id: TokenId) -> U128 {
        let owner = self
            .tokens
            .owner_by_id
            .get(&token_id)
            .expect("Asset NFT not found");
        require!(
            owner == env::predecessor_account_id(),
            "Only the current NFT owner can fund passport storage"
        );
        let deposit = env::attached_deposit().as_yoctonear();
        require!(
            deposit >= MIN_WORK_ORDER_DEPOSIT,
            "Attach at least the amount returned by get_ownership_storage_cost"
        );
        let before = env::storage_usage();
        let credit = self.get_ownership_storage_balance(token_id.clone()).0;
        ownership_storage_credits().insert(token_id.clone(), credit);
        let entry_cost = u128::from(env::storage_usage().saturating_sub(before))
            .checked_mul(env::storage_byte_cost().as_yoctonear())
            .expect("Storage cost overflow");
        let updated = credit
            .checked_add(deposit)
            .and_then(|v| v.checked_sub(entry_cost))
            .expect("Passport storage funding overflow or insufficient deposit");
        ownership_storage_credits().insert(token_id, updated);
        U128(updated)
    }

    /// NEP-171 core transfer. The child account is deliberately not transferred or drained:
    /// it remains the same child account and only the registry's NFT owner mapping changes.
    #[payable]
    pub fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        require!(
            memo.as_ref().is_none_or(|value| value.len() <= 256),
            "Transfer memo must be at most 256 bytes"
        );
        let storage_before = env::storage_usage();
        let storage_credit = self.get_ownership_storage_balance(token_id.clone()).0;
        let asset = self
            .assets
            .get(&token_id)
            .cloned()
            .unwrap_or_else(|| env::panic_str("Asset NFT not found"));
        require!(asset.status == "ACTIVE", "Asset account is not active");
        let previous_owner_id = self
            .tokens
            .owner_by_id
            .get(&token_id)
            .unwrap_or_else(|| env::panic_str("Asset NFT not found"));
        let mut history = ownership_history()
            .get(&token_id)
            .cloned()
            .unwrap_or_default();
        self.tokens.nft_transfer(
            receiver_id.clone(),
            token_id.clone(),
            approval_id,
            memo.clone(),
        );
        history.push(OwnershipEvent {
            owner_id: receiver_id,
            previous_owner_id: Some(previous_owner_id),
            timestamp_ns: env::block_timestamp(),
            memo,
        });
        ownership_history().insert(token_id.clone(), history);
        let growth = u128::from(env::storage_usage().saturating_sub(storage_before));
        let cost = growth
            .checked_mul(env::storage_byte_cost().as_yoctonear())
            .expect("Ownership history storage cost overflow");
        require!(
            storage_credit >= cost,
            "Passport history storage needs an owner-funded top-up before transfer"
        );
        ownership_storage_credits().insert(token_id, storage_credit - cost);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{VMContextBuilder, accounts};
    use near_sdk::testing_env;

    fn set_context(predecessor: AccountId, current: AccountId, deposit: NearToken) {
        let mut context = VMContextBuilder::new();
        context
            .current_account_id(current)
            .signer_account_id(predecessor.clone())
            .predecessor_account_id(predecessor)
            .attached_deposit(deposit)
            .prepaid_gas(Gas::from_tgas(100));
        testing_env!(context.build());
    }

    fn failed_asset_contract(owner: AccountId) -> Contract {
        let registry: AccountId = "registry.near".parse().unwrap();
        set_context(
            owner.clone(),
            registry.clone(),
            NearToken::from_millinear(100),
        );
        let mut contract = Contract::new();
        let token_id = "asset-01".to_string();
        contract.tokens.internal_mint(
            token_id.clone(),
            owner.clone(),
            Some(TokenMetadata {
                title: Some("Test asset".to_string()),
                description: Some("Test asset passport".to_string()),
                media: None,
                media_hash: None,
                copies: Some(1),
                issued_at: None,
                expires_at: None,
                starts_at: None,
                updated_at: None,
                extra: None,
                reference: None,
                reference_hash: None,
            }),
        );
        ownership_history().insert(
            token_id.clone(),
            vec![OwnershipEvent {
                owner_id: owner.clone(),
                previous_owner_id: None,
                timestamp_ns: 0,
                memo: Some("Initial mint".to_string()),
            }],
        );
        contract.assets.insert(
            token_id.clone(),
            AssetRecord {
                token_id,
                name: "Test asset".to_string(),
                asset_type: "test".to_string(),
                asset_account_id: "asset-01.registry.near".parse().unwrap(),
                reserve_bps: 2_000,
                created_by: owner,
                created_at_ns: 0,
                status: "FAILED".to_string(),
            },
        );
        provisioning_credits().insert("asset-01".to_string(), child_account_initial_balance());
        ownership_storage_credits().insert("asset-01".to_string(), MIN_NFT_STORAGE_DEPOSIT);
        contract
    }

    #[test]
    fn creation_cost_covers_embedded_code_storage_and_both_buffers() {
        let registry: AccountId = "registry.near".parse().unwrap();
        set_context(accounts(1), registry, NearToken::from_yoctonear(1));
        let code_storage_cost =
            (ASSET_ACCOUNT_CODE.len() as u128) * env::storage_byte_cost().as_yoctonear();
        assert_eq!(
            child_account_initial_balance(),
            code_storage_cost + ASSET_ACCOUNT_OPERATIONAL_BUFFER
        );
        assert_eq!(
            required_asset_creation_deposit(),
            code_storage_cost + ASSET_ACCOUNT_OPERATIONAL_BUFFER + MIN_NFT_STORAGE_DEPOSIT
        );
    }

    #[test]
    fn current_owner_can_retry_failed_provisioning() {
        let owner = accounts(1);
        let mut contract = failed_asset_contract(owner.clone());
        set_context(
            owner,
            "registry.near".parse().unwrap(),
            NearToken::from_yoctonear(1),
        );

        let _promise = contract.retry_asset_provisioning("asset-01".to_string());
        assert_eq!(
            contract.get_asset("asset-01".to_string()).unwrap().status,
            "PROVISIONING"
        );
    }

    #[test]
    #[should_panic(expected = "Only the current NFT owner can retry asset-account provisioning")]
    fn non_owner_cannot_retry_failed_provisioning() {
        let owner = accounts(1);
        let mut contract = failed_asset_contract(owner);
        set_context(
            accounts(2),
            "registry.near".parse().unwrap(),
            NearToken::from_yoctonear(1),
        );
        let _promise = contract.retry_asset_provisioning("asset-01".to_string());
    }

    #[test]
    #[should_panic(expected = "Asset account is not in a failed provisioning state")]
    fn active_asset_cannot_be_retried() {
        let owner = accounts(1);
        let mut contract = failed_asset_contract(owner.clone());
        let mut record = contract
            .assets
            .get(&"asset-01".to_string())
            .cloned()
            .unwrap();
        record.status = "ACTIVE".to_string();
        contract.assets.insert("asset-01".to_string(), record);
        set_context(
            owner,
            "registry.near".parse().unwrap(),
            NearToken::from_yoctonear(1),
        );
        let _promise = contract.retry_asset_provisioning("asset-01".to_string());
    }
    fn provisioning_result(result: near_sdk::PromiseResult) -> Contract {
        let mut contract = failed_asset_contract(accounts(1));
        let id = "asset-01".to_string();
        contract.assets.get_mut(&id).unwrap().status = "PROVISIONING".to_string();
        contract.assets.flush();
        let mut ctx = VMContextBuilder::new();
        ctx.current_account_id("registry.near".parse().unwrap())
            .predecessor_account_id("registry.near".parse().unwrap());
        testing_env!(
            ctx.build(),
            near_sdk::test_vm_config(),
            near_sdk::RuntimeFeesConfig::test(),
            std::collections::HashMap::default(),
            vec![result]
        );
        contract
    }

    #[test]
    fn empty_success_activates_and_consumes_credit() {
        let mut c = provisioning_result(near_sdk::PromiseResult::Successful(vec![]));
        assert!(c.on_asset_created("asset-01".into()));
        assert_eq!(c.get_asset("asset-01".into()).unwrap().status, "ACTIVE");
        assert!(
            provisioning_credits()
                .get(&"asset-01".to_string())
                .is_none()
        );
    }

    #[test]
    fn definite_failure_retains_credit_for_retry() {
        let mut c = provisioning_result(near_sdk::PromiseResult::Failed);
        assert!(!c.on_asset_created("asset-01".into()));
        assert_eq!(c.get_asset_retry_cost("asset-01".into()).0, 1);
    }

    #[test]
    fn unexpected_output_is_unknown_and_not_retryable() {
        let mut c = provisioning_result(near_sdk::PromiseResult::Successful(vec![0; 1024]));
        assert!(!c.on_asset_created("asset-01".into()));
        assert_eq!(
            c.get_asset("asset-01".into()).unwrap().status,
            "PROVISIONING_UNKNOWN"
        );
        assert!(
            provisioning_credits()
                .get(&"asset-01".to_string())
                .is_none()
        );
    }

    #[test]
    fn retry_quote_and_transfer_use_asset_credit_plus_exact_topup() {
        use near_sdk::mock::MockAction;
        let mut c = failed_asset_contract(accounts(1));
        let id = "asset-01".to_string();
        let target = child_account_initial_balance();
        provisioning_credits().insert(id.clone(), target - 123);
        assert_eq!(c.get_asset_retry_cost(id.clone()).0, 123);
        set_context(
            accounts(1),
            "registry.near".parse().unwrap(),
            NearToken::from_yoctonear(123),
        );
        drop(c.retry_asset_provisioning(id.clone()));
        assert_eq!(*provisioning_credits().get(&id).unwrap(), target);
        assert!(near_sdk::test_utils::get_created_receipts().iter().flat_map(|r| &r.actions)
            .any(|a| matches!(a, MockAction::Transfer { deposit, .. } if deposit.as_yoctonear() == target)));
    }

    #[test]
    #[should_panic(expected = "Attach exactly the amount returned by get_asset_retry_cost")]
    fn retry_rejects_insufficient_topup() {
        let mut c = failed_asset_contract(accounts(1));
        provisioning_credits().insert("asset-01".into(), child_account_initial_balance() - 10);
        set_context(
            accounts(1),
            "registry.near".parse().unwrap(),
            NearToken::from_yoctonear(1),
        );
        drop(c.retry_asset_provisioning("asset-01".into()));
    }

    #[test]
    fn legacy_missing_credit_requires_full_owner_funding() {
        let mut c = failed_asset_contract(accounts(1));
        let id = "asset-01".to_string();
        provisioning_credits().remove(&id);
        let required = child_account_initial_balance();
        assert_eq!(c.get_asset_retry_cost(id.clone()).0, required);
        set_context(
            accounts(1),
            "registry.near".parse().unwrap(),
            NearToken::from_yoctonear(required),
        );
        drop(c.retry_asset_provisioning(id.clone()));
        assert_eq!(*provisioning_credits().get(&id).unwrap(), required);
        assert_eq!(c.get_asset(id).unwrap().status, "PROVISIONING");
    }

    fn assert_uncertain_payout_keeps_lock(result: near_sdk::PromiseResult) {
        let mut c = provisioning_result(result);
        c.pending_work_orders.insert("asset-01:WO".into());
        assert!(!c.on_payout_result("asset-01:WO".into()));
        assert!(c.is_work_order_pending("asset-01".into(), "WO".into()));
        assert!(!c.is_work_order_paid("asset-01".into(), "WO".into()));
    }

    #[test]
    fn failed_payout_receipt_keeps_replay_lock() {
        assert_uncertain_payout_keeps_lock(near_sdk::PromiseResult::Failed);
    }

    #[test]
    fn malformed_payout_result_keeps_replay_lock() {
        assert_uncertain_payout_keeps_lock(near_sdk::PromiseResult::Successful(
            b"invalid".to_vec(),
        ));
    }

    #[test]
    fn oversized_payout_result_keeps_replay_lock() {
        assert_uncertain_payout_keeps_lock(near_sdk::PromiseResult::Successful(vec![0; 32]));
    }

    #[test]
    fn confirmed_payout_sets_paid_and_releases_pending_lock() {
        let mut c = provisioning_result(near_sdk::PromiseResult::Successful(b"true".to_vec()));
        c.pending_work_orders.insert("asset-01:WO".into());
        assert!(c.on_payout_result("asset-01:WO".into()));
        assert!(!c.is_work_order_pending("asset-01".into(), "WO".into()));
        assert!(c.is_work_order_paid("asset-01".into(), "WO".into()));
    }

    #[test]
    fn confirmed_restored_payout_can_be_retried() {
        let mut c = provisioning_result(near_sdk::PromiseResult::Successful(b"false".to_vec()));
        c.pending_work_orders.insert("asset-01:WO".into());
        assert!(!c.on_payout_result("asset-01:WO".into()));
        assert!(!c.is_work_order_pending("asset-01".into(), "WO".into()));
        assert!(!c.is_work_order_paid("asset-01".into(), "WO".into()));
    }
    #[test]
    fn nft_handover_during_pending_payout_preserves_asset_and_settles_work_order() {
        let owner = accounts(1);
        let technician = accounts(2);
        let buyer = accounts(3);
        let registry: AccountId = "registry.near".parse().unwrap();
        let token_id = "asset-01".to_string();
        let work_order_id = "WO-MID".to_string();
        let amount = NearToken::from_millinear(100).as_yoctonear();

        let mut contract = failed_asset_contract(owner.clone());
        let mut asset = contract.get_asset(token_id.clone()).unwrap();
        asset.status = "ACTIVE".to_string();
        let child_account_id = asset.asset_account_id.clone();
        contract.assets.insert(token_id.clone(), asset);

        contract
            .work_order_ids
            .insert(token_id.clone(), vec![work_order_id.clone()]);
        contract.work_orders.insert(
            work_order_key(&token_id, &work_order_id),
            WorkOrder {
                work_order_id: work_order_id.clone(),
                token_id: token_id.clone(),
                observation: "Pump stopped delivering water".to_string(),
                observation_evidence_hash: None,
                diagnosis: "Pump or delivery-path interruption".to_string(),
                diagnosis_source: "test-ruleset".to_string(),
                recommended_action: "Inspect pump and delivery line".to_string(),
                priority: "HIGH".to_string(),
                created_by: owner.clone(),
                created_at_ns: 1,
                status: "COMPLETION_SUBMITTED".to_string(),
                technician_id: Some(technician.clone()),
                quote_amount_yocto: Some(amount.to_string()),
                quote_note: Some("0.1 NEAR test quote".to_string()),
                quoted_at_ns: Some(2),
                approved_by: Some(owner.clone()),
                approved_at_ns: Some(3),
                completion_note: Some("Simulated completion report".to_string()),
                completion_evidence_hash: Some("a".repeat(64)),
                completion_evidence_uri: Some("demo://completion-fixture".to_string()),
                completed_at_ns: Some(4),
                paid_amount_yocto: None,
                paid_at_ns: None,
            },
        );

        // The owner starts the payout. The registry records PAYOUT_PENDING before
        // the child-account promise and callback execute.
        set_context(
            owner.clone(),
            registry.clone(),
            NearToken::from_yoctonear(MIN_WORK_ORDER_DEPOSIT),
        );
        let _payout =
            contract.confirm_work_order_completion(token_id.clone(), work_order_id.clone());
        assert!(contract.is_work_order_pending(token_id.clone(), work_order_id.clone()));
        assert_eq!(
            contract
                .get_work_order(token_id.clone(), work_order_id.clone())
                .unwrap()
                .status,
            "PAYOUT_PENDING"
        );

        // The NFT is sold while the payout promise is in flight.
        set_context(
            owner.clone(),
            registry.clone(),
            NearToken::from_yoctonear(1),
        );
        contract.nft_transfer(
            buyer.clone(),
            token_id.clone(),
            None,
            Some("Sale during pending payout".to_string()),
        );
        assert_eq!(
            contract.get_asset_owner(token_id.clone()),
            Some(buyer.clone())
        );
        assert_eq!(
            contract
                .get_asset(token_id.clone())
                .unwrap()
                .asset_account_id,
            child_account_id
        );
        assert!(contract.is_work_order_pending(token_id.clone(), work_order_id.clone()));
        assert_eq!(
            contract
                .get_work_order(token_id.clone(), work_order_id.clone())
                .unwrap()
                .status,
            "PAYOUT_PENDING"
        );

        let history = contract.get_asset_ownership_history(token_id.clone());
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].owner_id, owner);
        assert_eq!(history[0].previous_owner_id, None);
        assert_eq!(history[1].previous_owner_id, Some(accounts(1)));
        assert_eq!(history[1].owner_id, buyer);

        // The already-authorized payout callback settles against the same token ID
        // after ownership changes; it cannot be redirected or replayed by the seller.
        let mut callback_context = VMContextBuilder::new();
        callback_context
            .current_account_id(registry.clone())
            .signer_account_id(registry.clone())
            .predecessor_account_id(registry);
        testing_env!(
            callback_context.build(),
            near_sdk::test_vm_config(),
            near_sdk::RuntimeFeesConfig::test(),
            std::collections::HashMap::default(),
            vec![near_sdk::PromiseResult::Successful(b"true".to_vec())]
        );
        assert!(contract.on_payout_result(work_order_key(&token_id, &work_order_id)));
        let final_order = contract
            .get_work_order(token_id.clone(), work_order_id.clone())
            .unwrap();
        assert_eq!(final_order.status, "PAID");
        assert_eq!(final_order.paid_amount_yocto, Some(amount.to_string()));
        assert_eq!(final_order.technician_id, Some(technician));
        assert!(contract.is_work_order_paid(token_id.clone(), work_order_id.clone()));
        assert!(!contract.is_work_order_pending(token_id.clone(), work_order_id));
        assert_eq!(contract.get_asset_owner(token_id.clone()), Some(buyer));
        assert_eq!(contract.get_asset_ownership_history(token_id).len(), 2);
    }
    #[test]
    fn history_rent_is_charged_to_the_passport_and_account_identity_is_retained() {
        let owner = accounts(1);
        let mut c = failed_asset_contract(owner.clone());
        let id = "asset-01".to_string();
        c.assets.get_mut(&id).unwrap().status = "ACTIVE".into();
        let child = c.get_asset(id.clone()).unwrap().asset_account_id;
        let before = c.get_ownership_storage_balance(id.clone()).0;
        set_context(
            owner,
            "registry.near".parse().unwrap(),
            NearToken::from_yoctonear(1),
        );
        c.nft_transfer(accounts(2), id.clone(), None, Some("Handover".into()));
        assert!(c.get_ownership_storage_balance(id.clone()).0 < before);
        assert_eq!(c.get_asset(id.clone()).unwrap().asset_account_id, child);
        assert_eq!(c.get_asset_owner(id).unwrap(), accounts(2));
    }

    #[test]
    #[should_panic(expected = "owner-funded top-up")]
    fn exhausted_passport_storage_cannot_spend_shared_treasury() {
        let owner = accounts(1);
        let mut c = failed_asset_contract(owner.clone());
        let id = "asset-01".to_string();
        c.assets.get_mut(&id).unwrap().status = "ACTIVE".into();
        ownership_storage_credits().insert(id.clone(), 0);
        set_context(
            owner,
            "registry.near".parse().unwrap(),
            NearToken::from_yoctonear(1),
        );
        c.nft_transfer(accounts(2), id, None, Some("Handover".into()));
    }

    #[test]
    fn owner_can_refill_passport_history_without_touching_child_balances() {
        let owner = accounts(1);
        let mut c = failed_asset_contract(owner.clone());
        let id = "asset-01".to_string();
        let before = c.get_ownership_storage_balance(id.clone()).0;
        set_context(
            owner,
            "registry.near".parse().unwrap(),
            NearToken::from_millinear(10),
        );
        let after = c.fund_ownership_storage(id.clone()).0;
        assert_eq!(after, before + NearToken::from_millinear(10).as_yoctonear());
        assert_eq!(c.get_asset_owner(id).unwrap(), accounts(1));
    }

    #[test]
    #[should_panic(expected = "Only the current NFT owner")]
    fn outsider_cannot_control_passport_storage_funding() {
        let mut c = failed_asset_contract(accounts(1));
        set_context(
            accounts(2),
            "registry.near".parse().unwrap(),
            NearToken::from_millinear(10),
        );
        c.fund_ownership_storage("asset-01".into());
    }
}
