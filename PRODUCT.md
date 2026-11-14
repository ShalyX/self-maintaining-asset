# PRODUCT — THE SELF-MAINTAINING ASSET

**Product status:** Emergency prototype lock — 14 November 2026
**Working title:** The Self-Maintaining Asset (no external brand/name claim)
**Primary track:** NFTs That Do Something (Track 10)
**Secondary track:** None selected; do not claim Autonomous Business without real agent/payment proof.

## Product thesis

A physical asset should have a persistent operating identity, a dedicated programmable account, a protected maintenance reserve, and a constrained coordinator that helps organize upkeep. Its account, approved work orders, and service history should remain bound to the asset through operator handover.

## Target user

The first prototype is for the owner/operator of a revenue-generating solar water point or commercial borehole. The operator needs to know what money belongs to the asset, what maintenance has been authorized, what work has been completed, and whether the asset's financial and service history survives a change of operator.

**Market limitation:** Nigerian water-sector research supports the maintenance-finance problem, but direct demand for blockchain-based asset accounts and AI maintenance coordination is not yet validated by an operator interview. This prototype must not claim customer adoption or production suitability.

## Problem

Maintenance money, repair approvals, technician work and service history are commonly fragmented across personal accounts, messages, spreadsheets and contractors. A change of operator can separate the asset from the records and funds needed to keep it running. Generic NFT treasuries and water dashboards do not by themselves establish a persistent, transferable maintenance passport.

## One-line premise

A water asset that carries its own operating account, protected repair reserve and verified service history — even when its operator changes.

## Differentiation

The differentiator is the combined lifecycle, not merely an NFT with a wallet:
1. A physical asset receives a persistent NFT passport.
2. A dedicated keyless child account remains attached to the passport.
3. Revenue deposits are deterministically split into operating funds and a maintenance reserve.
4. A bounded maintenance coordinator structures issue triage and work orders, but cannot sign, approve or release funds.
5. A technician's quote and completion evidence are tied to a persistent work order.
6. The current NFT owner alone approves the final reserve payout.
7. Work-order history, evidence hashes and the child account survive NFT transfer.

The NFT represents an operational passport, not legal title to the physical asset.

## Primary user journey

1. Connect a NEAR testnet wallet.
2. Register a solar water point with its asset name/type, reserve policy and dedicated child account.
3. Fund the child account with testnet NEAR. The contract records the deposit and splits it according to the configured reserve percentage.
4. The operator enters a maintenance observation. The coordinator returns a structured triage/diagnosis and recommended work order. All inputs are clearly operator-supplied; no fake IoT readings are permitted.
5. The owner assigns a technician account.
6. The technician submits a quote and work scope.
7. The current owner reviews and approves the quote.
8. The technician submits completion notes and a SHA-256 evidence hash/reference.
9. The owner confirms completion. The registry verifies the current NFT owner, the approved quote, the technician and the completion state, then instructs the child account to pay from the protected reserve.
10. The passport shows the completed work order, payment state, evidence hash and remaining reserve.
11. The owner transfers the NFT. The buyer sees the same child account, balance and service history; the outgoing owner can no longer approve a payout.

## Coordinator authority boundary

- May structure operator observations, suggest diagnostic categories, priority, recommended checks and a work-order draft.
- May not claim sensor verification unless an actual sensor integration exists.
- May not assign or impersonate a technician.
- May not approve a quote, sign a transaction, move funds, or trigger a payout.
- The current NFT owner is the human approval gate.
- Contract authorization and reserve checks are the enforcement boundary; UI controls are not security.

## Locked P0 capabilities

| ID | Capability | Acceptance proof |
|---|---|---|
| P0-01 | Wallet connection and asset registration | Testnet wallet signs create_asset; registry creates NFT and dedicated child account |
| P0-02 | Asset treasury and reserve split | Real testnet deposit; on-chain get_balances shows total, operating and reserve split |
| P0-03 | Bounded maintenance coordination | Operator observation produces a structured, auditable diagnosis/recommendation with explicit source and limitations; no autonomous spending |
| P0-04 | Persistent work-order lifecycle | Issue, technician assignment, quote, owner approval, completion evidence hash and final confirmation are persisted against the NFT |
| P0-05 | Human-gated reserve payout | Only current NFT owner can confirm; exact approved quote is paid to assigned technician from child reserve; replay/overdraft/unauthorized attempts fail |
| P0-06 | Asset passport and service history | UI reads asset, owner, account, balances and work-order history from NEAR; evidence hashes are visible |
| P0-07 | Authority handover | NFT transfer preserves child account/balance/history; new owner can continue, former owner cannot approve |
| P0-08 | Reliability and judge reproducibility | Empty/error/loading/insufficient-reserve/pending/failed states; clean setup and deterministic demo instructions |

## Non-goals and explicit limitations

- No claim of legal ownership transfer for the physical borehole.
- No live NGN/mobile-money revenue rail.
- No actual IoT telemetry or sensor-verified condition.
- No public mainnet or production deployment claim.
- No open technician marketplace, credit underwriting, autonomous spend, or unbounded agent authority.
- A testnet demo account and sample water-point profile are prototype fixtures, not customer assets.
- A browser-local evidence file is not permanent decentralized storage; the chain stores a hash/reference. A production storage provider and operator agreement remain future requirements.
- Full NEP-171 compliance is not claimed unless nft_transfer_call and resolver behavior are implemented and tested.

## Magic/payoff moment

The owner completes and pays a maintenance work order from the asset's protected reserve. The owner then transfers the NFT to a second account. The new owner opens the passport and sees the same child account, remaining reserve, paid repair and evidence hash; the previous owner is rejected if they attempt a new approval.

## Proof standard

The demo is complete only when this journey is executed against NEAR testnet and transaction outcomes are recorded. Local Sandbox tests are useful supporting evidence but do not substitute for testnet transactions. Any mocked technician interaction or sample evidence must be labelled as a test fixture.
