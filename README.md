# The Self-Maintaining Asset

**A physical asset that carries its own operating account, protected maintenance reserve and service history — even when its operator changes.**

Built for the **NEAR Hackathon · Track 10: NFTs That Do Something**.

> This is a testnet prototype. It does not transfer legal ownership of a physical asset, connect to live water sensors, accept NGN payments, or claim that a real repair took place.

[Open the live testnet app](https://self-maintaining-asset.vercel.app) · [Judge walkthrough](JUDGE_WALKTHROUGH.md) · [On-chain evidence](evidence/testnet-proof.json)

## The problem

Maintenance funds, repair approvals and service records are often split across personal accounts, contractor conversations and spreadsheets. When an operator changes, the asset can lose the financial context and service history needed to keep it running.

A dashboard alone does not solve that continuity problem. The asset needs a persistent identity that carries its own operating account, reserve and record of authorized work.

### Market context and limits

- A CRS operational study in Maiduguri documents how low user-fee collection can leave community water committees unable to fund repeated repairs: [CRS water-sector research](https://www.crs.org/sites/default/files/tools-research/engaging_the_informal_private_water_sector_in_nigeria_final_report_crs_2023.pdf).
- Nigeria's 2021 WASHNORM reports widespread water-point operations and maintenance challenges: [UNICEF WASHNORM 2021](https://www.unicef.org/nigeria/media/5951/file/2021%20WASHNORM%20Report%20.pdf).

These sources support the maintenance-finance problem; they do not establish demand for blockchain asset accounts. No direct operator interview or customer adoption is claimed.

## The solution

The Self-Maintaining Asset gives a solar-powered water point an NFT-backed operational passport and a dedicated keyless NEAR sub-account.

- **Persistent identity:** an NEP-171 NFT identifies the asset.
- **Asset-bound account:** a dedicated child account holds its funds and remains attached to the same token ID.
- **Deterministic reserve:** revenue deposits are split by contract rules into operating funds and a maintenance reserve.
- **Constrained coordinator:** operator observations become structured maintenance recommendations and work orders. The coordinator cannot sign, approve or spend.
- **Technician accountability:** the assigned technician submits a quote and completion report tied to the work order.
- **Human approval:** only the current NFT owner can approve the quote and confirm the final reserve payout.
- **Portable history:** the asset account, balances and work-order records remain with the NFT through an ownership handover.

The NFT is an operational passport. It is **not** a deed or legal title to the physical borehole.

## How it works

1. The owner connects a NEAR testnet wallet and registers a water point.
2. The registry mints an NFT and provisions a dedicated sub-account. The creator funds the child account and registry storage.
3. Anyone can deposit testnet NEAR into the asset account. The contract applies the configured reserve split (the demo uses 80% operating / 20% maintenance).
4. The operator records an observed issue. A transparent ruleset produces a bounded diagnosis, priority and recommended checks.
5. The current owner assigns a technician account.
6. The technician submits a quote and scope.
7. The current owner approves the quote on chain.
8. The technician submits a completion note, SHA-256 evidence digest and reference.
9. The current owner confirms completion. The registry checks the assigned technician, approved quote, evidence and replay state, then instructs the child account to pay the exact quoted amount from its maintenance reserve.
10. The owner transfers the NFT. The next owner sees the same account, remaining balance and service history; the former owner can no longer authorize new work or payouts.

## Architecture

```text
Operator / Owner Wallet
        |
        v
React + Vite Asset Passport
        |
        v
NEAR Asset Registry (NEP-171)
  |     |       |        |
  |     |       |        +-- Persistent work orders + evidence digests
  |     |       +----------- Owner / technician authorization
  |     +------------------- NFT owner mapping and handover
  |
  +-- token-id.registry.testnet
        Dedicated Asset Account
        |-- operating balance
        |-- maintenance reserve
        +-- restricted release_maintenance()
```

### Components

| Component | Responsibility |
|---|---|
| `contracts/asset-registry` | NEP-171 asset passport, child-account provisioning, work-order lifecycle, owner/technician authorization, payout callback and replay protection |
| `contracts/asset-account` | Dedicated asset account, revenue accounting, deterministic reserve split and restricted reserve payout |
| `app` | Wallet-connected asset passport, treasury view, maintenance desk, work-order controls and handover UI |
| `demo-evidence` | Explicitly labelled, reproducible testnet evidence fixture |

The registry embeds the independently built asset-account WASM at contracts/asset-registry/res/asset-account.wasm.

### NEAR integration

The project uses NEAR Protocol directly: NEP-171 NFT ownership, registry-controlled sub-account provisioning, NEAR testnet RPC reads, wallet-signed function calls and cross-contract reserve payout. The selected track is NFTs That Do Something. No NEAR Intents, Chain Signatures or other sponsor integrations are claimed.

## Contract-enforced safety

The interface is not the security boundary. The contracts enforce:

- Only the current NFT owner can register a work order, assign the technician, approve a quote, confirm payment or transfer the passport.
- Only the assigned technician can submit the quote and completion evidence.
- The payout recipient must be the assigned technician.
- The payout amount must exactly equal the approved quote.
- The asset account checks that the maintenance reserve can cover the payout.
- A work order cannot be paid twice. A confirmed failure with reserve restoration can be retried; an unknown outcome remains locked for reconciliation.
- The child account only accepts reserve-release instructions from its registry.
- The registry does not subsidize asset provisioning from a shared treasury; the creator funds the child account.

### Current prototype economics

- Default reserve policy: 20% of revenue deposits.
- Work-order storage deposit: 0.05 NEAR, quoted by the registry.
- Payout bookkeeping deposit: 0.01 NEAR, quoted by the registry.
- Passport history: creation prepays an asset-specific storage allowance; transfers debit actual growth. The current owner can top it up with `fund_ownership_storage`; the app adds a 0.01 NEAR top-up when needed.
- Asset creation cost: calculated on chain from child-code storage, operating buffer and registry storage.
- The demo uses testnet NEAR only. These deposits are prototype contract economics, not a production pricing policy.

## Run the app

Requirements: Node.js 24+ and npm.

```bash
cd app
npm ci
cp .env.example .env.local
npm run dev
```

Set `VITE_REGISTRY_ID` to the deployed registry account in `.env.local`. The demo defaults to token ID `lagos-water-01` and asset name `Lagos Solar Water Point 01`.

Production build and lint:

```bash
npm run build
npm run lint
```

## Build and test the contracts

Requirements: Linux, Python 3, Rust 1.97.1, the `wasm32-unknown-unknown` target and Sandbox-compatible host dependencies. Rust is pinned in each contract's toolchain file.

```bash
python3 scripts/validate.py
```

This builds the child with `RUSTFLAGS="--cfg near"`, embeds those exact bytes in the registry, builds the registry, then runs both unit suites and all three Sandbox integrations serially. The deliberately invalid deployment fixture has a separate package name and output file. Production binaries are backed up before synchronization. Logs and hashes go to `.validation/`; the two release binaries go to `artifacts/` only after all checks pass.

For a targeted local unit check:

```bash
cd contracts/asset-registry
cargo test --locked --lib -- --test-threads=1
```

The release build is a pinned-toolchain local build. The scaffold's Docker reproducible-build metadata has not been independently validated; no reproducible-build attestation is claimed.

## Reproducible testnet demonstration

The registry example `contracts/asset-registry/examples/testnet_demo.rs` deploys the registry and executes the end-to-end flow against NEAR testnet using four dedicated demo accounts:

- Registry: `assetops141126.testnet`
- Original owner: `aliceasset141126.testnet`
- New owner: `bobasset141126.testnet`
- Technician: `techasset141126.testnet`

It reads signing keys from local NEAR CLI credential files under `~/.near-credentials/testnet/`. **Never copy those credential files or private keys into this repository.**

After building both contracts:

```bash
cd contracts/asset-registry
ASSET_REGISTRY_WASM_PATH=target/wasm32-unknown-unknown/release/asset_registry.wasm \
  cargo run --locked --example testnet_demo
```

To rerun the lifecycle after the registry is already deployed, set `SKIP_REGISTRY_DEPLOY=1`. The script preserves existing asset/work-order state and stops rather than replaying a payout whose outcome is unknown.

The testnet demo is intended to prove the following payoff: the passport moves from the original owner to the new owner, the child account and maintenance reserve remain the same, the former owner is rejected, and the new owner can authorize the technician's exact approved payout.

## Evidence boundary

The file in `demo-evidence/maintenance-completion-fixture.txt` is intentionally labelled as a **simulated testnet fixture**. It does not represent a real inspection or repair. Its SHA-256 digest verifies that the referenced file has not changed; it does not prove that a physical-world event occurred.

In the browser prototype, an uploaded evidence file remains in that browser session. The chain stores its digest and a reference, not a permanent decentralized copy of the file.

## What this prototype does not claim

- No live IoT telemetry, sensor-verified diagnosis or condition monitoring.
- No autonomous spending or AI-controlled wallet.
- No real NGN/mobile-money payment rail.
- No physical repair or customer adoption claim.
- No legal transfer of the physical borehole.
- No mainnet or production readiness.
- No full NEP-171 `nft_transfer_call` / resolver implementation; the prototype uses the core NFT transfer path.

The maintenance coordinator is a transparent ruleset in this prototype, not a claim of an LLM integration. It can organize operator-supplied information but cannot certify a fault or a repair.

## Verification status

This section is updated as each proof gate passes. Local unit tests, Sandbox integration tests and public testnet transactions are reported separately; none is substituted for another.

- Asset-account unit tests: 6/6 passed.
- App production build: passed.
- App lint: 0 warnings, 0 errors.
- Asset-registry unit tests: 20/20 passed.
- Isolated Sandbox integrations: 3/3 passed against the current release artifacts, including full maintenance lifecycle, unauthorized retry, real failed child deployment and code recovery.
- Public testnet: deployment, creation, 1 NEAR revenue split, diagnosis, assignment, quote, owner approval, completion digest, NFT handover, former-owner rejection and 0.1 NEAR payout verified.
- Final demo state: owner `bobasset141126.testnet`; work order `WO-001` is `PAID`; operating 0.8 NEAR; maintenance reserve 0.1 NEAR.
- Deployed frontend browser QA: in progress; tracked separately in [EVIDENCE.md](EVIDENCE.md).

## Hackathon disclosure

Built by Shallom Alabi for the November 2026 NEAR Hackathon. This is an independent project; the team lead has no other entry in this hackathon.

## License

MIT. See [LICENSE](LICENSE).
