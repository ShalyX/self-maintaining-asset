# Approved product coverage

| Locked capability | Release evidence | Scope |
|---|---|---|
| Wallet and asset creation | Browser-created qa-water-141126; finalized receipt in browser-qa.json | Real Meteor testnet wallet |
| Revenue and reserve allocation | Main demo 1 NEAR split 80/20; browser QA 0.05 NEAR deposit | Real child-account balances |
| Bounded maintenance coordination | Browser observation → transparent-ruleset-v1 diagnosis → on-chain work order | Operator input; no sensors or LLM |
| Persistent work-order lifecycle | WO-001 diagnosis, technician, quote, approval, completion and PAID final state | Full signed testnet run |
| Human payout authority | Current owner exact 0.1 NEAR payment; former owner rejection | Contract enforcement, no autonomous spend |
| Passport and service history | Final deployed UI screenshots and views | Same child, balances, work order and digest |
| NFT handover and continuity | Alice → Bob, preserved child, correct final payout authority | Testnet plus serial Sandbox |
| Rebuild and validation | Two clean Ubuntu CI runs, 26 unit and 3 serial integration tests | Cross-environment byte identity remains unverified |

Browser acceptance explicitly covers wallet connection, creation, cancellation, deposit, triage and work-order creation. The other role actions were executed through the signed testnet runner and their final states verified in the UI; all role buttons were not exercised end to end in the browser. This boundary is recorded rather than represented as complete browser acceptance.

The implementation and integrated on-chain journey are verified. Judge-ready publication is still gated on final demo QA, a human-published X demo URL and exact application consent. No mainnet readiness is claimed.
