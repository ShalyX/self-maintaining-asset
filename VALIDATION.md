# VALIDATION — THE SELF-MAINTAINING ASSET

## Approved thesis under test
A physical asset should have a persistent operating identity, a protected financial reserve and a constrained agent that coordinates maintenance. The first demonstration candidate is a revenue-generating solar water point / commercial borehole.

This is not yet a product lock. The target customer, exact workflow, and P0 scope remain open.

## Gate 1 — Asset identity, account and ownership transfer
**Question:** Can each physical asset be represented by an NEP-171 NFT linked to its own NEAR sub-account, with funds and maintenance history remaining attached to the asset when the NFT changes hands?

### Evidence and findings
- NEAR Track 10 explicitly encourages NEP-171 and sub-accounts.
- NEAR Top Level Accounts uses a registry -> extension -> wallet pattern.
- Source inspection of the TLA wallet shows ownership rotation clears spending grants and sweeps native NEAR to the existing payout account before the payout account is repointed. The registry transfer path also checks allowlisted FT balances before dispatch. Therefore, do not assume a TLA wallet's balance automatically follows an NFT to the buyer.
- Proposed alternative to test: a custom asset registry mints the NFT and creates a child asset-account contract. The child account accepts incoming funds and only accepts outgoing payment instructions from the registry. The registry checks the current NFT owner before authorizing a payment. NFT transfer changes the registry's owner mapping; the child account and its balance remain unchanged. The account has no user-controlled full-access key.
- This architecture is a hypothesis, not yet proven. Cross-contract account creation, access control, and transfer edge cases must be tested.

### Pass criteria
1. Mint one asset NFT and create its distinct child account.
2. Deposit testnet NEAR into that child account.
3. Transfer the NFT from Alice to Bob.
4. Verify the same child account and balance remain.
5. Verify Alice can no longer authorize a payout.
6. Verify Bob can authorize a permitted payout.
7. Verify a non-owner and an unauthorized contract call are rejected.
8. Verify failed payout does not silently erase reserve accounting.

**Status: VERIFIED IN LOCAL SANDBOX.** All eight pass criteria are covered by the three serial registry integration scenarios: mint/child creation, child funding, NFT transfer with the same child account and balance, former-owner rejection, current-owner payout, unauthorized-call rejection, and failed-payout accounting recovery. This is local Sandbox proof only; no testnet deployment is claimed. The current NFT interface supports direct transfer but not `nft_transfer_call`, so complete NEP-171 core-interface compliance is not claimed.

## Gate 2 — Financial safety and maintenance reserve
**Question:** Can the asset account automatically allocate revenue to a reserve while preventing the AI agent or operator from spending protected funds outside the maintenance policy?

### Proposed invariants
- Every deposit is accounted for exactly once.
- Revenue split is deterministic and recorded on chain.
- Maintenance reserve cannot be withdrawn as general operating funds.
- The agent may create a maintenance proposal and request quotes, but cannot directly release funds.
- Only the current NFT owner can approve a payout, and only against an approved work order.
- Payout amount cannot exceed the funded reserve or the approved quote.
- Duplicate approval, replay, unauthorized recipient and over-budget payout are rejected.
- Failed cross-contract payment restores or preserves the correct accounting state.
- A transfer of the NFT never resets, duplicates or drains the reserve.

**Status: CONTRACT-LEVEL INVARIANTS VERIFIED IN LOCAL SANDBOX; PRODUCT/AGENT INTEGRATION OPEN.** The contracts and Sandbox tests cover deterministic revenue allocation, reserve protection, current-owner authorization, work-order payout bounds, replay protection, uncertain outcomes, and failed-transfer reserve restoration. A user-facing agent proposal/technician workflow and public testnet transaction remain unbuilt.

## Gate 3 — Real maintenance workflow and payoff
**Question:** Can a real asset operator go from an observed maintenance issue to a completed and paid service job?

### Required user journey
1. Register a specific water asset with pump, solar, tank and service metadata.
2. Fund its operating account and maintenance reserve.
3. Record a maintenance issue with a real operator-supplied observation (no fabricated IoT telemetry).
4. The agent creates a structured diagnosis and recommended work order.
5. A technician submits a quote and identity/account.
6. The owner reviews the quote and approves it.
7. The technician submits completion evidence (report, before/after photos or checklist).
8. The owner confirms completion.
9. The contract pays the technician from the maintenance reserve and records the service event.
10. The asset passport displays the new maintenance history and remaining reserve.

### Pass criteria
- All core state transitions are real and persisted.
- Payment is a real NEAR testnet transaction.
- Completion evidence is hashed/referenced and tied to the work order.
- Human approval remains the final payout gate.
- The product clearly distinguishes operator input from sensor-verified facts.
- Demo reset path is documented.

**Status: OPEN — workflow specified for validation; no integrated journey exists.**

## Gate 4 — Market viability and differentiation
**Question:** Is this a real, differentiated problem rather than a generic NFT treasury, job marketplace or water dashboard?

### Problem evidence
- CRS's 2022 operational research in Maiduguri says community water committees can become trapped in repeated breakages because low user-fee collection leaves them unable to pay for repairs. It found private borehole owners invest significantly more in O&M than WATCOMs. Its pilots included 12 private borehole owners and 90 pushcart sellers; the borehole-owner work included savings/financial-management support, flow meters, maintenance targets and monitoring plans. Source: https://www.crs.org/sites/default/files/tools-research/engaging_the_informal_private_water_sector_in_nigeria_final_report_crs_2023.pdf
- Nigeria's WASHNORM 2021 estimated 2,313,566 water points and 32,769 solar-powered motorized boreholes. It reported only 11% of water points as dependable for O&M and only 7.2% of publicly used water points with a tariff system. Source: https://www.unicef.org/nigeria/media/5951/file/2021%20WASHNORM%20Report%20.pdf
- A separate 2024 Nigerian survey of 1,696 communities reports substantial public-borehole failure/repair needs; keep its sample and public-facility scope explicit.
- These sources establish water-infrastructure maintenance and financing as real issues and support focusing first on private, revenue-generating borehole operators rather than volunteer committees. They do not prove demand for blockchain accounts, AI procurement or a crypto payment rail.

### Competitor findings
- MINTIQ is a live public testnet project (per its builders) for generic NFT treasuries, AI agents, reserve rules, contributor jobs, scoring and payouts. Its website is publicly reachable; underlying transactions were not independently verified.
- Nivry is a public NEAR project/research repository around NFT-bound agent authority and treasury. It is a direct technical concept competitor, but its implementation is not independently verified.
- ProofOps describes AI-assisted industrial maintenance evidence, human approval and escrow settlement.
- Project Maji/MajiPay already combines water payments, maintenance funding, system tracking and automated alerts.
- Water Station offers solar/off-grid prepaid water kiosks, traceable sales, operator tooling and remote monitoring.
- GeoTek Water Solutions offers borehole monitoring and maintenance alerts.
- UpKeep and other CMMS tools already cover work orders, preventive maintenance and asset history.

### Differentiation hypothesis
Do not build:
- a generic AI agent NFT marketplace;
- an open-ended freelancer marketplace;
- a water-payment or borehole-monitoring dashboard;
- a generic escrow product.

Test this narrower proposition:
**A physical asset's identity, dedicated account, protected maintenance reserve, approved repair workflow and service history remain bound together through operator handover.**

The asset NFT is the physical asset's operational passport, not a claim of legal title. The AI agent coordinates maintenance but cannot spend the reserve without the current owner's approval. The first template is a commercial solar water point; the architecture may later support other revenue-generating infrastructure assets.

### Commercial risks
- No direct interview with a water-point operator yet.
- No evidence that small Nigerian operators want an on-chain treasury.
- No fiat/mobile-money integration selected; testnet NEAR deposits are not equivalent to customer NGN revenue.
- MajiPay and Water Station already solve parts of the water revenue/maintenance problem.
- NFT handover of a service account does not legally transfer the physical asset; any real deployment needs off-chain agreements.
- A DePIN/infrastructure operator or asset-finance audience may be a more natural initial user than an informal community water committee.

**Status: MODIFY/NARROW — core thesis survives provisionally, but market differentiation must be asset-specific and the first user segment must be proven.**

## Overall validation verdict
**IN PROGRESS — NOT PASSED.**

The concept is not ready for PRODUCT_LOCK. Gate 1's account persistence and owner-authority mechanism is verified in local Sandbox. Gate 2's contract-level financial controls are verified in local Sandbox, while agent/product integration remains open. Gate 3's real maintenance workflow and Gate 4's differentiated customer need are the highest-risk open questions.

## Next validation work
The bounded engineering spike is complete in local Sandbox; do not repeat it as the next task. Advance Gate 3 with a minimum end-to-end operator maintenance workflow, and Gate 4 with direct target-operator evidence for portable service history, asset-bound reserves and authority handover. Preserve the current contract architecture unless a specific product requirement or adversarial finding justifies a change. Public testnet deployment, complete NEP-171 core-interface compliance, product lock and submission readiness remain separate gates.


## Validation research addendum — 13 November 2026

### Additional competitor kill-search
- PaygOps (Masunga, Tanzania) already provides PAYG operations software for solar and water businesses: customer/credit contracts, mobile-money collections, inventory, after-sales tickets, offline field agents, and integrations with 45+ appliance brands. Source: https://liners.com/paygops (third-party product directory; verify feature claims with vendor before relying on them).
- Project Maji's MajiPay explicitly links digital water payments, operations, maintenance funding, real-time system tracking and automated alerts. Source: https://www.projectmaji.org/maji-solutions
- Avisers / Water Station is an especially close adjacent competitor: prepaid solar water kiosks, metered sales, revenue visibility, operator app, fault alerts, technician coverage and maintenance funding from water revenue. Its own pages say the system is workshop-tested and entering pilots; distinguish vendor claims from independently measured field results. Sources: https://ws.avisers.com/en/water-point-management-system and https://ws.avisers.com/en/solutions/wash-ngos
- OmiChain already focuses on verified water projects, vetted drillers, evidence-backed completion and conditional contractor payments. Source: https://omichain.org/
- NEAR's own Track 10 starter prompt explicitly illustrates an NFT carrying a sub-account, balance, ownership and access rights. Our NFT-plus-wallet primitive alone is therefore not original. Source: https://github.com/nearai/near-hackathon-prompt-examples/blob/main/tracks/10-nfts.md
- NEAR Top Level Accounts are live and already implement NFT-linked account control. Their transfer semantics include special asset handling; do not claim our custom implementation is novel merely because it binds an NFT to an account. Sources: https://docs.nearaccounts.com/docs/architecture and https://near.org/blog/introducing-top-level-accounts-and-recovery-keys-on-near

### Revised differentiation assessment
The water vertical is a strong problem context but crowded in operations, payments, monitoring, maintenance reserves and verified repairs. The only remaining potentially defensible wedge is the **portable operating history + asset-bound reserve + authority handover protocol** across independent operators and service providers, not a water kiosk management suite. This wedge still needs customer validation and competitor-specific proof. If we cannot establish that an asset's service history and restricted reserve need to persist across operator changes (and that current products fail to provide this), Gate 4 fails.

### Technical execution observation — superseded by engineering verification
The earlier WSL/E: I/O and Sandbox startup failures were resolved for the verification run. Rust 1.97.1 / near-sdk 5.30.1 were used from the Ubuntu 24.04 local workspace. The production child and registry artifacts were rebuilt, hashes verified, and all 19 unit tests plus 3 serial isolated Sandbox integrations passed. Strict Clippy, formatting and documentation test targets passed. See the validated-engineering review package, validation-results.json and logs for commands, timestamps and exit codes. No public testnet/mainnet deployment is claimed.

### Current validation status — 13 November 2026
- Gate 1: VERIFIED IN LOCAL SANDBOX — child account/balance continuity, owner handover, payout authority and failure recovery.
- Gate 2: CONTRACT-LEVEL INVARIANTS VERIFIED IN LOCAL SANDBOX; user-facing agent/product integration remains open.
- Gate 3: OPEN — no integrated operator-to-technician maintenance journey or public testnet transaction.
- Gate 4: MODIFY/NARROW — portable asset history, asset-bound reserve and authority handover remain a differentiation hypothesis requiring direct customer evidence.
- Overall: IN PROGRESS; product lock remains prohibited.
