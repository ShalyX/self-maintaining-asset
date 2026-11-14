use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk::{AccountId, Gas, NearToken, PanicOnDefault, Promise, env, near, require};

const BPS_DENOMINATOR: u128 = 10_000;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    registry_id: AccountId,
    token_id: String,
    reserve_bps: u16,
    operating_balance: u128,
    maintenance_reserve: u128,
    total_deposited: u128,
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct AssetBalances {
    pub token_id: String,
    pub registry_id: AccountId,
    pub reserve_bps: u16,
    pub total_deposited: U128,
    pub operating_balance: U128,
    pub maintenance_reserve: U128,
    pub actual_account_balance: U128,
}

#[near]
impl Contract {
    /// Initializes a keyless asset sub-account. Only its parent registry may initialize it.
    #[init]
    pub fn new_vault(registry_id: AccountId, token_id: String, reserve_bps: u16) -> Self {
        require!(
            env::predecessor_account_id() == registry_id,
            "Only the asset registry can initialize this account"
        );
        require!(
            reserve_bps <= 10_000,
            "Reserve percentage cannot exceed 100%"
        );
        let parent_suffix = format!(".{}", registry_id);
        require!(
            env::current_account_id().as_str().ends_with(&parent_suffix),
            "Asset account must be a sub-account of its registry"
        );
        Self {
            registry_id,
            token_id,
            reserve_bps,
            operating_balance: 0,
            maintenance_reserve: 0,
            total_deposited: 0,
        }
    }

    /// Anyone may fund the asset. Deposits are split deterministically by contract rules.
    #[payable]
    pub fn deposit_revenue(&mut self) -> AssetBalances {
        let amount = env::attached_deposit().as_yoctonear();
        require!(amount > 0, "Deposit must be greater than zero");

        let weighted = amount
            .checked_mul(self.reserve_bps as u128)
            .unwrap_or_else(|| env::panic_str("Deposit split overflow"));
        let reserve = weighted / BPS_DENOMINATOR;
        let operating = amount - reserve;

        self.total_deposited = self
            .total_deposited
            .checked_add(amount)
            .unwrap_or_else(|| env::panic_str("Total deposit overflow"));
        self.operating_balance = self
            .operating_balance
            .checked_add(operating)
            .unwrap_or_else(|| env::panic_str("Operating balance overflow"));
        self.maintenance_reserve = self
            .maintenance_reserve
            .checked_add(reserve)
            .unwrap_or_else(|| env::panic_str("Maintenance reserve overflow"));

        env::log_str(
            &json!({
                "event": "asset_revenue_deposited",
                "token_id": self.token_id.clone(),
                "amount_yocto": amount.to_string(),
                "operating_yocto": operating.to_string(),
                "maintenance_reserve_yocto": reserve.to_string()
            })
            .to_string(),
        );
        self.get_balances()
    }

    /// Only the NFT registry can release maintenance funds.
    /// The registry must independently verify the current NFT owner and approved work order.
    pub fn release_maintenance(
        &mut self,
        recipient_id: AccountId,
        amount: U128,
        work_order_id: String,
    ) -> Promise {
        require!(
            env::predecessor_account_id() == self.registry_id,
            "Only the asset registry may authorize maintenance payouts"
        );
        require!(amount.0 > 0, "Payout must be greater than zero");
        require!(
            self.maintenance_reserve >= amount.0,
            "Insufficient maintenance reserve"
        );

        // Deduct before the external transfer. The private callback restores the reserve on failure.
        self.maintenance_reserve -= amount.0;

        let callback_args = json!({
            "amount": amount,
            "work_order_id": work_order_id
        })
        .to_string()
        .into_bytes();

        Promise::new(recipient_id)
            .transfer(NearToken::from_yoctonear(amount.0))
            .then(Promise::new(env::current_account_id()).function_call(
                "on_maintenance_transfer".to_string(),
                callback_args,
                NearToken::from_yoctonear(0),
                Gas::from_tgas(10),
            ))
    }

    /// Private callback: failed transfers restore the protected reserve.
    #[private]
    pub fn on_maintenance_transfer(&mut self, amount: U128, work_order_id: String) -> bool {
        match env::promise_result_checked(0, 0) {
            Ok(_) => {
                env::log_str(
                    &json!({
                        "event": "asset_maintenance_paid",
                        "token_id": self.token_id.clone(),
                        "work_order_id": work_order_id,
                        "amount_yocto": amount.0.to_string()
                    })
                    .to_string(),
                );
                true
            }
            Err(_) => {
                self.maintenance_reserve = self
                    .maintenance_reserve
                    .checked_add(amount.0)
                    .unwrap_or_else(|| env::panic_str("Reserve restoration overflow"));
                env::log_str(
                    &json!({
                        "event": "asset_maintenance_payment_failed",
                        "token_id": self.token_id.clone(),
                        "work_order_id": work_order_id,
                        "restored_yocto": amount.0.to_string()
                    })
                    .to_string(),
                );
                false
            }
        }
    }

    pub fn get_balances(&self) -> AssetBalances {
        AssetBalances {
            token_id: self.token_id.clone(),
            registry_id: self.registry_id.clone(),
            reserve_bps: self.reserve_bps,
            total_deposited: U128(self.total_deposited),
            operating_balance: U128(self.operating_balance),
            maintenance_reserve: U128(self.maintenance_reserve),
            actual_account_balance: U128(env::account_balance().as_yoctonear()),
        }
    }

    pub fn get_registry_id(&self) -> AccountId {
        self.registry_id.clone()
    }

    pub fn get_token_id(&self) -> String {
        self.token_id.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{VMContextBuilder, accounts};
    use near_sdk::testing_env;

    fn context(predecessor: AccountId, current: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(current)
            .signer_account_id(predecessor.clone())
            .predecessor_account_id(predecessor);
        builder
    }

    fn initialized_vault() -> Contract {
        let registry: AccountId = "registry.near".parse().unwrap();
        let vault: AccountId = "asset-01.registry.near".parse().unwrap();
        let ctx = context(registry.clone(), vault);
        testing_env!(ctx.build());
        Contract::new_vault(registry, "asset-01".to_string(), 2_000)
    }

    #[test]
    fn revenue_is_split_into_operating_and_maintenance_balances() {
        let mut contract = initialized_vault();
        let mut ctx = context(accounts(1), "asset-01.registry.near".parse().unwrap());
        ctx.attached_deposit(NearToken::from_near(1));
        testing_env!(ctx.build());

        let balances = contract.deposit_revenue();
        assert_eq!(
            balances.total_deposited.0,
            NearToken::from_near(1).as_yoctonear()
        );
        assert_eq!(
            balances.operating_balance.0,
            NearToken::from_millinear(800).as_yoctonear()
        );
        assert_eq!(
            balances.maintenance_reserve.0,
            NearToken::from_millinear(200).as_yoctonear()
        );
    }

    #[test]
    #[should_panic(expected = "Only the asset registry may authorize maintenance payouts")]
    fn non_registry_cannot_release_reserve() {
        let mut contract = initialized_vault();
        let mut ctx = context(accounts(1), "asset-01.registry.near".parse().unwrap());
        ctx.attached_deposit(NearToken::from_near(1));
        testing_env!(ctx.build());
        contract.deposit_revenue();

        let ctx = context(accounts(1), "asset-01.registry.near".parse().unwrap());
        testing_env!(ctx.build());
        let _ = contract.release_maintenance(
            accounts(2),
            U128(NearToken::from_millinear(10).as_yoctonear()),
            "WO-1".to_string(),
        );
    }

    #[test]
    #[should_panic(expected = "Insufficient maintenance reserve")]
    fn reserve_cannot_be_overdrawn() {
        let mut contract = initialized_vault();
        let ctx = context(
            "registry.near".parse().unwrap(),
            "asset-01.registry.near".parse().unwrap(),
        );
        testing_env!(ctx.build());
        let _ = contract.release_maintenance(
            accounts(2),
            U128(NearToken::from_near(1).as_yoctonear()),
            "WO-2".to_string(),
        );
    }

    fn payout_callback_context(result: near_sdk::PromiseResult) {
        let current: AccountId = "asset-01.registry.near".parse().unwrap();
        let mut ctx = VMContextBuilder::new();
        ctx.current_account_id(current.clone())
            .predecessor_account_id(current);
        testing_env!(
            ctx.build(),
            near_sdk::test_vm_config(),
            near_sdk::RuntimeFeesConfig::test(),
            std::collections::HashMap::default(),
            vec![result]
        );
    }

    fn reserve_after_authorized_release() -> Contract {
        let mut contract = initialized_vault();
        let mut ctx = context(accounts(1), "asset-01.registry.near".parse().unwrap());
        ctx.attached_deposit(NearToken::from_near(1));
        testing_env!(ctx.build());
        contract.deposit_revenue();

        let ctx = context(
            "registry.near".parse().unwrap(),
            "asset-01.registry.near".parse().unwrap(),
        );
        testing_env!(ctx.build());
        let _promise = contract.release_maintenance(
            accounts(2),
            U128(NearToken::from_millinear(100).as_yoctonear()),
            "WO-CALLBACK".to_string(),
        );
        contract
    }

    #[test]
    fn failed_transfer_callback_restores_the_reserved_amount() {
        let mut contract = reserve_after_authorized_release();
        assert_eq!(
            contract.get_balances().maintenance_reserve.0,
            NearToken::from_millinear(100).as_yoctonear()
        );
        payout_callback_context(near_sdk::PromiseResult::Failed);
        assert!(!contract.on_maintenance_transfer(
            U128(NearToken::from_millinear(100).as_yoctonear()),
            "WO-CALLBACK".to_string()
        ));
        assert_eq!(
            contract.get_balances().maintenance_reserve.0,
            NearToken::from_millinear(200).as_yoctonear()
        );
    }

    #[test]
    fn successful_transfer_callback_keeps_the_reserve_deducted() {
        let mut contract = reserve_after_authorized_release();
        payout_callback_context(near_sdk::PromiseResult::Successful(vec![]));
        assert!(contract.on_maintenance_transfer(
            U128(NearToken::from_millinear(100).as_yoctonear()),
            "WO-CALLBACK".to_string()
        ));
        assert_eq!(
            contract.get_balances().maintenance_reserve.0,
            NearToken::from_millinear(100).as_yoctonear()
        );
    }

    #[test]
    fn registry_can_authorize_a_maintenance_payment_from_reserve() {
        let mut contract = initialized_vault();
        let mut ctx = context(accounts(1), "asset-01.registry.near".parse().unwrap());
        ctx.attached_deposit(NearToken::from_near(1));
        testing_env!(ctx.build());
        contract.deposit_revenue();

        let ctx = context(
            "registry.near".parse().unwrap(),
            "asset-01.registry.near".parse().unwrap(),
        );
        testing_env!(ctx.build());
        let _promise = contract.release_maintenance(
            accounts(2),
            U128(NearToken::from_millinear(100).as_yoctonear()),
            "WO-3".to_string(),
        );
        assert_eq!(
            contract.get_balances().maintenance_reserve.0,
            NearToken::from_millinear(100).as_yoctonear()
        );
    }
}
