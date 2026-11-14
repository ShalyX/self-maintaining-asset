import { Buffer } from "buffer";
(globalThis as unknown as {Buffer: typeof Buffer}).Buffer ??= Buffer;

const RPC_URL = (import.meta.env.VITE_RPC_URL as string | undefined)?.trim() || "https://rpc.testnet.near.org";
const GAS_100_TGAS = 100_000_000_000_000n;
const GAS_60_TGAS = 60_000_000_000_000n;
const PENDING_KEY = "asset-passport:pending-wallet-intent";
let selectorPromise: Promise<any> | null = null;
let sdkPromise: Promise<any> | null = null;

export function registryId(): string { return (import.meta.env.VITE_REGISTRY_ID as string | undefined)?.trim() ?? ""; }
export function isConfigured(): boolean { return registryId().length > 0; }
async function sdk() {
  sdkPromise ??= Promise.all([import("@near-wallet-selector/core"), import("@near-wallet-selector/meteor-wallet")])
    .then(([core, wallet]) => ({...core, ...wallet})).catch((e) => { sdkPromise = null; throw e; });
  return sdkPromise;
}
async function selector(): Promise<any> {
  selectorPromise ??= sdk().then(({setupWalletSelector, setupMeteorWallet}) => setupWalletSelector({network: "testnet", modules: [setupMeteorWallet()]}))
    .catch((e: unknown) => { selectorPromise = null; throw e; });
  return selectorPromise;
}
export async function accounts(): Promise<Array<{accountId: string}>> { return (await (await selector()).wallet()).getAccounts(); }
export async function connect(): Promise<void> {
  if (!registryId()) throw new Error("Registry contract is not configured yet.");
  await (await (await selector()).wallet("meteor-wallet")).signIn({contractId: registryId(), successUrl: window.location.href, failureUrl: window.location.href});
}
export async function disconnect(): Promise<void> { await (await (await selector()).wallet()).signOut(); }
function encodeArgs(args: unknown): string {
  let text = "";
  for (const byte of new TextEncoder().encode(JSON.stringify(args ?? {}))) text += String.fromCharCode(byte);
  return btoa(text);
}
async function rpc(method: string, params: unknown): Promise<any> {
  const response = await fetch(RPC_URL, {method: "POST", headers: {"content-type": "application/json"}, signal: AbortSignal.timeout(25000), body: JSON.stringify({jsonrpc: "2.0", id: "asset-passport", method, params})});
  if (!response.ok) throw new Error(`NEAR RPC returned HTTP ${response.status}`);
  const payload = await response.json();
  if (payload.error) throw new Error(`NEAR RPC ${method}: ${payload.error.message || JSON.stringify(payload.error.data || payload.error)}`);
  if (!payload.result) throw new Error(`NEAR RPC ${method} returned no result.`);
  return payload.result;
}
export async function viewAt<T = any>(accountId: string, methodName: string, args: Record<string, unknown> = {}): Promise<T> {
  const result = await rpc("query", {request_type: "call_function", finality: "final", account_id: accountId, method_name: methodName, args_base64: encodeArgs(args)});
  if (result.error) throw new Error(`${methodName}: ${result.error}`);
  if (!Array.isArray(result.result)) throw new Error(`${methodName} returned an invalid RPC response.`);
  const text = new TextDecoder().decode(new Uint8Array(result.result));
  return text ? JSON.parse(text) as T : null as T;
}
export class TransactionError extends Error {
  transactionHash: string;
  constructor(message: string, hash: string) { super(message); this.transactionHash = hash; }
}
export class WalletRedirectError extends Error {}
function clearPending() { try { sessionStorage.removeItem(PENDING_KEY); } catch { /* RPC state remains authoritative. */ } }
function readPending(): any { try { return JSON.parse(sessionStorage.getItem(PENDING_KEY) || "null"); } catch { return null; } }
export async function confirmTransaction(hash: string, signerId: string): Promise<void> {
  let outcome;
  try { outcome = await rpc("tx", {tx_hash: hash, sender_account_id: signerId, wait_until: "FINAL"}); }
  catch (e) { throw new TransactionError(`Transaction submitted, but confirmation is unavailable. Check its explorer record before retrying. ${e instanceof Error ? e.message : ""}`, hash); }
  const final = outcome.final_execution_outcome || outcome;
  if (!final.status || typeof final.status !== "object") throw new TransactionError("Transaction has no final outcome yet. Do not resubmit it; check its explorer record.", hash);
  const failure = final.status.Failure || final.receipts_outcome?.find((receipt: any) => receipt.outcome?.status?.Failure)?.outcome.status.Failure;
  if (failure) { clearPending(); throw new TransactionError(`An on-chain action failed. Refresh the passport to inspect recovery state. ${JSON.stringify(failure)}`, hash); }
  if (!("SuccessValue" in final.status)) throw new TransactionError("Transaction is still pending. Check its explorer record before retrying.", hash);
  clearPending();
}
export async function consumeWalletReturn(): Promise<string | null> {
  const url = new URL(window.location.href);
  const error = url.searchParams.get("errorCode");
  if (error) {
    clearPending();
    const message = url.searchParams.get("errorMessage") || error;
    url.searchParams.delete("errorCode"); url.searchParams.delete("errorMessage");
    window.history.replaceState({}, "", url);
    throw new Error(`Wallet did not approve the transaction: ${message}`);
  }
  const hash = url.searchParams.get("transactionHashes")?.split(",")[0];
  if (!hash) return null;
  const pending = readPending();
  const signerId = pending?.signerId || (await accounts())[0]?.accountId;
  if (!signerId) throw new TransactionError("Reconnect the signing wallet to confirm this submitted transaction. Do not submit it again.", hash);
  await confirmTransaction(hash, signerId);
  url.searchParams.delete("transactionHashes");
  window.history.replaceState({}, "", url);
  return hash;
}
type Call = {method: string; args: Record<string, unknown>; deposit: string; gas: bigint};
async function sendCalls(receiverId: string, calls: Call[]): Promise<string> {
  const wallet = await (await selector()).wallet();
  const signerId = (await wallet.getAccounts())[0]?.accountId;
  if (!signerId) throw new Error("Connect a NEAR testnet wallet first.");
  const pending = readPending();
  if (pending && Date.now() - pending.startedAt < 120000) throw new Error("A wallet request is already pending. Check wallet activity before sending another transaction.");
  const {actionCreators} = await sdk();
  try { sessionStorage.setItem(PENDING_KEY, JSON.stringify({signerId, receiverId, startedAt: Date.now(), methods: calls.map((call) => call.method)})); } catch { /* Wallet confirmation is still mandatory. */ }
  let result: any;
  try { result = await wallet.signAndSendTransaction({receiverId, actions: calls.map((call) => actionCreators.functionCall(call.method, call.args, call.gas, BigInt(call.deposit)))}); }
  catch (e) { clearPending(); throw e; }
  if (!result) throw new WalletRedirectError("Approve or reject the request in Meteor Wallet. The passport will verify the transaction when the wallet returns.");
  const hash = result.transaction?.hash;
  if (typeof hash !== "string" || !hash) throw new Error("The wallet returned no transaction hash. Check wallet activity before retrying.");
  await confirmTransaction(hash, signerId);
  return hash;
}
export async function callAt(receiverId: string, methodName: string, args: Record<string, unknown> = {}, depositYocto = "0", gas = GAS_60_TGAS): Promise<string> {
  return sendCalls(receiverId, [{method: methodName, args, deposit: depositYocto, gas}]);
}
export async function callRegistry(methodName: string, args: Record<string, unknown> = {}, depositYocto = "0", gas = GAS_100_TGAS): Promise<string> {
  if (!registryId()) throw new Error("Registry contract is not configured yet.");
  return callAt(registryId(), methodName, args, depositYocto, gas);
}
export async function transferPassport(tokenId: string, receiverId: string): Promise<string> {
  const [credit, topup] = await Promise.all([
    viewAt<string>(registryId(), "get_ownership_storage_balance", {token_id: tokenId}),
    viewAt<string>(registryId(), "get_ownership_storage_cost", {}),
  ]);
  const calls: Call[] = [];
  if (BigInt(credit) < BigInt(topup)) calls.push({method: "fund_ownership_storage", args: {token_id: tokenId}, deposit: topup, gas: 30_000_000_000_000n});
  calls.push({method: "nft_transfer", args: {token_id: tokenId, receiver_id: receiverId, approval_id: null, memo: "Operational passport handover"}, deposit: "1", gas: GAS_60_TGAS});
  return sendCalls(registryId(), calls);
}
export function parseNearAmount(value: string): string {
  const normalized = value.trim();
  if (!/^\d+(\.\d{0,24})?$/.test(normalized)) throw new Error("Enter a valid NEAR amount (up to 24 decimal places).");
  const [whole, fraction = ""] = normalized.split(".");
  return (BigInt(whole || "0") * 10n ** 24n + BigInt((fraction + "0".repeat(24)).slice(0, 24))).toString();
}
export function formatNear(value: string | number | bigint | null | undefined, digits = 4): string {
  if (value === undefined || value === null) return "—";
  try { const amount = BigInt(value); const base = 10n ** 24n; return `${(amount / base).toLocaleString("en-US")}.${(amount % base).toString().padStart(24, "0").slice(0, digits)}`; } catch { return "—"; }
}
export function formatAccount(value?: string | null): string {
  if (!value) return "Not connected";
  return value.length > 28 ? `${value.slice(0, 13)}…${value.slice(-10)}` : value;
}
