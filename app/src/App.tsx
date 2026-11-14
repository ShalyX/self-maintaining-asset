import { consumeWalletReturn, transferPassport, TransactionError, WalletRedirectError } from "./near";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  accounts,
  callAt,
  callRegistry,
  connect,
  disconnect,
  formatAccount,
  formatNear,
  isConfigured,
  parseNearAmount,
  registryId,
  viewAt,
} from "./near";
import { triageIssue, type TriageResult } from "./triage";
import "./App.css";

type Asset = {
  token_id: string;
  name: string;
  asset_type: string;
  asset_account_id: string;
  reserve_bps: number;
  created_by: string;
  created_at_ns: number;
  status: string;
};

type Balances = {
  token_id: string;
  reserve_bps: number;
  total_deposited: string;
  operating_balance: string;
  maintenance_reserve: string;
  actual_account_balance: string;
};

type OwnershipEvent = {
  owner_id: string;
  previous_owner_id?: string | null;
  timestamp_ns: number;
  memo?: string | null;
};

type WorkOrder = {
  work_order_id: string;
  token_id: string;
  observation: string;
  observation_evidence_hash?: string | null;
  diagnosis: string;
  diagnosis_source: string;
  recommended_action: string;
  priority: string;
  created_by: string;
  created_at_ns: number;
  status: string;
  technician_id?: string | null;
  quote_amount_yocto?: string | null;
  quote_note?: string | null;
  approved_by?: string | null;
  completion_note?: string | null;
  completion_evidence_hash?: string | null;
  completion_evidence_uri?: string | null;
  paid_amount_yocto?: string | null;
};

const DEFAULT_TOKEN_ID = (import.meta.env.VITE_DEMO_TOKEN_ID as string | undefined) || "lagos-water-01";
const TOKEN_ID = new URL(window.location.href).searchParams.get("asset") || DEFAULT_TOKEN_ID;
const ASSET_NAME = (import.meta.env.VITE_DEMO_ASSET_NAME as string | undefined) || "Lagos Solar Water Point 01";
const ASSET_TYPE = "solar_water_point";

function shortHash(value?: string | null) {
  if (!value) return "Not recorded";
  return `${value.slice(0, 12)}…${value.slice(-8)}`;
}

function statusClass(status?: string) {
  const value = (status || "unknown").toLowerCase().replaceAll("_", "-");
  if (["active", "paid", "approved"].includes(value)) return "status status-good";
  if (["failed", "payment-failed", "payout-unknown", "provisioning-unknown"].includes(value)) return "status status-warn";
  return "status status-neutral";
}

function formatDate(ns?: number) {
  if (!ns) return "—";
  return new Date(ns / 1_000_000).toLocaleString("en-NG", {
    day: "2-digit",
    month: "short",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function yocto(value: string | number | undefined) {
  if (value === undefined) return "0";
  return String(value);
}

async function hashFile(file: File): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", await file.arrayBuffer());
  return Array.from(new Uint8Array(digest)).map((b) => b.toString(16).padStart(2, "0")).join("");
}

async function fileDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = () => reject(new Error("Could not read the selected evidence file."));
    reader.readAsDataURL(file);
  });
}

function saveLocalEvidence(hash: string, dataUrl: string) {
  try {
    if (dataUrl.length < 2_000_000) sessionStorage.setItem(`asset-evidence:${hash}`, dataUrl);
  } catch {
    // The on-chain digest remains the durable reference if browser storage is full.
  }
}

function readLocalEvidence(hash?: string | null) {
  if (!hash) return "";
  try {
    return sessionStorage.getItem(`asset-evidence:${hash}`) || "";
  } catch {
    return "";
  }
}

function App() {
  const [accountId, setAccountId] = useState("");
  const [assetInput, setAssetInput] = useState(TOKEN_ID);
  const [newAssetName, setNewAssetName] = useState(TOKEN_ID === DEFAULT_TOKEN_ID ? ASSET_NAME : "");
  const [asset, setAsset] = useState<Asset | null>(null);
  const [ownerId, setOwnerId] = useState("");
  const [balances, setBalances] = useState<Balances | null>(null);
  const [workOrders, setWorkOrders] = useState<WorkOrder[]>([]);
  const [ownershipHistory, setOwnershipHistory] = useState<OwnershipEvent[]>([]);
  const [activeTab, setActiveTab] = useState<"overview" | "maintenance" | "passport" | "handover">("overview");
  const [loading, setLoading] = useState(false);
  const [busy, setBusy] = useState("");
  const actionInFlight = useRef(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [observation, setObservation] = useState("");
  const [issueFile, setIssueFile] = useState<File | null>(null);
  const [issueHash, setIssueHash] = useState("");
  const [triage, setTriage] = useState<TriageResult | null>(null);
  const [technicianId, setTechnicianId] = useState("");
  const [quoteAmount, setQuoteAmount] = useState("0.12");
  const [quoteNote, setQuoteNote] = useState("Replace the worn pump seal, inspect the coupling and run a delivery test.");
  const [completionNote, setCompletionNote] = useState("Pump seal replaced; coupling inspected and delivery test completed.");
  const [completionFile, setCompletionFile] = useState<File | null>(null);
  const [completionPreview, setCompletionPreview] = useState("");
  const [revenueAmount, setRevenueAmount] = useState("1");
  const [transferTo, setTransferTo] = useState("");
  const [lastTx, setLastTx] = useState("");

  const configured = isConfigured();
  const isOwner = Boolean(accountId && ownerId && accountId === ownerId);
  const selectedWorkOrder = workOrders[0] ?? null;
  const activeCount = workOrders.filter((w) => !["PAID", "CANCELLED"].includes(w.status)).length;

  const refresh = useCallback(async () => {
    if (!configured) return;
    setLoading(true);
    try {
      const id = registryId();
      const [assetData, currentOwner, orders, ownership] = await Promise.all([
        viewAt<Asset | null>(id, "get_asset", { token_id: TOKEN_ID }),
        viewAt<string | null>(id, "get_asset_owner", { token_id: TOKEN_ID }),
        viewAt<WorkOrder[]>(id, "get_asset_work_orders", { token_id: TOKEN_ID }),
        viewAt<OwnershipEvent[]>(id, "get_asset_ownership_history", { token_id: TOKEN_ID }),
      ]);
      setAsset(assetData);
      setOwnerId(currentOwner || "");
      setWorkOrders(Array.isArray(orders) ? [...orders].reverse() : []);
      setOwnershipHistory(Array.isArray(ownership) ? ownership : []);
      if (assetData?.asset_account_id && assetData.status === "ACTIVE") {
        const balanceData = await viewAt<Balances>(assetData.asset_account_id, "get_balances", {});
        setBalances(balanceData);
      } else {
        setBalances(null);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not read the asset from NEAR testnet.");
    } finally {
      setLoading(false);
    }
  }, [configured]);

  const syncWallet = useCallback(async () => {
    try {
      const connected = await accounts();
      setAccountId(connected[0]?.accountId || "");
    } catch {
      setAccountId("");
    }
  }, []);

  useEffect(() => {
    let active = true;
    void (async () => {
      try {
        const connected = await accounts();
        if (active) setAccountId(connected[0]?.accountId || "");
        const hash = await consumeWalletReturn();
        if (active && hash) {
          setLastTx(hash);
          setNotice("Wallet transaction confirmed on NEAR testnet. The passport shows its resulting state.");
          await refresh();
        }
      } catch (e) {
        if (active && e instanceof TransactionError) { setLastTx(e.transactionHash); setError(e.message); }
        else if (active && e instanceof Error && e.message.startsWith("Wallet did not approve")) setError(e instanceof Error ? e.message : "Wallet declined the request.");
      }
    })();
    void Promise.resolve().then(() => refresh());
    return () => { active = false; };
  }, [refresh]);

  const evidencePreview = selectedWorkOrder
    ? selectedWorkOrder.completion_evidence_hash
      ? readLocalEvidence(selectedWorkOrder.completion_evidence_hash)
      : ""
    : completionPreview;

  const runAction = useCallback(async (label: string, action: () => Promise<string>) => {
    if (actionInFlight.current) return false;
    actionInFlight.current = true;
    setBusy(label);
    setLastTx("");
    setError("");
    setNotice("");
    try {
      const tx = await action();
      setLastTx(tx);
      setNotice(`${label}: transaction confirmed on NEAR testnet. Review the resulting on-chain state.`);
      window.setTimeout(() => void refresh(), 1800);
      return true;
    } catch (e) {
      if (e instanceof WalletRedirectError) setNotice(e.message);
      else {
        if (e instanceof TransactionError) setLastTx(e.transactionHash);
        setError(e instanceof Error ? e.message : `${label} failed.`);
      }
      return false;
    } finally {
      actionInFlight.current = false;
      setBusy("");
    }
  }, [refresh]);

  async function handleConnect() {
    setBusy("Connecting wallet");
    setError("");
    try {
      await connect();
      await syncWallet();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Wallet connection failed.");
    } finally {
      setBusy("");
    }
  }

  async function handleDisconnect() {
    setBusy("Disconnecting");
    try {
      await disconnect();
      setAccountId("");
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not disconnect wallet.");
    } finally {
      setBusy("");
    }
  }

  async function handleCreateAsset() {
    if (!accountId) return setError("Connect the asset owner's testnet wallet first.");
    if (!newAssetName.trim()) return setError("Enter a name for this asset before registering its passport.");
    await runAction("Register asset passport", async () => {
      const cost = await viewAt<string>(registryId(), "get_asset_creation_cost", {});
      return callRegistry("create_asset", {
        token_id: TOKEN_ID,
        name: newAssetName.trim(),
        asset_type: ASSET_TYPE,
        reserve_bps: 2000,
      }, yocto(cost), 100_000_000_000_000n);
    });
  }

  async function handleRetryProvisioning() {
    await runAction("Recover asset account", async () => {
      const cost = await viewAt<string>(registryId(), "get_asset_retry_cost", {token_id: TOKEN_ID});
      return callRegistry("retry_asset_provisioning", {token_id: TOKEN_ID}, cost);
    });
  }

  async function handleDeposit() {
    if (!asset) return;
    await runAction("Fund operating account", () =>
      callAt(asset.asset_account_id, "deposit_revenue", {}, parseNearAmount(revenueAmount)),
    );
  }

  async function handlePrepareTriage() {
    if (observation.trim().length < 10) {
      setError("Describe the observed issue in at least 10 characters.");
      return;
    }
    setError("");
    try {
      const result = triageIssue(observation);
      const file = issueFile;
      if (file) {
        const hash = await hashFile(file);
        const dataUrl = await fileDataUrl(file);
        saveLocalEvidence(hash, dataUrl);
        setIssueHash(hash);
      } else {
        setIssueHash("");
      }
      setTriage(result);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not prepare the maintenance recommendation.");
    }
  }

  async function handleCreateWorkOrder() {
    if (!triage) return setError("Run the maintenance coordinator before creating a work order.");
    const id = `WO-${Date.now().toString(36).toUpperCase()}`;
    const ok = await runAction("Create maintenance work order", async () => {
      const cost = await viewAt<string>(registryId(), "get_work_order_creation_cost", {});
      return callRegistry("create_work_order", {
        token_id: TOKEN_ID,
        work_order_id: id,
        observation,
        observation_evidence_hash: issueHash || null,
        diagnosis: triage.diagnosis,
        diagnosis_source: triage.source,
        recommended_action: triage.recommendedAction,
        priority: triage.priority,
      }, yocto(cost), 100_000_000_000_000n);
    });
    if (ok) {
      setObservation("");
      setTriage(null);
      setIssueFile(null);
      setIssueHash("");
      setActiveTab("maintenance");
    }
  }

  async function handleAssignTechnician(order: WorkOrder) {
    if (!technicianId.trim()) return setError("Enter the technician's NEAR testnet account.");
    await runAction("Assign technician", () =>
      callRegistry("assign_technician", {
        token_id: TOKEN_ID,
        work_order_id: order.work_order_id,
        technician_id: technicianId.trim(),
      }),
    );
  }

  async function handleSubmitQuote(order: WorkOrder) {
    await runAction("Submit technician quote", () =>
      callRegistry("submit_technician_quote", {
        token_id: TOKEN_ID,
        work_order_id: order.work_order_id,
        quote_amount: parseNearAmount(quoteAmount),
        quote_note: quoteNote,
      }),
    );
  }

  async function handleApproveQuote(order: WorkOrder) {
    await runAction("Approve technician quote", () =>
      callRegistry("approve_work_order_quote", {
        token_id: TOKEN_ID,
        work_order_id: order.work_order_id,
      }),
    );
  }

  async function handleSubmitCompletion(order: WorkOrder) {
    const file = completionFile;
    if (!file) return setError("Attach a completion photo or checklist before submitting evidence.");
    if (completionNote.trim().length < 5) return setError("Add a short completion report.");
    let dataUrl = "";
    const ok = await runAction("Submit completion evidence", async () => {
      const hash = await hashFile(file);
      dataUrl = await fileDataUrl(file);
      saveLocalEvidence(hash, dataUrl);
      return callRegistry("submit_work_order_completion", {
        token_id: TOKEN_ID,
        work_order_id: order.work_order_id,
        completion_note: completionNote,
        completion_evidence_hash: hash,
        completion_evidence_uri: `browser-local://sha256/${hash}`,
      });
    });
    if (ok) setCompletionPreview(dataUrl);
  }

  async function handleConfirmPayment(order: WorkOrder) {
    const reserve = BigInt(balances?.maintenance_reserve ?? "0");
    const approvedQuote = BigInt(order.quote_amount_yocto ?? "0");
    if (reserve < approvedQuote) {
      setError(`Insufficient maintenance reserve. The asset has ${formatNear(reserve)} NEAR available, but the approved quote is ${formatNear(approvedQuote)} NEAR. Fund the asset account before confirming payment.`);
      return;
    }
    await runAction("Confirm completion and pay technician", async () => {
      const cost = await viewAt<string>(registryId(), "get_maintenance_payout_cost", {});
      return callRegistry("confirm_work_order_completion", {
        token_id: TOKEN_ID,
        work_order_id: order.work_order_id,
      }, yocto(cost), 100_000_000_000_000n);
    });
  }

  async function handleTransfer() {
    if (!transferTo.trim()) return setError("Enter the receiving NEAR testnet account.");
    await runAction("Transfer asset passport", () => transferPassport(TOKEN_ID, transferTo.trim()));
  }

  const reservePercent = useMemo(() => Math.max(0, Math.min(100, asset?.reserve_bps ? asset.reserve_bps / 100 : 0)), [asset?.reserve_bps]);

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand-lockup">
          <div className="brand-mark"><span>SA</span><i /></div>
          <div><strong>SELF-MAINTAINING ASSET</strong><small>OPERATING PASSPORT</small></div>
        </div>
        <div className="workspace-label">WORKSPACE</div>
        <nav className="side-nav">
          <button className={activeTab === "overview" ? "nav-item active" : "nav-item"} onClick={() => setActiveTab("overview")}><span className="nav-icon">⌂</span>Overview</button>
          <button className={activeTab === "maintenance" ? "nav-item active" : "nav-item"} onClick={() => setActiveTab("maintenance")}><span className="nav-icon">↗</span>Maintenance <em>{activeCount || ""}</em></button>
          <button className={activeTab === "passport" ? "nav-item active" : "nav-item"} onClick={() => setActiveTab("passport")}><span className="nav-icon">▤</span>Asset passport</button>
          <button className={activeTab === "handover" ? "nav-item active" : "nav-item"} onClick={() => setActiveTab("handover")}><span className="nav-icon">⇄</span>Handover</button>
        </nav>
        <div className="sidebar-bottom">
          <div className="network-card"><span className="pulse-dot" />NEAR TESTNET <small>Prototype environment</small></div>
          <div className="sidebar-foot"><span>Built for continuity.</span><span>v0.1 · TESTNET</span></div>
        </div>
      </aside>

      <main className="main-area">
        <header className="topbar">
          <div className="breadcrumb"><span>ASSET OPERATIONS</span><b>/</b><strong>{activeTab === "overview" ? "Overview" : activeTab === "maintenance" ? "Maintenance" : activeTab === "passport" ? "Asset passport" : "Handover"}</strong></div>
          <div className="topbar-actions">
            <span className="network-pill"><span className="pulse-dot" />TESTNET</span>
            {accountId ? (
              <button className="wallet-button connected" onClick={handleDisconnect}><span className="wallet-avatar">{accountId.slice(0, 1).toUpperCase()}</span>{formatAccount(accountId)} <span className="chevron">⌄</span></button>
            ) : (
              <button className="wallet-button" onClick={handleConnect} disabled={!configured || Boolean(busy)}>{busy === "Connecting wallet" ? "Connecting…" : "Connect wallet"}</button>
            )}
          </div>
        </header>

        <div className="content-wrap">
          <section className="page-heading">
            <div>
              <div className="eyebrow">PHYSICAL ASSET · FINANCIAL CONTINUITY</div>
              <h1>{activeTab === "overview" ? "Asset operations" : activeTab === "maintenance" ? "Maintenance desk" : activeTab === "passport" ? "Operational passport" : "Authority handover"}</h1>
              <p>{activeTab === "overview" ? "One asset. One account. A maintenance history that stays with it." : activeTab === "maintenance" ? "Coordinate repairs with a human approval gate at every spend." : activeTab === "passport" ? "The asset's account, reserve and service history in one persistent record." : "Transfer operational control without leaving the asset's financial history behind."}</p>
            </div>
            <button className="refresh-button" onClick={() => void refresh()} disabled={loading || !configured}><span className={loading ? "spin" : ""}>↻</span> Refresh chain data</button>
          </section>

          <form className="asset-switcher panel" onSubmit={(event) => { event.preventDefault(); const id = assetInput.trim(); if (!/^[a-z0-9][a-z0-9-]{1,22}[a-z0-9]$/.test(id)) { setError("Use an asset ID of 3–24 lowercase letters, numbers or hyphens, starting and ending with a letter or number."); return; } const url = new URL(window.location.href); url.searchParams.set("asset", id); window.location.assign(url.toString()); }}>
            <label htmlFor="asset-id-picker">OPEN OR REGISTER AN ASSET</label>
            <input id="asset-id-picker" value={assetInput} onChange={(event) => setAssetInput(event.target.value)} maxLength={24} required aria-label="Asset ID" />
            <button className="refresh-button" disabled={!!busy}>Open passport</button>
            {TOKEN_ID !== DEFAULT_TOKEN_ID && <a href={window.location.pathname}>View demo passport</a>}
          </form>

          {activeTab === "overview" && (
            <section className="story-panel panel">
              <div className="story-intro">
                <div className="eyebrow">WHY THIS ASSET HAS A PASSPORT</div>
                <h2>When the operator changes, the asset should keep its money and memory.</h2>
                <p>Maintenance funds, repair approvals and service records often live in separate accounts, chats and spreadsheets. This passport binds the operating account, protected reserve and work-order history to one persistent asset identity.</p>
              </div>
              <div className="story-cards">
                <div className="story-card"><span>01 / PROBLEM</span><strong>Fragmented upkeep</strong><p>Funds and service records can be separated from the physical asset when an operator or contractor changes.</p></div>
                <div className="story-card"><span>02 / SOLUTION</span><strong>Asset-bound continuity</strong><p>A NEP-171 passport anchors a dedicated account, deterministic reserve split and persistent maintenance history.</p></div>
                <div className="story-card"><span>03 / VALUE</span><strong>Safer handover</strong><p>The next owner inherits the same account and service record. The previous owner loses approval authority.</p></div>
              </div>
              <div className="story-flow"><span>OBSERVE</span><b>→</b><span>TECHNICIAN QUOTE</span><b>→</b><span>OWNER APPROVAL</span><b>→</b><span>COMPLETION EVIDENCE</span><b>→</b><span>RESERVE PAYOUT</span></div>
              <div className="story-note"><span>PROTOTYPE BOUNDARY</span><p>The coordinator uses a transparent ruleset and operator-supplied observations. It does not read live sensors, certify a physical repair or spend autonomously. Every payout requires the current NFT owner's signed approval.</p></div>
            </section>
          )}

          {!configured && <div className="notice notice-warn"><strong>Registry not configured.</strong> Set VITE_REGISTRY_ID in app/.env.local after the registry is deployed to NEAR testnet. No sample balances or fake chain data are shown.</div>}
          {error && <div className="notice notice-error"><strong>Action needs attention</strong><span>{error}</span>{lastTx && <a href={`https://testnet.nearblocks.io/txns/${lastTx}`} target="_blank" rel="noreferrer">Inspect transaction before retrying ↗</a>}<button onClick={() => setError("")}>Dismiss</button></div>}
          {notice && <div className="notice notice-success"><strong>{notice}</strong>{lastTx && <a href={`https://testnet.nearblocks.io/txns/${lastTx}`} target="_blank" rel="noreferrer">View transaction ↗</a>}<button onClick={() => setNotice("")}>Dismiss</button></div>}

          {configured && !asset && !loading && !error && (
            <section className="empty-asset panel">
              <div className="empty-orb"><span>⌁</span></div>
              <div className="eyebrow">NO ASSET PASSPORT FOUND</div>
              <h2>Give this water point a persistent identity.</h2>
              <p>Registering creates the asset NFT and its dedicated keyless account. The creator funds provisioning; the registry does not subsidize it.</p>
              <label className="new-asset-name">Asset name<input aria-label="New asset name" value={newAssetName} maxLength={128} onChange={(event) => setNewAssetName(event.target.value)} placeholder="e.g. Community Solar Water Point" /></label>
              <div className="empty-meta"><span>Token ID <b>{TOKEN_ID}</b></span><span>Reserve policy <b>20% of deposits</b></span></div>
              <button className="primary-button" onClick={() => void handleCreateAsset()} disabled={!accountId || Boolean(busy)}>{busy || "Register asset passport"} <span>→</span></button>
              {!accountId && <small>Connect the intended owner wallet to register the asset.</small>}
            </section>
          )}

          {asset?.status === "FAILED" && <div className="notice notice-warn"><strong>Asset account provisioning failed.</strong><span>The NFT and its funding record are preserved. The current owner can recover it using the registry's exact retry quote.</span><button disabled={!isOwner || Boolean(busy)} onClick={() => void handleRetryProvisioning()}>Recover asset account</button></div>}
          {asset?.status === "PROVISIONING_UNKNOWN" && <div className="notice notice-warn">Provisioning outcome needs reconciliation. Automatic retry is blocked to protect the asset's funds.</div>}
          {asset && activeTab === "overview" && (
            <>
              <section className="stat-grid">
                <StatCard label="Maintenance reserve" value={formatNear(balances?.maintenance_reserve)} suffix="NEAR" detail={`${reservePercent.toFixed(0)}% allocation policy`} tone="green" icon="◈" />
                <StatCard label="Operating balance" value={formatNear(balances?.operating_balance)} suffix="NEAR" detail="Available operating funds" tone="blue" icon="↗" />
                <StatCard label="Total deposited" value={formatNear(balances?.total_deposited)} suffix="NEAR" detail="Recorded by asset account" tone="sand" icon="＋" />
                <StatCard label="Open work orders" value={String(activeCount)} suffix="" detail={`${workOrders.length} recorded in passport`} tone="violet" icon="▤" />
              </section>

              <div className="overview-grid">
                <section className="panel asset-panel">
                  <div className="panel-head"><div><div className="eyebrow">ASSET PASSPORT</div><h2>{asset.name}</h2></div><span className={statusClass(asset.status)}>{asset.status}</span></div>
                  <div className="asset-illustration">
                    <div className="sun-disc" /><div className="water-line water-line-one" /><div className="water-line water-line-two" />
                    <div className="asset-art-label"><span>FIELD ASSET / 001</span><strong>Solar water point</strong></div>
                    <div className="asset-glyph">⌁</div>
                  </div>
                  <div className="asset-details">
                    <div><small>ASSET ID</small><strong>{asset.token_id}</strong></div>
                    <div><small>ASSET ACCOUNT</small><strong className="mono">{formatAccount(asset.asset_account_id)}</strong></div>
                    <div><small>CURRENT OWNER</small><strong className="mono">{formatAccount(ownerId)}</strong></div>
                    <div><small>RESERVE POLICY</small><strong>{reservePercent.toFixed(0)}% of deposits</strong></div>
                  </div>
                  <div className="panel-foot"><span>Created {formatDate(asset.created_at_ns)}</span><button className="text-button" onClick={() => setActiveTab("passport")}>View full passport ↗</button></div>
                </section>

                <section className="panel coordinator-panel">
                  <div className="panel-head"><div><div className="eyebrow">MAINTENANCE COORDINATOR</div><h2>Keep the asset in service.</h2></div><span className="coordinator-mark">✳</span></div>
                  <p className="muted">Turn an operator observation into a structured work order. The coordinator suggests; the owner decides.</p>
                  <label className="field-label" htmlFor="issue-observation">WHAT DID YOU OBSERVE?</label>
                  <textarea id="issue-observation" value={observation} onChange={(e) => setObservation(e.target.value)} placeholder="e.g. The pump stopped delivering water after the inverter showed a low-voltage fault…" rows={4} />
                  <div className="file-row"><label className="file-button">＋ Attach observation photo<input type="file" accept="image/*,.pdf,.txt" onChange={(e) => setIssueFile(e.target.files?.[0] || null)} /></label><span>{issueFile?.name || "Optional evidence"}</span></div>
                  {triage ? (
                    <div className="triage-result">
                      <div className="triage-top"><span className="eyebrow">COORDINATOR DRAFT</span><span className={statusClass(triage.priority)}>{triage.priority}</span></div>
                      <strong>{triage.diagnosis}</strong><p>{triage.recommendedAction}</p>
                      <small>Source: transparent ruleset v1 · Not sensor-verified · Human approval required</small>
                      <div className="triage-actions"><button className="primary-button" onClick={() => void handleCreateWorkOrder()} disabled={!isOwner || Boolean(busy)}>Create on-chain work order <span>→</span></button><button className="text-button" onClick={() => setTriage(null)}>Edit observation</button></div>
                    </div>
                  ) : (
                    <button className="secondary-button full-width" onClick={() => void handlePrepareTriage()} disabled={!observation.trim() || Boolean(busy)}>Prepare maintenance recommendation <span>→</span></button>
                  )}
                  <div className="guardrail-note"><span>◈</span><p>The coordinator has no wallet key and cannot approve or release funds. Every payout requires the current NFT owner's signed transaction.</p></div>
                </section>
              </div>

              <section className="panel treasury-panel">
                <div className="panel-head"><div><div className="eyebrow">ASSET TREASURY</div><h2>Fund the work before the breakdown.</h2></div><span className="status status-good">ON-CHAIN</span></div>
                <div className="treasury-body">
                  <div className="reserve-visual"><div className="reserve-ring" style={{ background: `conic-gradient(#4d9161 0 ${reservePercent * 3.6}deg, #e6eee6 ${reservePercent * 3.6}deg 360deg)` }}><div><strong>{reservePercent.toFixed(0)}%</strong><small>RESERVE RATE</small></div></div><div><strong>{formatNear(balances?.maintenance_reserve)} NEAR</strong><small>Protected maintenance reserve</small></div></div>
                  <div className="treasury-info"><div className="balance-row"><span>Operating funds</span><strong>{formatNear(balances?.operating_balance)} NEAR</strong></div><div className="balance-row"><span>Maintenance reserve</span><strong>{formatNear(balances?.maintenance_reserve)} NEAR</strong></div><div className="balance-row"><span>Actual child account balance</span><strong>{formatNear(balances?.actual_account_balance)} NEAR</strong></div><div className="deposit-control"><label className="field-label" htmlFor="revenue-amount">TESTNET REVENUE DEPOSIT</label><div className="input-action"><input id="revenue-amount" value={revenueAmount} onChange={(e) => setRevenueAmount(e.target.value)} inputMode="decimal" /><span>NEAR</span><button className="primary-button" onClick={() => void handleDeposit()} disabled={!accountId || Boolean(busy)}>Deposit</button></div><small>Anyone can fund the asset account; the contract applies the split. Testnet NEAR only.</small></div></div>
                </div>
              </section>
              <WorkOrderList orders={workOrders.slice(0, 3)} accountId={accountId} ownerId={ownerId} onOpen={() => setActiveTab("maintenance")} />
            </>
          )}

          {asset && activeTab === "maintenance" && (
            <section className="maintenance-layout">
              <div className="panel">
                <div className="panel-head"><div><div className="eyebrow">WORK ORDER REGISTER</div><h2>Maintenance lifecycle</h2></div><span className="status status-neutral">{workOrders.length} TOTAL</span></div>
                {workOrders.length ? workOrders.map((order) => (
                  <WorkOrderCard key={order.work_order_id} order={order} accountId={accountId} ownerId={ownerId} technicianId={technicianId} setTechnicianId={setTechnicianId} quoteAmount={quoteAmount} setQuoteAmount={setQuoteAmount} quoteNote={quoteNote} setQuoteNote={setQuoteNote} completionNote={completionNote} setCompletionNote={setCompletionNote} completionFile={completionFile} setCompletionFile={setCompletionFile} onAssign={() => void handleAssignTechnician(order)} onQuote={() => void handleSubmitQuote(order)} onApprove={() => void handleApproveQuote(order)} onComplete={() => void handleSubmitCompletion(order)} onPay={() => void handleConfirmPayment(order)} />
                )) : <EmptyState title="No maintenance work orders yet" body="Start with an operator-supplied observation in the overview. The coordinator will prepare a structured draft before it is written to NEAR." action={() => setActiveTab("overview")} actionLabel="Create first work order" />}
              </div>
              <aside className="panel policy-panel"><div className="eyebrow">SPEND POLICY</div><h3>Human approval is the last mile.</h3><PolicyStep number="01" title="Observe" text="The operator records the issue. No synthetic telemetry." /><PolicyStep number="02" title="Quote" text="The assigned technician submits a signed quote." /><PolicyStep number="03" title="Approve" text="The current NFT owner approves the quote on chain." /><PolicyStep number="04" title="Prove" text="The technician submits a completion note and evidence hash." /><PolicyStep number="05" title="Pay" text="The owner confirms. The child account pays from reserve." /><div className="guardrail-note"><span>◈</span><p>Contract rules enforce owner authorization, quote matching, reserve sufficiency and payout replay protection.</p></div></aside>
            </section>
          )}

          {asset && activeTab === "passport" && (
            <section className="passport-layout">
              <div className="panel passport-main"><div className="eyebrow">OPERATIONAL IDENTITY</div><div className="passport-title"><div className="passport-stamp">SA<br/><small>NEAR</small></div><div><h2>{asset.name}</h2><p>Non-fungible operational passport · {asset.token_id}</p></div><span className={statusClass(asset.status)}>{asset.status}</span></div><div className="passport-grid"><PassportField label="Asset account" value={asset.asset_account_id} /><PassportField label="Current NFT owner" value={ownerId} /><PassportField label="Reserve policy" value={`${reservePercent.toFixed(0)}% of every deposit`} /><PassportField label="Total deposited" value={`${formatNear(balances?.total_deposited)} NEAR`} /><PassportField label="Operating balance" value={`${formatNear(balances?.operating_balance)} NEAR`} /><PassportField label="Maintenance reserve" value={`${formatNear(balances?.maintenance_reserve)} NEAR`} /></div><div className="passport-history"><div className="eyebrow">PERSISTENT SERVICE HISTORY</div>{workOrders.length ? workOrders.map((o) => <div className="history-row" key={o.work_order_id}><div className="history-dot" /><div><strong>{o.work_order_id} · {o.diagnosis}</strong><small>{formatDate(o.created_at_ns)} · {o.diagnosis_source}</small>{o.paid_amount_yocto && <small>Paid from reserve: {formatNear(o.paid_amount_yocto)} NEAR</small>}{o.completion_evidence_hash && <small>Evidence SHA-256: <span className="mono">{shortHash(o.completion_evidence_hash)}</span></small>}{o.completion_evidence_uri && <small>Evidence reference: <span className="mono">{o.completion_evidence_uri}</span></small>}</div><span className={statusClass(o.status)}>{o.status}</span></div>) : <p className="muted">No service events recorded yet. Completed jobs will appear here and remain with the asset through handover.</p>}</div><div className="passport-history"><div className="eyebrow">OWNERSHIP HISTORY · WHO HELD THE PASSPORT</div>{ownershipHistory.length ? ownershipHistory.map((event) => <div className="history-row" key={String(event.timestamp_ns) + "-" + event.owner_id}><div className="history-dot" /><div><strong>{event.previous_owner_id ? "Transferred from " + formatAccount(event.previous_owner_id) + " to " + formatAccount(event.owner_id) : "Minted to " + formatAccount(event.owner_id)}</strong><small>{formatDate(event.timestamp_ns)} · {event.memo || "Ownership event"}</small></div><span className="status status-neutral">{event.previous_owner_id ? "TRANSFER" : "MINT"}</span></div>) : <p className="muted">No ownership events are recorded yet.</p>}</div>{evidencePreview.startsWith("data:image/") && <div className="evidence-preview-wrap"><div className="eyebrow">LOCAL EVIDENCE PREVIEW · BROWSER SESSION</div><img className="evidence-preview" src={evidencePreview} alt="Completion evidence stored in this browser session" /></div>}</div>
              <aside className="panel proof-panel"><div className="eyebrow">PORTABILITY PROOF</div><h3>The passport moves. The history stays.</h3><p>Unlike a technician's chat thread or an operator's spreadsheet, the asset's account and service records are keyed to the NFT token ID.</p><div className="proof-stat"><strong>{workOrders.length}</strong><small>ON-CHAIN WORK ORDERS</small></div><div className="proof-stat"><strong>{formatNear(balances?.maintenance_reserve)}</strong><small>NEAR RESERVE BALANCE</small></div><button className="secondary-button full-width" onClick={() => setActiveTab("handover")}>Review handover controls →</button></aside>
            </section>
          )}

          {asset && activeTab === "handover" && (
            <section className="handover-layout">
              <div className="panel handover-main"><div className="eyebrow">NEP-171 OWNER HANDOVER</div><h2>Change the operator, not the asset's memory.</h2><p className="muted">The NFT owner mapping changes. The dedicated child account, balance and work-order history remain attached to the same token ID.</p><div className="handover-compare"><div><small>FROM / CURRENT OWNER</small><strong className="mono">{formatAccount(ownerId)}</strong></div><div className="handover-arrow">→</div><div><small>TO / NEW OWNER</small><strong className="mono">{formatAccount(transferTo)}</strong></div></div><label className="field-label" htmlFor="transfer-account">RECEIVING NEAR TESTNET ACCOUNT</label><input id="transfer-account" className="text-input" value={transferTo} onChange={(e) => setTransferTo(e.target.value)} placeholder="new-owner.testnet" /><button className="primary-button" onClick={() => void handleTransfer()} disabled={!isOwner || !transferTo.trim() || Boolean(busy)}>Transfer operational passport <span>→</span></button><small className="muted">The NFT transfer uses 1 yoctoNEAR. If passport history storage is low, the same wallet request also includes the quoted 0.01 NEAR storage top-up. This transfers the NFT, not legal title to the physical borehole.</small></div>
              <aside className="panel handover-checklist"><div className="eyebrow">WHAT STAYS WITH THE ASSET</div><CheckItem title="Dedicated child account" text={asset.asset_account_id} /><CheckItem title="Operating balance" text={`${formatNear(balances?.operating_balance)} NEAR`} /><CheckItem title="Protected maintenance reserve" text={`${formatNear(balances?.maintenance_reserve)} NEAR`} /><CheckItem title="Service history" text={`${workOrders.length} on-chain work orders`} /><CheckItem title="Payout authority" text="Moves to the current NFT owner" /><div className="guardrail-note"><span>!</span><p>After transfer, the previous owner is no longer authorized to approve new payouts. Verify the new owner and history on chain.</p></div></aside>
            </section>
          )}

          {loading && <div className="loading-line"><span className="pulse-dot" />Reading finalised NEAR testnet state…</div>}
          <footer className="page-footer"><span>THE SELF-MAINTAINING ASSET · TESTNET PROTOTYPE</span><span>Operator input ≠ sensor verification · No autonomous spending</span></footer>
        </div>
      </main>
    </div>
  );
}

function StatCard({ label, value, suffix, detail, tone, icon }: { label: string; value: string; suffix: string; detail: string; tone: string; icon: string }) {
  return <div className={`stat-card tone-${tone}`}><div className="stat-top"><span>{label}</span><i>{icon}</i></div><div className="stat-value">{value}<small>{suffix}</small></div><div className="stat-detail">{detail}</div></div>;
}

function WorkOrderList({ orders, onOpen }: { orders: WorkOrder[]; accountId: string; ownerId: string; onOpen: () => void }) {
  return <section className="panel workorders-preview"><div className="panel-head"><div><div className="eyebrow">SERVICE RECORD</div><h2>Recent maintenance</h2></div><button className="text-button" onClick={onOpen}>Open maintenance desk →</button></div>{orders.length ? orders.map((o) => <div className="workorder-row" key={o.work_order_id}><div className="workorder-icon">⌁</div><div className="workorder-copy"><strong>{o.diagnosis}</strong><small>{o.work_order_id} · {formatDate(o.created_at_ns)}</small></div><span className={statusClass(o.priority)}>{o.priority}</span><span className={statusClass(o.status)}>{o.status}</span></div>) : <p className="muted">No maintenance events yet. Record the first operator observation to begin the asset's service history.</p>}</section>;
}

function WorkOrderCard(props: {
  order: WorkOrder; accountId: string; ownerId: string; technicianId: string; setTechnicianId: (v: string) => void;
  quoteAmount: string; setQuoteAmount: (v: string) => void; quoteNote: string; setQuoteNote: (v: string) => void;
  completionNote: string; setCompletionNote: (v: string) => void; completionFile: File | null; setCompletionFile: (v: File | null) => void;
  onAssign: () => void; onQuote: () => void; onApprove: () => void; onComplete: () => void; onPay: () => void;
}) {
  const { order, accountId, ownerId } = props;
  const isOwner = accountId === ownerId;
  const isTech = accountId === order.technician_id;
  const quoteNear = order.quote_amount_yocto ? formatNear(order.quote_amount_yocto, 3) : "—";
  return <article className="workorder-card"><div className="workorder-card-head"><div><div className="eyebrow">{order.work_order_id} · {formatDate(order.created_at_ns)}</div><h3>{order.diagnosis}</h3></div><span className={statusClass(order.status)}>{order.status.replaceAll("_", " ")}</span></div><p>{order.observation}</p><div className="workorder-summary"><div><small>PRIORITY</small><strong>{order.priority}</strong></div><div><small>COORDINATOR SOURCE</small><strong>{order.diagnosis_source}</strong></div><div><small>RECOMMENDED ACTION</small><strong>{order.recommended_action}</strong></div></div>{order.quote_amount_yocto && <div className="quote-summary"><span>Technician quote</span><strong>{quoteNear} NEAR</strong><small>{order.quote_note}</small></div>}{order.completion_evidence_hash && <div className="evidence-summary"><span>Completion evidence · SHA-256</span><strong className="mono">{shortHash(order.completion_evidence_hash)}</strong><small>{order.completion_note}</small>{order.completion_evidence_uri && <small className="mono">Reference: {order.completion_evidence_uri}</small>}</div>}
    {isOwner && order.status === "DIAGNOSED" && <div className="action-block"><label className="field-label">ASSIGN TECHNICIAN ACCOUNT</label><div className="input-action"><input value={props.technicianId} onChange={(e) => props.setTechnicianId(e.target.value)} placeholder="technician.testnet" /><button className="primary-button" onClick={props.onAssign}>Assign</button></div></div>}
    {isTech && order.status === "AWAITING_QUOTE" && <div className="action-block"><label className="field-label">YOUR QUOTE</label><div className="input-action"><input value={props.quoteAmount} onChange={(e) => props.setQuoteAmount(e.target.value)} inputMode="decimal" /><span>NEAR</span></div><textarea value={props.quoteNote} onChange={(e) => props.setQuoteNote(e.target.value)} rows={2} /><button className="primary-button" onClick={props.onQuote}>Submit signed quote →</button></div>}
    {isOwner && order.status === "QUOTED" && <div className="action-block"><div className="approval-note"><span>◈</span><div><strong>Quote awaits owner approval</strong><small>Only the current NFT owner can authorize the quoted scope and amount.</small></div></div><button className="primary-button" onClick={props.onApprove}>Approve quote →</button></div>}
    {isTech && order.status === "APPROVED" && <div className="action-block"><label className="field-label">COMPLETION REPORT</label><textarea value={props.completionNote} onChange={(e) => props.setCompletionNote(e.target.value)} rows={2} /><label className="file-button">＋ Attach completion evidence<input type="file" accept="image/*,.pdf,.txt" onChange={(e) => props.setCompletionFile(e.target.files?.[0] || null)} /></label><small>{props.completionFile?.name || "Required · file hash is recorded on chain; file remains in this browser session."}</small><button className="primary-button" onClick={props.onComplete} disabled={!props.completionFile}>Submit completion proof →</button></div>}
    {isOwner && ["COMPLETION_SUBMITTED", "PAYMENT_FAILED"].includes(order.status) && <div className="action-block"><div className="approval-note"><span>✓</span><div><strong>Completion evidence submitted</strong><small>Review the technician report and confirm payment of the exact approved quote from the protected reserve.</small></div></div><button className="primary-button" onClick={props.onPay}>Confirm completion & pay {quoteNear} NEAR →</button></div>}
    {order.status === "PAYOUT_PENDING" && <div className="approval-note"><span>◷</span><div><strong>Reserve payout is processing</strong><small>The registry is waiting for the child-account transfer callback. Do not submit another payout.</small></div></div>}
    {order.status === "PAID" && <div className="paid-note">✓ Paid from the asset's maintenance reserve · {formatNear(order.paid_amount_yocto)} NEAR</div>}
    {order.status === "PAYOUT_UNKNOWN" && <div className="warn-note">Payout outcome is unknown. The contract keeps the work-order replay lock; reconcile the transaction before retrying.</div>}
    {order.status === "PAYMENT_FAILED" && <div className="warn-note">Transfer failed and reserve was restored. The current owner may retry payment after checking the technician account.</div>}
  </article>;
}

function EmptyState({ title, body, action, actionLabel }: { title: string; body: string; action: () => void; actionLabel: string }) {
  return <div className="empty-state"><div className="empty-orb">⌁</div><h3>{title}</h3><p>{body}</p><button className="secondary-button" onClick={action}>{actionLabel} →</button></div>;
}
function PolicyStep({ number, title, text }: { number: string; title: string; text: string }) { return <div className="policy-step"><span>{number}</span><div><strong>{title}</strong><p>{text}</p></div></div>; }
function PassportField({ label, value }: { label: string; value: string }) { return <div className="passport-field"><small>{label}</small><strong className="mono">{value}</strong></div>; }
function CheckItem({ title, text }: { title: string; text: string }) { return <div className="check-item"><span>✓</span><div><strong>{title}</strong><small>{text}</small></div></div>; }

export default App;
