# EVIDENCE — CLAIM REGISTRY

| ID | Claim | Status | Evidence / method | Limitation |
|---|---|---|---|---|
| C-01 | Community water committees can struggle to fund repairs; private borehole owners are an active, revenue-generating market | VERIFIED | CRS 2022 operational research: https://www.crs.org/sites/default/files/tools-research/engaging_the_informal_private_water_sector_in_nigeria_final_report_crs_2023.pdf; 12 borehole owners and 90 pushcart sellers were included in pilots; savings/financial management and O&M support were part of the work | Maiduguri-specific and small pilot sample; does not prove demand for blockchain |
| C-02 | Nigeria has a material solar-borehole and water-point maintenance base | VERIFIED | UNICEF/NBS/World Bank WASHNORM 2021: https://www.unicef.org/nigeria/media/5951/file/2021%20WASHNORM%20Report%20.pdf; estimated 2,313,566 water points and 32,769 solar-powered motorized boreholes; only 11% of water points met the report's O&M dependability standard | 2021 survey; counts are not all commercial/private assets |
| C-03 | MajiPay already links water payments, maintenance funding, tracking and alerts | VERIFIED | https://www.projectmaji.org/maji-solutions | Adjacent product; no NFT-bound treasury evidence |
| C-04 | Water Station offers solar/off-grid prepaid water, operator tooling and traceable revenue | VERIFIED | https://ws.avisers.com/en | No token-bound asset account advertised |
| C-05 | GeoTek Water Solutions offers borehole monitoring and maintenance alerts | OBSERVED | https://play.google.com/store/apps/details?id=com.geotek.watersolutions | Developer-supplied product claims |
| C-06 | MINTIQ presents a generic NFT agent treasury with jobs, reserves and payouts | OBSERVED | https://mintiq-pink.vercel.app; public builder posts | Website reachable; underlying chain behavior not independently verified |
| C-07 | Nivry is a current NEAR concept competitor for NFT-bound agent authority/treasury | OBSERVED | https://github.com/Nebulaz7/Nivry | Research/planning repo; shipped behavior not verified |
| C-08 | NEAR TLA transfer behavior can sweep funds to the outgoing payout account | VERIFIED | Source: https://github.com/houseofstake/tla-contracts/blob/d85860a8cc2bfcb8b617b28cab558da5d39333fc/contracts/hos-wallet/src/lib.rs and contracts/tla-registry/src/marketplace.rs | Must re-check against the exact deployed version before relying on it |
| C-09 | Custom NFT registry + child asset account can preserve balance and owner-gated payouts through transfer | VERIFIED — LOCAL SANDBOX | Three serial Sandbox integrations: child creation/funding, same-account balance continuity through NFT transfer, former-owner rejection, current-owner payout, failed-payout restoration, provisioning failure/recovery and funding conservation. See validated-engineering/validation-results.json and logs. | No public testnet/mainnet proof; direct transfer supported, `nft_transfer_call` absent; not a complete NEP-171 core-interface claim |
| C-10 | Operators will adopt an on-chain asset treasury for water infrastructure | UNPROVEN | No direct customer interview or adoption evidence | Critical market assumption |
| C-11 | A real technician repair workflow can be funded and settled from the asset's reserve on NEAR | UNPROVEN | Planned testnet workflow | Requires integrated transaction evidence |
| C-12 | A fiat/mobile-money rail is available for Nigerian customer revenue | UNPROVEN | No provider selected or integration tested | Do not claim live NGN payments |


## Engineering audit checkpoint — 13 November 2026

C-09/C-11 remain unverified for the current draft. Prior handoff test results are historical reports; this run executed no contract assertions because WSL became read-only. Critical mint-refund subsidy observed in installed NFT standard/source call path. Proposed fixes and unexecuted regression tests are in the recovered engineering package linked from PROJECT.md. Preserved artifact hashes match initial inspection; they predate draft changes.


## Local contract evidence — 13 November 2026

C-09 technical mechanism VERIFIED IN LOCAL SANDBOX: account/balance continuity and current-owner payout authority, plus provisioning recovery and funding conservation. Evidence: C:\Users\USER\Documents\Codex\2026-11-13\near-hackathon-engineering-continuation-handoff-continue-2\outputs\validated-engineering/validation-results.json and logs/. No public testnet/mainnet or real maintenance work is implied. C-10/C-11 real-user/real-technician adoption/workflow claims remain open beyond this local simulation.


## Final release — 14 November 2026

6 asset-account unit tests, 20 asset-registry unit tests and 3 isolated Sandbox integrations passed against the release artifacts. Logs and hash manifest: `evidence/`. Public testnet full maintenance lifecycle and handover passed; finalized RPC proof is `evidence/testnet-proof.json`. Hosted app: https://self-maintaining-asset.vercel.app . Wallet QA found the MyNearWallet retirement; Meteor replacement and browser acceptance remain in progress. GitHub publication, public demo post and submission are not yet claimed.


## Browser acceptance and clean CI

Meteor connection and real wallet-signed creation of qa-water-141126 passed. A cancelled deposit displayed an error, then a deliberately approved 0.05 NEAR deposit finalized. Operator observation, deterministic triage and work-order creation persisted on chain. Transaction links, browser proof limits and mobile tab results are in `evidence/browser-qa.json`. All four mobile tabs now work; no horizontal overflow at 390px. Current screenshots: `evidence/screenshots/`.

Clean Ubuntu CI run 43633555968 passed frontend build/lint and the full contract rebuild plus serial integrations. The first artifact upload omitted hidden .validation files; the upload setting is corrected for the next run. This did not affect test execution or its successful result.

The installed HyperFrames capture initially failed with Windows spawn EFTYPE; explicitly selecting official Windows Chrome repaired it, and a fresh URL capture succeeded. The failed partial capture is excluded from the film. Media production continues from real product captures. No X post or hackathon application has been submitted.
