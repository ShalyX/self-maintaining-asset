use near_api::{
    Account, AccountId as ApiAccountId, Contract, NearGas, NearToken, NetworkConfig, Signer,
};
use near_sdk::json_types::U128;
use near_sdk::serde_json::{Value, json};
use std::{error::Error, fs, io};

const REGISTRY: &str = "assetops141126.testnet";
const OWNER: &str = "aliceasset141126.testnet";
const TRANSFEREE: &str = "bobasset141126.testnet";
const TECHNICIAN: &str = "techasset141126.testnet";
const TOKEN_ID: &str = "lagos-water-01";
const WORK_ORDER_ID: &str = "WO-001";
const EVIDENCE_HASH: &str = "e46d36ea7e8eb36dddabed6d8572d4d6adc653dedfea63d6a53c84091d8a1b5f";

fn error(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, message.to_string())
}

fn load_signer(account_id: &str) -> Result<std::sync::Arc<Signer>, Box<dyn Error>> {
    let home = std::env::var("HOME")?;
    let path = format!("{home}/.near-credentials/testnet/{account_id}.json");
    let value: Value = near_sdk::serde_json::from_str(&fs::read_to_string(path)?)?;
    let private_key = value
        .get("private_key")
        .and_then(Value::as_str)
        .ok_or_else(|| error("NEAR credential file has no private_key field"))?;
    Ok(Signer::from_secret_key(private_key.parse()?)?)
}

async fn read_asset(registry: &Contract, network: &NetworkConfig) -> Result<Value, Box<dyn Error>> {
    Ok(registry
        .call_function("get_asset", json!({"token_id": TOKEN_ID}))
        .read_only()
        .fetch_from(network)
        .await?
        .data)
}

async fn read_order(registry: &Contract, network: &NetworkConfig) -> Result<Value, Box<dyn Error>> {
    Ok(registry
        .call_function(
            "get_work_order",
            json!({"token_id": TOKEN_ID, "work_order_id": WORK_ORDER_ID}),
        )
        .read_only()
        .fetch_from(network)
        .await?
        .data)
}

async fn read_ownership_history(
    registry: &Contract,
    network: &NetworkConfig,
) -> Result<Value, Box<dyn Error>> {
    Ok(registry
        .call_function("get_asset_ownership_history", json!({"token_id": TOKEN_ID}))
        .read_only()
        .fetch_from(network)
        .await?
        .data)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("NEAR TESTNET — SELF-MAINTAINING ASSET");
    println!("Registry: {REGISTRY}");
    println!("Asset: {TOKEN_ID}");

    let network = NetworkConfig::from_rpc_url("testnet", "https://rpc.testnet.near.org".parse()?);
    let registry_id: ApiAccountId = REGISTRY.parse()?;
    let owner_id: ApiAccountId = OWNER.parse()?;
    let transferee_id: ApiAccountId = TRANSFEREE.parse()?;
    let technician_id: ApiAccountId = TECHNICIAN.parse()?;
    let registry_signer = load_signer(REGISTRY)?;
    let owner_signer = load_signer(OWNER)?;
    let transferee_signer = load_signer(TRANSFEREE)?;
    let technician_signer = load_signer(TECHNICIAN)?;
    let registry = Account(registry_id.clone()).as_contract();

    let wasm_path = std::env::var("ASSET_REGISTRY_WASM_PATH").unwrap_or_else(|_| {
        "target/wasm32-unknown-unknown/release/asset_registry.wasm".to_string()
    });
    let registry_wasm = fs::read(&wasm_path)?;
    let skip_deploy = std::env::var("SKIP_REGISTRY_DEPLOY").unwrap_or_default() == "1";
    if !skip_deploy {
        println!("1/8 Deploying and initializing registry contract…");
        Contract::deploy(registry_id.clone())
            .use_code(registry_wasm)
            .with_init_call("new", json!({}))?
            .with_signer(registry_signer)
            .send_to(&network)
            .await
            .map(|outcome| {
                println!(
                    "TX {} signer={} receiver={}",
                    outcome.outcome().transaction_hash,
                    outcome.transaction().signer_id(),
                    outcome.transaction().receiver_id()
                );
                outcome
            })?
            .assert_success();
    } else {
        println!("1/8 Registry deployment skipped by SKIP_REGISTRY_DEPLOY=1");
    }

    let mut asset = read_asset(&registry, &network).await?;
    if asset.is_null() {
        println!("2/8 Registering the asset passport and its dedicated child account…");
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
                    "token_id": TOKEN_ID,
                    "name": "Lagos Solar Water Point 01",
                    "asset_type": "solar_water_point",
                    "reserve_bps": 2000
                }),
            )
            .transaction()
            .deposit(NearToken::from_yoctonear(creation_cost.0))
            .gas(NearGas::from_tgas(100))
            .with_signer(owner_id.clone(), owner_signer.clone())
            .send_to(&network)
            .await
            .map(|outcome| {
                println!(
                    "TX {} signer={} receiver={}",
                    outcome.outcome().transaction_hash,
                    outcome.transaction().signer_id(),
                    outcome.transaction().receiver_id()
                );
                outcome
            })?
            .assert_success();
        asset = read_asset(&registry, &network).await?;
    } else {
        println!("2/8 Existing asset passport found; preserving its state.");
    }
    if asset["status"] != "ACTIVE" {
        return Err(error(
            "Asset account is not ACTIVE; stop and reconcile provisioning before continuing",
        )
        .into());
    }
    let child_id: ApiAccountId = asset["asset_account_id"]
        .as_str()
        .ok_or_else(|| error("Asset record has no child account id"))?
        .parse()?;
    let child = Account(child_id.clone()).as_contract();

    let mut balances: Value = child
        .call_function("get_balances", json!({}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    if balances["total_deposited"].as_str().unwrap_or("0") == "0" {
        println!("3/8 Depositing 1 NEAR; contract splits 80% operating / 20% maintenance reserve…");
        child
            .call_function("deposit_revenue", json!({}))
            .transaction()
            .deposit(NearToken::from_near(1))
            .with_signer(owner_id.clone(), owner_signer.clone())
            .send_to(&network)
            .await
            .map(|outcome| {
                println!(
                    "TX {} signer={} receiver={}",
                    outcome.outcome().transaction_hash,
                    outcome.transaction().signer_id(),
                    outcome.transaction().receiver_id()
                );
                outcome
            })?
            .assert_success();
        balances = child
            .call_function("get_balances", json!({}))
            .read_only()
            .fetch_from(&network)
            .await?
            .data;
    } else {
        println!("3/8 Existing asset revenue deposit found; not funding twice.");
    }

    let mut order = read_order(&registry, &network).await?;
    let current_owner: Value = registry
        .call_function("get_asset_owner", json!({"token_id": TOKEN_ID}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    let mut owner_name = current_owner.as_str().unwrap_or("");
    let owner_signer_for =
        |name: &str| -> Result<(ApiAccountId, std::sync::Arc<Signer>), Box<dyn Error>> {
            if name == OWNER {
                Ok((owner_id.clone(), owner_signer.clone()))
            } else if name == TRANSFEREE {
                Ok((transferee_id.clone(), transferee_signer.clone()))
            } else {
                Err(error("Asset owner is not one of the configured demo accounts").into())
            }
        };

    if order.is_null() {
        println!("4/8 Recording operator observation and coordinator diagnosis…");
        let storage_cost: U128 = registry
            .call_function("get_work_order_creation_cost", json!({}))
            .read_only()
            .fetch_from(&network)
            .await?
            .data;
        let (current_owner_id, current_owner_signer) = owner_signer_for(owner_name)?;
        registry
            .call_function(
                "create_work_order",
                json!({
                    "token_id": TOKEN_ID,
                    "work_order_id": WORK_ORDER_ID,
                    "observation": "Pump stopped delivering water after the controller showed a fault.",
                    "observation_evidence_hash": null,
                    "diagnosis": "Pump or delivery-path interruption",
                    "diagnosis_source": "transparent-ruleset-v1",
                    "recommended_action": "Isolate power, inspect controller protection trips and verify the delivery line.",
                    "priority": "HIGH"
                }),
            )
            .transaction()
            .deposit(NearToken::from_yoctonear(storage_cost.0))
            .gas(NearGas::from_tgas(100))
            .with_signer(current_owner_id, current_owner_signer)
            .send_to(&network)
            .await.map(|outcome| { println!("TX {} signer={} receiver={}", outcome.outcome().transaction_hash, outcome.transaction().signer_id(), outcome.transaction().receiver_id()); outcome })?
            .assert_success();
        order = read_order(&registry, &network).await?;
    } else {
        println!("4/8 Existing work order found; continuing its recorded lifecycle.");
    }

    let mut status = order["status"].as_str().unwrap_or("UNKNOWN").to_string();
    if status == "DIAGNOSED" {
        println!("5/8 Assigning the technician…");
        let (current_owner_id, current_owner_signer) = owner_signer_for(owner_name)?;
        registry
            .call_function(
                "assign_technician",
                json!({
                    "token_id": TOKEN_ID,
                    "work_order_id": WORK_ORDER_ID,
                    "technician_id": technician_id
                }),
            )
            .transaction()
            .with_signer(current_owner_id, current_owner_signer)
            .send_to(&network)
            .await
            .map(|outcome| {
                println!(
                    "TX {} signer={} receiver={}",
                    outcome.outcome().transaction_hash,
                    outcome.transaction().signer_id(),
                    outcome.transaction().receiver_id()
                );
                outcome
            })?
            .assert_success();
        order = read_order(&registry, &network).await?;
        status = order["status"].as_str().unwrap_or("UNKNOWN").to_string();
    }
    if status == "AWAITING_QUOTE" {
        println!("5/8 Technician submitting a 0.1 NEAR quote…");
        registry
            .call_function(
                "submit_technician_quote",
                json!({
                    "token_id": TOKEN_ID,
                    "work_order_id": WORK_ORDER_ID,
                    "quote_amount": NearToken::from_millinear(100).as_yoctonear().to_string(),
                    "quote_note": "DEMO QUOTE: replace pump seal, inspect coupling and run a delivery test."
                }),
            )
            .transaction()
            .with_signer(technician_id.clone(), technician_signer.clone())
            .send_to(&network)
            .await.map(|outcome| { println!("TX {} signer={} receiver={}", outcome.outcome().transaction_hash, outcome.transaction().signer_id(), outcome.transaction().receiver_id()); outcome })?
            .assert_success();
        order = read_order(&registry, &network).await?;
        status = order["status"].as_str().unwrap_or("UNKNOWN").to_string();
    }
    if status == "QUOTED" {
        println!("5/8 Current owner approving the technician quote…");
        let (current_owner_id, current_owner_signer) = owner_signer_for(owner_name)?;
        registry
            .call_function(
                "approve_work_order_quote",
                json!({"token_id": TOKEN_ID, "work_order_id": WORK_ORDER_ID}),
            )
            .transaction()
            .with_signer(current_owner_id, current_owner_signer)
            .send_to(&network)
            .await
            .map(|outcome| {
                println!(
                    "TX {} signer={} receiver={}",
                    outcome.outcome().transaction_hash,
                    outcome.transaction().signer_id(),
                    outcome.transaction().receiver_id()
                );
                outcome
            })?
            .assert_success();
        order = read_order(&registry, &network).await?;
        status = order["status"].as_str().unwrap_or("UNKNOWN").to_string();
    }
    if status == "APPROVED" {
        println!("6/8 Technician submitting the labelled testnet evidence fixture…");
        registry
            .call_function(
                "submit_work_order_completion",
                json!({
                    "token_id": TOKEN_ID,
                    "work_order_id": WORK_ORDER_ID,
                    "completion_note": "SIMULATED TESTNET DEMO ONLY: completion workflow exercised; no physical repair performed.",
                    "completion_evidence_hash": EVIDENCE_HASH,
                    "completion_evidence_uri": format!("repo://demo-evidence/maintenance-completion-fixture.txt#sha256={EVIDENCE_HASH}")
                }),
            )
            .transaction()
            .with_signer(technician_id.clone(), technician_signer.clone())
            .send_to(&network)
            .await.map(|outcome| { println!("TX {} signer={} receiver={}", outcome.outcome().transaction_hash, outcome.transaction().signer_id(), outcome.transaction().receiver_id()); outcome })?
            .assert_success();
        order = read_order(&registry, &network).await?;
        status = order["status"].as_str().unwrap_or("UNKNOWN").to_string();
    }

    if status == "PAYOUT_UNKNOWN" || status == "PAYOUT_PENDING" {
        return Err(error(
            "Payout is pending or unknown; do not replay it. Reconcile the transaction first.",
        )
        .into());
    }

    if owner_name == OWNER {
        println!("7/8 Transferring the passport from the original operator to the new owner…");
        registry
            .call_function(
                "nft_transfer",
                json!({
                    "receiver_id": transferee_id,
                    "token_id": TOKEN_ID,
                    "approval_id": null,
                    "memo": "NEAR Hackathon testnet operator handover"
                }),
            )
            .transaction()
            .deposit(NearToken::from_yoctonear(1))
            .with_signer(owner_id.clone(), owner_signer.clone())
            .send_to(&network)
            .await
            .map(|outcome| {
                println!(
                    "TX {} signer={} receiver={}",
                    outcome.outcome().transaction_hash,
                    outcome.transaction().signer_id(),
                    outcome.transaction().receiver_id()
                );
                outcome
            })?
            .assert_success();
        owner_name = TRANSFEREE;
    }
    if owner_name != TRANSFEREE {
        return Err(error("Expected the transferee to be the current asset owner").into());
    }

    if status == "COMPLETION_SUBMITTED" || status == "PAYMENT_FAILED" {
        println!(
            "8/8 Verifying former-owner rejection, then paying the assigned technician from reserve…"
        );
        let payout_cost: U128 = registry
            .call_function("get_maintenance_payout_cost", json!({}))
            .read_only()
            .fetch_from(&network)
            .await?
            .data;
        // The former owner must not be able to authorize payment after handover.
        registry
            .call_function(
                "confirm_work_order_completion",
                json!({"token_id": TOKEN_ID, "work_order_id": WORK_ORDER_ID}),
            )
            .transaction()
            .deposit(NearToken::from_yoctonear(payout_cost.0))
            .gas(NearGas::from_tgas(100))
            .with_signer(owner_id.clone(), owner_signer.clone())
            .send_to(&network)
            .await
            .map(|outcome| {
                println!("EXPECTED_FAILURE_TX {}", outcome.outcome().transaction_hash);
                outcome
            })?
            .assert_failure();

        let technician_before = Account(technician_id.clone())
            .tokens()
            .near_balance()
            .fetch_from(&network)
            .await?
            .total
            .as_yoctonear();
        registry
            .call_function(
                "confirm_work_order_completion",
                json!({"token_id": TOKEN_ID, "work_order_id": WORK_ORDER_ID}),
            )
            .transaction()
            .deposit(NearToken::from_yoctonear(payout_cost.0))
            .gas(NearGas::from_tgas(100))
            .with_signer(transferee_id.clone(), transferee_signer.clone())
            .send_to(&network)
            .await
            .map(|outcome| {
                println!(
                    "TX {} signer={} receiver={}",
                    outcome.outcome().transaction_hash,
                    outcome.transaction().signer_id(),
                    outcome.transaction().receiver_id()
                );
                outcome
            })?
            .assert_success();
        let technician_after = Account(technician_id.clone())
            .tokens()
            .near_balance()
            .fetch_from(&network)
            .await?
            .total
            .as_yoctonear();
        if technician_after < technician_before + NearToken::from_millinear(100).as_yoctonear() {
            return Err(error("Technician balance did not increase by the approved quote").into());
        }
    }

    let final_asset = read_asset(&registry, &network).await?;
    let ownership_history = read_ownership_history(&registry, &network).await?;
    let history = ownership_history
        .as_array()
        .ok_or_else(|| error("Ownership history view did not return an array"))?;
    if history.len() < 2 {
        return Err(error("Expected initial mint and transfer events in ownership history").into());
    }
    if history.first().and_then(|event| event["owner_id"].as_str()) != Some(OWNER)
        || history.last().and_then(|event| event["owner_id"].as_str()) != Some(TRANSFEREE)
    {
        return Err(error(
            "Ownership history does not show the original owner followed by the transferee",
        )
        .into());
    }
    let final_owner: Value = registry
        .call_function("get_asset_owner", json!({"token_id": TOKEN_ID}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    let final_order = read_order(&registry, &network).await?;
    let final_balances: Value = child
        .call_function("get_balances", json!({}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    if final_owner.as_str() != Some(TRANSFEREE) {
        return Err(error("Final owner is not the transferee").into());
    }
    if final_asset["asset_account_id"].as_str() != Some(child_id.as_str()) {
        return Err(error("Child account identity changed during handover").into());
    }
    if final_order["status"] != "PAID" {
        return Err(error("Work order did not reach PAID status").into());
    }

    println!("TESTNET_DEMO_VERIFIED");
    println!("asset={}", final_asset);
    println!("owner={}", final_owner);
    println!("ownership_history={}", ownership_history);
    println!("work_order={}", final_order);
    println!("balances={}", final_balances);
    println!("Evidence SHA-256: {EVIDENCE_HASH}");
    println!("No physical repair was performed; the on-chain evidence is a labelled demo fixture.");
    Ok(())
}
