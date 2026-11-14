# Threat model — The Self-Maintaining Asset

## Security objective

An NFT represents operational authority over a physical asset's dedicated NEAR account. A sale must move that authority and the asset's balances and service history to the buyer without allowing the seller to retain spending power, redirect a pending payment or erase the record.

This is a testnet prototype, not an audited financial system.

## Assets and trust boundaries

- **Asset passport:** NEP-171 token ID and current owner mapping.
- **Child account:** keyless asset-specific sub-account holding operating funds and the maintenance reserve.
- **Reserve:** the portion of deposits reserved for approved maintenance.
- **Work order:** observation, diagnosis source, assigned technician, quote, owner approval, completion report, evidence digest and payout state.
- **Ownership history:** append-only mint and transfer events keyed to the token ID.
- **Actors:** current owner, former owner/seller, assigned technician, registry contract, child-account contract and read-only UI/RPC.

The coordinator is not trusted with authority. It has no wallet key and cannot approve work, transfer the NFT or release funds.

## Threats and controls

| Threat / attacker | Control |
|---|---|
| Seller keeps a second key to the asset account | The registry creates the child account without granting an external full-access key. The child account exposes no owner withdrawal method. |
| Seller drains the maintenance reserve before sale | The child contract separates operating balance from maintenance reserve. Only the registry can call the reserve-release method; there is no arbitrary owner withdrawal. |
| Seller transfers the NFT after a payout is initiated | The registry records a pending lock before scheduling the cross-contract payout. The recipient and amount are already fixed to the assigned technician and approved quote. NFT transfer does not clear or redirect that payout; the callback settles the same work order under the same token ID. A dedicated test covers handover during a pending payout. |
| Seller or buyer replays a payout | Pending and paid work-order locks plus explicit lifecycle states reject duplicate or concurrent payouts. |
| Seller redirects funds to a related account | The registry requires the payout recipient to equal the assigned technician and the amount to equal the approved quote. |
| Technician changes a quote after approval | Only the assigned technician can quote; only the current owner can approve; state transitions prevent quote resubmission after approval. |
| Technician submits completion without evidence | Completion requires a 64-character SHA-256 digest and a reference. The current owner must still confirm payment. |
| Operator presents a fake repair photo or report | The digest proves file integrity, not real-world truth. The UI labels operator input and demo fixtures clearly; no sensor verification or physical repair is claimed. |
| Transfer fails or callback output is malformed | A definite child-transfer failure restores the reserve and can be retried. An unknown or malformed result retains the replay lock for reconciliation. |
| Former owner claims authority after sale | Every new work-order assignment, quote approval, payout confirmation and NFT transfer checks the current owner mapping. |
| Ownership history is altered by the seller | The registry appends ownership events on mint and transfer. There is no public method to edit or delete those events. |
| UI or RPC shows stale information | The UI reads finalised testnet state and offers an explicit refresh. Contract authorization is authoritative; the UI is not the security boundary. |

## Residual risks and explicit limits

- The registry deployment account retains code-deployment authority on testnet. A production deployment should use a controlled upgrade policy or remove/restrict that authority after deployment.
- The contracts have not undergone an independent security audit and must not be used with mainnet funds.
- An unknown asynchronous payout outcome intentionally remains locked; manual reconciliation is required rather than risking a duplicate payment.
- A SHA-256 evidence digest does not prove that a physical inspection or repair occurred. The referenced file may remain in a browser session or repository; it is not automatically stored on decentralized storage.
- There is no live IoT telemetry, oracle, certified diagnosis, NGN payment rail or autonomous spending.
- NFT transfer changes operational control only. It does not transfer legal title to the physical borehole or water point.
- The prototype implements the core NEP-171 transfer path; full nft_transfer_call/resolver behavior is not claimed.

## Verification required before submission

1. Contract unit tests cover reserve restoration, successful payout callback, malformed/unknown callback locking and ownership transfer while payout is pending.
2. Sandbox integration tests cover real child-account deployment, revenue split, work-order lifecycle, reserve payout, transfer and former-owner rejection.
3. A public testnet run proves the same asset account and work-order history remain after handover.
4. UI build and lint pass; evidence and demo data remain explicitly labelled.
