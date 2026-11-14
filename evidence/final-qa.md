# Final production QA — 14 November 2026, 21:15 WAT

Target: https://self-maintaining-asset.vercel.app

- Production deployment rebuilt from commit de745a1 and promoted; the alias serves bundle `assets/index-D-C5GzJJ.js`, identical to the newest deployment URL.
- Desktop (1440×900): Overview, Maintenance, Asset passport and Handover all render live chain data — 0.1 NEAR reserve, 0.8 NEAR operating, WO-001 PAID with completion digest, owner `bobasset141126.testnet`, child `lagos-water-01.assetops141126.testnet`.
- Mobile (390×844): all four workspace tabs present in the collapsed rail; content stacks without horizontal overflow.
- Screenshots in `evidence/screenshots-final/` were captured in this pass from the production URL.

Not covered by this pass: wallet-signed transactions. Those were exercised earlier and are recorded in `evidence/browser-qa.json` and `evidence/testnet-proof.json`.
