use near_api::{Account, AccountId as ApiAccountId, NearGas, NearToken};
use near_sdk::{
    AccountId as SandboxAccountId,
    json_types::U128,
    serde_json::{Value, json},
};

async fn start_sandbox() -> testresult::TestResult<near_sandbox::Sandbox> {
    let config = near_sandbox::SandboxConfig {
        additional_genesis: Some(json!({"protocol_version": 85, "epoch_length": 1_000_000})),
        ..Default::default()
    };
    let sandbox = near_sandbox::Sandbox::start_sandbox_with_config(config).await?;
    let genesis: Value = near_sdk::serde_json::from_slice(&std::fs::read(
        sandbox.home_dir.path().join("genesis.json"),
    )?)?;
    assert_eq!(genesis["protocol_version"], 85);
    Ok(sandbox)
}

async fn create_account(
    sandbox: &near_sandbox::Sandbox,
    name: &str,
) -> testresult::TestResult<Account> {
    let sandbox_account_id: SandboxAccountId = name.parse().unwrap();
    sandbox
        .create_account(sandbox_account_id)
        .initial_balance(NearToken::from_near(10))
        .send()
        .await?;
    let account_id: ApiAccountId = name.parse().unwrap();
    Ok(Account(account_id))
}

#[tokio::test]
async fn asset_account_balance_and_authority_survive_nft_transfer() -> testresult::TestResult<()> {
    let sandbox = start_sandbox().await?;
    let network = near_api::NetworkConfig::from_rpc_url("sandbox", sandbox.rpc_addr.parse()?);

    let alice = create_account(&sandbox, "alice.sandbox").await?;
    let bob = create_account(&sandbox, "bob.sandbox").await?;
    let technician = create_account(&sandbox, "tech.sandbox").await?;
    let registry = create_account(&sandbox, "registry.sandbox")
        .await?
        .as_contract();

    let signer = near_api::Signer::from_secret_key(
        near_sandbox::config::DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY
            .parse()
            .unwrap(),
    )?;

    // Deploy the NFT registry. It embeds the independently built asset-account WASM.
    let registry_wasm_path = std::env::var_os("ASSET_REGISTRY_WASM_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target/near/asset_registry.wasm")
        });
    let registry_wasm = std::fs::read(registry_wasm_path)?;
    near_api::Contract::deploy(registry.account_id().clone())
        .use_code(registry_wasm)
        .with_init_call("new", json!({}))?
        .with_signer(signer.clone())
        .send_to(&network)
        .await?
        .assert_success();

    let creation_cost: U128 = registry
        .call_function("get_asset_creation_cost", json!({}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;

    let treasury_before = Account(registry.account_id().clone())
        .tokens()
        .near_balance()
        .fetch_from(&network)
        .await?
        .total
        .as_yoctonear();
    // Alice pays the exact creator-funded provisioning cost; the shared registry treasury is not subsidized.
    registry
        .call_function(
            "create_asset",
            json!({
                "token_id": "borehole-01",
                "name": "Solar Borehole 01",
                "asset_type": "solar_borehole",
                "reserve_bps": 2000
            }),
        )
        .transaction()
        .deposit(NearToken::from_yoctonear(creation_cost.0))
        .gas(NearGas::from_tgas(100))
        .with_signer(alice.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?
        .assert_success();

    let asset: Value = registry
        .call_function("get_asset", json!({"token_id": "borehole-01"}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(asset["status"], "ACTIVE");
    let child_id: ApiAccountId = asset["asset_account_id"].as_str().unwrap().parse()?;
    assert_eq!(
        child_id.as_str(),
        format!("borehole-01.{}", registry.account_id())
    );
    let child = Account(child_id.clone()).as_contract();

    // Create a real persistent work order before handover.
    let work_order_storage_cost: U128 = registry
        .call_function("get_work_order_creation_cost", json!({}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    registry
        .call_function(
            "create_work_order",
            json!({
                "token_id": "borehole-01",
                "work_order_id": "WO-001",
                "observation": "Pump stopped delivering water after the controller showed a fault.",
                "observation_evidence_hash": null,
                "diagnosis": "Pump or delivery-path interruption",
                "diagnosis_source": "transparent-ruleset-v1",
                "recommended_action": "Isolate power, inspect controller protection trips and verify the delivery line.",
                "priority": "HIGH"
            }),
        )
        .transaction()
        .deposit(NearToken::from_yoctonear(work_order_storage_cost.0))
        .gas(NearGas::from_tgas(100))
        .with_signer(alice.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?
        .assert_success();

    registry
        .call_function(
            "assign_technician",
            json!({
                "token_id": "borehole-01",
                "work_order_id": "WO-001",
                "technician_id": technician.account_id()
            }),
        )
        .transaction()
        .with_signer(alice.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?
        .assert_success();

    registry
        .call_function(
            "submit_technician_quote",
            json!({
                "token_id": "borehole-01",
                "work_order_id": "WO-001",
                "quote_amount": NearToken::from_millinear(100).as_yoctonear().to_string(),
                "quote_note": "Replace pump seal, inspect coupling and run a delivery test."
            }),
        )
        .transaction()
        .with_signer(technician.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?
        .assert_success();

    registry
        .call_function(
            "approve_work_order_quote",
            json!({"token_id": "borehole-01", "work_order_id": "WO-001"}),
        )
        .transaction()
        .with_signer(alice.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?
        .assert_success();

    registry
        .call_function(
            "submit_work_order_completion",
            json!({
                "token_id": "borehole-01",
                "work_order_id": "WO-001",
                "completion_note": "Pump seal replaced; coupling inspected and delivery test passed.",
                "completion_evidence_hash": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "completion_evidence_uri": "browser-local://sha256/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            }),
        )
        .transaction()
        .with_signer(technician.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?
        .assert_success();

    let submitted_order: Value = registry
        .call_function(
            "get_work_order",
            json!({"token_id": "borehole-01", "work_order_id": "WO-001"}),
        )
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(submitted_order["status"], "COMPLETION_SUBMITTED");
    assert_eq!(
        submitted_order["technician_id"],
        technician.account_id().as_str()
    );

    let treasury_after = Account(registry.account_id().clone())
        .tokens()
        .near_balance()
        .fetch_from(&network)
        .await?
        .total
        .as_yoctonear();
    assert!(
        treasury_after >= treasury_before,
        "Permissionless creation subsidized from registry treasury"
    );
    assert!(
        treasury_after
            <= treasury_before
                + work_order_storage_cost.0
                + NearToken::from_millinear(110).as_yoctonear()
    );
    // Revenue enters the asset's own account and is split 80/20 by contract rules.
    child
        .call_function("deposit_revenue", json!({}))
        .transaction()
        .deposit(NearToken::from_near(1))
        .with_signer(alice.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?
        .assert_success();

    let before_transfer: Value = child
        .call_function("get_balances", json!({}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(
        before_transfer["maintenance_reserve"],
        NearToken::from_millinear(200).as_yoctonear().to_string()
    );
    assert_eq!(
        before_transfer["operating_balance"],
        NearToken::from_millinear(800).as_yoctonear().to_string()
    );

    // Transfer the NFT. The child account is not transferred, drained, or recreated.
    registry
        .call_function(
            "nft_transfer",
            json!({
                "receiver_id": bob.account_id(),
                "token_id": "borehole-01",
                "approval_id": null,
                "memo": "operator handover"
            }),
        )
        .transaction()
        .deposit(NearToken::from_yoctonear(1))
        .with_signer(alice.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?
        .assert_success();

    let token: Value = registry
        .call_function("nft_token", json!({"token_id": "borehole-01"}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(token["owner_id"], bob.account_id().as_str());

    let asset_after_transfer: Value = registry
        .call_function("get_asset", json!({"token_id": "borehole-01"}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(asset_after_transfer["asset_account_id"], child_id.as_str());

    let after_transfer: Value = child
        .call_function("get_balances", json!({}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(
        after_transfer["maintenance_reserve"],
        before_transfer["maintenance_reserve"]
    );
    assert_eq!(
        after_transfer["operating_balance"],
        before_transfer["operating_balance"]
    );

    // The former owner cannot confirm the completed work after handover.
    let payout_cost: U128 = registry
        .call_function("get_maintenance_payout_cost", json!({}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    registry
        .call_function(
            "confirm_work_order_completion",
            json!({"token_id": "borehole-01", "work_order_id": "WO-001"}),
        )
        .transaction()
        .deposit(NearToken::from_yoctonear(payout_cost.0))
        .gas(NearGas::from_tgas(100))
        .with_signer(alice.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?
        .assert_failure();

    // The current owner confirms the completed repair. The exact approved quote
    // is paid to the assigned technician from the asset's protected reserve.
    let technician_before = technician
        .tokens()
        .near_balance()
        .fetch_from(&network)
        .await?
        .total;
    registry
        .call_function(
            "confirm_work_order_completion",
            json!({"token_id": "borehole-01", "work_order_id": "WO-001"}),
        )
        .transaction()
        .deposit(NearToken::from_yoctonear(payout_cost.0))
        .gas(NearGas::from_tgas(100))
        .with_signer(bob.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?
        .assert_success();

    let technician_after = technician
        .tokens()
        .near_balance()
        .fetch_from(&network)
        .await?
        .total;
    assert_eq!(
        technician_after.as_yoctonear(),
        technician_before.as_yoctonear() + NearToken::from_millinear(100).as_yoctonear()
    );

    let after_payout: Value = child
        .call_function("get_balances", json!({}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(
        after_payout["maintenance_reserve"],
        NearToken::from_millinear(100).as_yoctonear().to_string()
    );

    let paid_order: Value = registry
        .call_function(
            "get_work_order",
            json!({"token_id": "borehole-01", "work_order_id": "WO-001"}),
        )
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(paid_order["status"], "PAID");
    assert_eq!(
        paid_order["paid_amount_yocto"],
        NearToken::from_millinear(100).as_yoctonear().to_string()
    );

    let paid: bool = registry
        .call_function(
            "is_work_order_paid",
            json!({"token_id": "borehole-01", "work_order_id": "WO-001"}),
        )
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert!(paid);

    // A completed and paid work order cannot be replayed for a second payout.
    registry
        .call_function(
            "confirm_work_order_completion",
            json!({"token_id": "borehole-01", "work_order_id": "WO-001"}),
        )
        .transaction()
        .deposit(NearToken::from_yoctonear(payout_cost.0))
        .gas(NearGas::from_tgas(100))
        .with_signer(bob.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?
        .assert_failure();

    // The child account rejects direct payout attempts from a user; only the registry can call it.
    child
        .call_function(
            "release_maintenance",
            json!({
                "recipient_id": technician.account_id(),
                "amount": NearToken::from_millinear(10).as_yoctonear().to_string(),
                "work_order_id": "WO-UNAUTHORIZED"
            }),
        )
        .transaction()
        .with_signer(alice.account_id().clone(), signer)
        .send_to(&network)
        .await?
        .assert_failure();

    Ok(())
}

#[tokio::test]
async fn underfunded_asset_creation_is_rejected_before_mint_and_can_be_retried()
-> testresult::TestResult<()> {
    let sandbox = start_sandbox().await?;
    let network = near_api::NetworkConfig::from_rpc_url("sandbox", sandbox.rpc_addr.parse()?);

    let alice = create_account(&sandbox, "alice.sandbox").await?;
    let registry = create_account(&sandbox, "registry.sandbox")
        .await?
        .as_contract();
    let signer = near_api::Signer::from_secret_key(
        near_sandbox::config::DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY
            .parse()
            .unwrap(),
    )?;

    let registry_wasm_path = std::env::var_os("ASSET_REGISTRY_WASM_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target/near/asset_registry.wasm")
        });
    let registry_wasm = std::fs::read(registry_wasm_path)?;
    near_api::Contract::deploy(registry.account_id().clone())
        .use_code(registry_wasm)
        .with_init_call("new", json!({}))?
        .with_signer(signer.clone())
        .send_to(&network)
        .await?
        .assert_success();

    // An underfunded request must fail synchronously, before minting an unusable NFT.
    registry
        .call_function(
            "create_asset",
            json!({
                "token_id": "underfunded-01",
                "name": "Underfunded Borehole",
                "asset_type": "solar_borehole",
                "reserve_bps": 2000
            }),
        )
        .transaction()
        .deposit(NearToken::from_millinear(100))
        .gas(NearGas::from_tgas(100))
        .with_signer(alice.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?
        .assert_failure();

    let missing_asset: Value = registry
        .call_function("get_asset", json!({"token_id": "underfunded-01"}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert!(missing_asset.is_null());

    let missing_token: Value = registry
        .call_function("nft_token", json!({"token_id": "underfunded-01"}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert!(missing_token.is_null());

    // The same token id remains available and succeeds once the caller supplies the quoted cost.
    let creation_cost: U128 = registry
        .call_function("get_asset_creation_cost", json!({}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    registry
        .call_function(
            "create_asset",
            json!({
                "token_id": "underfunded-01",
                "name": "Underfunded Borehole",
                "asset_type": "solar_borehole",
                "reserve_bps": 2000
            }),
        )
        .transaction()
        .deposit(NearToken::from_yoctonear(creation_cost.0))
        .gas(NearGas::from_tgas(100))
        .with_signer(alice.account_id().clone(), signer)
        .send_to(&network)
        .await?
        .assert_success();

    let recovered_asset: Value = registry
        .call_function("get_asset", json!({"token_id": "underfunded-01"}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(recovered_asset["status"], "ACTIVE");

    Ok(())
}

#[tokio::test]
async fn failed_child_provisioning_can_be_retried_after_registry_code_recovery()
-> testresult::TestResult<()> {
    let sandbox = start_sandbox().await?;
    let network = near_api::NetworkConfig::from_rpc_url("sandbox", sandbox.rpc_addr.parse()?);

    let alice = create_account(&sandbox, "alice.sandbox").await?;
    let registry = create_account(&sandbox, "registry.sandbox")
        .await?
        .as_contract();
    let signer = near_api::Signer::from_secret_key(
        near_sandbox::config::DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY
            .parse()
            .unwrap(),
    )?;

    let failing_path = std::env::var_os("ASSET_REGISTRY_FAIL_WASM_PATH")
        .map(std::path::PathBuf::from)
        .expect("ASSET_REGISTRY_FAIL_WASM_PATH must be set");
    let production_path = std::env::var_os("ASSET_REGISTRY_WASM_PATH")
        .map(std::path::PathBuf::from)
        .expect("ASSET_REGISTRY_WASM_PATH must be set");

    // This registry embeds a shorter invalid child WASM, forcing the
    // child provisioning receipt to fail while leaving the registry contract itself valid.
    let failing_wasm = std::fs::read(failing_path)?;
    near_api::Contract::deploy(registry.account_id().clone())
        .use_code(failing_wasm)
        .with_init_call("new", json!({}))?
        .with_signer(signer.clone())
        .send_to(&network)
        .await?
        .assert_success();

    let creation_cost: U128 = registry
        .call_function("get_asset_creation_cost", json!({}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    registry
        .call_function(
            "create_asset",
            json!({
                "token_id": "retry-borehole",
                "name": "Retry Borehole",
                "asset_type": "solar_borehole",
                "reserve_bps": 2000
            }),
        )
        .transaction()
        .deposit(NearToken::from_yoctonear(creation_cost.0))
        .gas(NearGas::from_tgas(100))
        .with_signer(alice.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?
        .assert_success();

    let failed_asset: Value = registry
        .call_function("get_asset", json!({"token_id": "retry-borehole"}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(failed_asset["status"], "FAILED");
    let missing_child_id: ApiAccountId =
        failed_asset["asset_account_id"].as_str().unwrap().parse()?;
    let absent = Account(missing_child_id)
        .tokens()
        .near_balance()
        .fetch_from(&network)
        .await
        .expect_err("Failed atomic provisioning must not leave a child account");
    let absent_message = format!("{absent:?}");
    assert!(
        absent_message.contains("UnknownAccount") || absent_message.contains("does not exist"),
        "Expected missing-account proof, not an infrastructure error: {absent_message}"
    );

    let preserved_token: Value = registry
        .call_function("nft_token", json!({"token_id":"retry-borehole"}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(preserved_token["owner_id"], alice.account_id().as_str());
    let failed_balance = Account(registry.account_id().clone())
        .tokens()
        .near_balance()
        .fetch_from(&network)
        .await?
        .total
        .as_yoctonear();
    let quote: U128 = registry
        .call_function("get_asset_retry_cost", json!({"token_id":"retry-borehole"}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(quote.0, 1);
    let outsider = create_account(&sandbox, "outsider.sandbox").await?;
    registry
        .call_function(
            "retry_asset_provisioning",
            json!({"token_id":"retry-borehole"}),
        )
        .transaction()
        .deposit(NearToken::from_yoctonear(quote.0))
        .gas(NearGas::from_tgas(100))
        .with_signer(outsider.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?
        .assert_failure();
    registry
        .call_function(
            "retry_asset_provisioning",
            json!({"token_id":"retry-borehole"}),
        )
        .transaction()
        .deposit(NearToken::from_near(1))
        .gas(NearGas::from_tgas(100))
        .with_signer(alice.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?
        .assert_failure();
    for _ in 0..2 {
        registry
            .call_function(
                "retry_asset_provisioning",
                json!({"token_id":"retry-borehole"}),
            )
            .transaction()
            .deposit(NearToken::from_yoctonear(1))
            .gas(NearGas::from_tgas(100))
            .with_signer(alice.account_id().clone(), signer.clone())
            .send_to(&network)
            .await?
            .assert_success();
        let asset: Value = registry
            .call_function("get_asset", json!({"token_id":"retry-borehole"}))
            .read_only()
            .fetch_from(&network)
            .await?
            .data;
        assert_eq!(asset["status"], "FAILED");
    }
    let after_failures = Account(registry.account_id().clone())
        .tokens()
        .near_balance()
        .fetch_from(&network)
        .await?
        .total
        .as_yoctonear();
    // Repeated failures return this asset's funding, with only tiny gas rewards changing balance.
    assert!(after_failures >= failed_balance);
    assert!(after_failures - failed_balance < NearToken::from_millinear(10).as_yoctonear());
    // Replacing registry code preserves state. The failed attempt's child funding
    // remains in the registry because the child receipt's actions rolled back atomically.
    let production_wasm = std::fs::read(production_path)?;
    near_api::Contract::deploy(registry.account_id().clone())
        .use_code(production_wasm)
        .without_init_call()
        .with_signer(signer.clone())
        .send_to(&network)
        .await?
        .assert_success();

    let preserved_asset: Value = registry
        .call_function("get_asset", json!({"token_id": "retry-borehole"}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(
        preserved_asset, failed_asset,
        "Code recovery must preserve the complete asset record"
    );

    let recovery_quote: U128 = registry
        .call_function("get_asset_retry_cost", json!({"token_id":"retry-borehole"}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert!(
        recovery_quote.0 > 1,
        "Fixture must require an actual owner-funded code-size top-up"
    );
    let before_recovery = Account(registry.account_id().clone())
        .tokens()
        .near_balance()
        .fetch_from(&network)
        .await?
        .total
        .as_yoctonear();
    registry
        .call_function(
            "retry_asset_provisioning",
            json!({"token_id": "retry-borehole"}),
        )
        .transaction()
        .deposit(NearToken::from_yoctonear(recovery_quote.0))
        .gas(NearGas::from_tgas(100))
        .with_signer(alice.account_id().clone(), signer)
        .send_to(&network)
        .await?
        .assert_success();

    let after_recovery = Account(registry.account_id().clone())
        .tokens()
        .near_balance()
        .fetch_from(&network)
        .await?
        .total
        .as_yoctonear();
    let child_funding =
        creation_cost.0 - NearToken::from_millinear(100).as_yoctonear() + 2 + recovery_quote.0;
    let expected = before_recovery + recovery_quote.0 - child_funding;
    assert!(after_recovery >= expected);
    assert!(after_recovery - expected < NearToken::from_millinear(10).as_yoctonear());
    let active_asset: Value = registry
        .call_function("get_asset", json!({"token_id": "retry-borehole"}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(active_asset["status"], "ACTIVE");
    assert_eq!(
        active_asset["asset_account_id"],
        failed_asset["asset_account_id"]
    );

    let child_id: ApiAccountId = active_asset["asset_account_id"].as_str().unwrap().parse()?;
    let child = Account(child_id).as_contract();
    let balances: Value = child
        .call_function("get_balances", json!({}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(balances["token_id"], "retry-borehole");

    let token: Value = registry
        .call_function("nft_token", json!({"token_id": "retry-borehole"}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(token["owner_id"], alice.account_id().as_str());

    Ok(())
}
