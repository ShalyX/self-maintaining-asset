# Judge walkthrough

Open [the live app](https://self-maintaining-asset.vercel.app). No wallet is needed to inspect the completed demo.

1. **Overview:** inspect the water asset, its dedicated account and current balances. The 1 NEAR deposit allocated 0.8 to operating funds and 0.2 to reserve; the approved payout left 0.1 in reserve.
2. **Maintenance:** open `WO-001`. Follow the recorded observation, ruleset diagnosis, assigned technician, 0.1 NEAR quote, owner approval and completion digest through to `PAID`.
3. **Asset passport:** inspect the persistent child account and service record.
4. **Handover:** inspect the initial owner and subsequent transfer. The owner is now `bobasset141126.testnet`; the account identity did not change. The former owner's attempted payout failed; the new owner's payout succeeded.
5. To try a fresh asset, connect a funded Meteor testnet wallet, enter a unique asset ID in **Open or register an asset**, and register it. The wallet displays the on-chain provisioning quote. Use a second wallet as the assigned technician for its quote and completion steps. Testnet tokens only.

The completion file is a labelled testnet fixture. No physical repair or sensor verification is claimed. The coordinator is a transparent ruleset; the current owner signs approval and payout.

## Public transaction proof

| Action | Result | Explorer |
|---|---|---|
| Revenue deposit (80/20 split) | Success | [HGM6DPXmN4…](https://testnet.nearblocks.io/txns/HGM6DPXmN4k6LX4GpJV52Gt8mj822p1ApgeT8CbqSnyX) |
| new | Success | [3oMobgskgY…](https://testnet.nearblocks.io/txns/3oMobgskgY1ZgpWTwBn1aznB8aYkpgeXuTBoRoDeNav1) |
| Asset creation and child provisioning | Success | [FNgWWefSXQ…](https://testnet.nearblocks.io/txns/FNgWWefSXQL4sNd3M7nGvSX3as477MFwVaSpb1MpisBL) |
| Observation and diagnosis | Success | [hxu4NGNsqL…](https://testnet.nearblocks.io/txns/hxu4NGNsqLit6xXHMVb3YSuC4qxmp27MHLr4Cmx126b) |
| Technician assignment | Success | [7VzpWN4nkg…](https://testnet.nearblocks.io/txns/7VzpWN4nkg5LkQaT6LLeKyQ8zRHz4EktpWXNtLsqvAd8) |
| Technician quote | Success | [315y157Yoo…](https://testnet.nearblocks.io/txns/315y157YooHVih1vkd5w9hvHa9hkakU8Virsqch47r85) |
| Owner quote approval | Success | [AyJCibtjzP…](https://testnet.nearblocks.io/txns/AyJCibtjzPRtXGMnn7h1kMsUuTirLY212EEJeLCVEJWp) |
| Completion evidence | Success | [9GPAtaitAu…](https://testnet.nearblocks.io/txns/9GPAtaitAuTGPCNxqwqnmqA5urVoqt6ZJtSVUP12v1h8) |
| Ownership handover | Success | [5y5bx9b3Fr…](https://testnet.nearblocks.io/txns/5y5bx9b3FrGHaioPo5JNZSNcYoehWSSHeG4WVxP4yjk8) |
| Completion confirmation and payout | Expected former-owner rejection | [2foDpm6VXV…](https://testnet.nearblocks.io/txns/2foDpm6VXVceYyGgpDFroP54QcLQn9dAFcedLG26xf2w) |
| Completion confirmation and payout | Success | [CGi7wQ4TNp…](https://testnet.nearblocks.io/txns/CGi7wQ4TNp3Nq6Dd3KQp2uikvoCb7tuCB8axTKXc8pLC) |

The raw finalized RPC outcomes, receipt results, account code hashes, access-key lists and final views are in [testnet-proof.json](evidence/testnet-proof.json). The child has zero access keys. Local artifact hashes are in [artifact-manifest.json](evidence/artifact-manifest.json).
