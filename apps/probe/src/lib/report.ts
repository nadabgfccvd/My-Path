// Gera o relatório (JSON para o desenvolvedor + HTML legível) inteiramente no frontend.
import type { DeviceInfo, EnvReport, LiveEvent, MonitorInfo, Stats } from "./types";
import { TASKS, type Group } from "./tasks";
import { describe, deviceLabel } from "./describe";
import type { Lang } from "./i18n.svelte";

export type TaskStatus = "done" | "pending" | "skipped";

export interface ReportInput {
  lang: Lang;
  env: EnvReport | null;
  monitors: MonitorInfo[];
  devices: DeviceInfo[];
  status: Record<string, TaskStatus>;
  counts: Record<string, number>;
  skippedGroups: Group[];
  samples: Stats[];
  latencies: number[];
  events: LiveEvent[];
  totalEvents: number;
  diagnostics: string[];
  userNotes: string;
}

export interface Verdict {
  challenge: number;
  name: [string, string];
  result: "ok" | "warn" | "fail" | "untested";
  detail: [string, string];
}

function pct(n: number, d: number) {
  return d === 0 ? "—" : `${Math.round((100 * n) / d)}%`;
}

export function latencySummary(lat: number[]) {
  if (lat.length === 0) return { n: 0, avg: 0, p95: 0, max: 0 };
  const s = [...lat].sort((a, b) => a - b);
  const avg = s.reduce((a, b) => a + b, 0) / s.length;
  return { n: s.length, avg, p95: s[Math.min(s.length - 1, Math.floor(s.length * 0.95))], max: s[s.length - 1] };
}

export function verdicts(r: ReportInput): Verdict[] {
  const byGroup = (g: Group, ids?: string[]) => {
    const list = TASKS.filter((t) => t.group === g && (!ids || ids.includes(t.id)));
    const active = list.filter((t) => r.status[t.id] !== "skipped");
    return { done: active.filter((t) => r.status[t.id] === "done").length, total: active.length };
  };
  const res = (done: number, total: number): Verdict["result"] =>
    total === 0 ? "untested" : done === total ? "ok" : done === 0 ? "fail" : "warn";

  const c1m = byGroup("mouse", ["m_middle", "m_wheel_up", "m_wheel_down"]);
  const c1k = byGroup("keyboard");
  const c1 = { done: c1m.done + c1k.done, total: c1m.total + c1k.total };

  const c2ids = ["m_left", "m_right", "t_tap1_kind", "t_right_kind"];
  const c2list = TASKS.filter((t) => c2ids.includes(t.id) && r.status[t.id] !== "skipped");
  const c2 = { done: c2list.filter((t) => r.status[t.id] === "done").length, total: c2list.length };

  const c3 = byGroup("touchpad", TASKS.filter((t) => t.group === "touchpad" && t.challenge === 3).map((t) => t.id));

  const perm = r.diagnostics.filter((d) => /permiss|permission|input/i.test(d));
  const maxCpu = Math.max(0, ...r.samples.map((s) => s.cpu_tree));
  const maxRam = Math.max(0, ...r.samples.map((s) => s.ram_tree_mb));
  const lat = latencySummary(r.latencies);
  const linux = r.env?.env.os_family === "linux";

  const out: Verdict[] = [
    { challenge: 1, name: ["Captura global de mouse e teclado", "Global mouse & keyboard capture"], result: res(c1.done, c1.total),
      detail: [`${c1.done} de ${c1.total} tarefas detectadas`, `${c1.done} of ${c1.total} tasks detected`] },
    { challenge: 2, name: ["Mouse ou touchpad?", "Mouse or touchpad?"], result: res(c2.done, c2.total),
      detail: [`${c2.done} de ${c2.total} cliques atribuídos ao dispositivo certo`, `${c2.done} of ${c2.total} clicks attributed to the right device`] },
    { challenge: 3, name: ["Gestos do touchpad", "Touchpad gestures"], result: res(c3.done, c3.total),
      detail: [`${c3.done} de ${c3.total} gestos reconhecidos`, `${c3.done} of ${c3.total} gestures recognised`] },
  ];
  if (linux)
    out.push({ challenge: 7, name: ["Permissões no Linux", "Linux permissions"], result: perm.length ? "fail" : r.devices.length ? "ok" : "warn",
      detail: perm.length ? [perm[0], perm[0]] : [`${r.devices.length} dispositivos legíveis`, `${r.devices.length} readable devices`] });
  out.push({ challenge: 9, name: ["Desempenho", "Performance"],
    result: r.samples.length === 0 ? "untested" : maxCpu < 3 && maxRam < 150 ? "ok" : maxCpu < 6 && maxRam < 250 ? "warn" : "fail",
    detail: [
      `CPU máx. ${maxCpu.toFixed(1)}% (meta < 3%) · RAM máx. ${maxRam.toFixed(0)} MB (meta < 150 MB) · atraso médio ${lat.avg.toFixed(1)} ms`,
      `Max CPU ${maxCpu.toFixed(1)}% (goal < 3%) · max RAM ${maxRam.toFixed(0)} MB (goal < 150 MB) · avg delay ${lat.avg.toFixed(1)} ms`,
    ] });
  return out;
}

export function systemSlug(env: EnvReport | null): string {
  if (!env) return "desconhecido";
  const e = env.env;
  return [e.os_name ?? e.os_family, e.session_type].filter(Boolean).join(" ");
}

export function buildJson(r: ReportInput): string {
  return JSON.stringify(
    {
      format: "mypath-probe-report",
      format_version: 1,
      generated_at: new Date().toISOString(),
      ui_lang: r.lang,
      environment: r.env,
      monitors: r.monitors,
      devices: r.devices,
      verdicts: verdicts(r).map((v) => ({ challenge: v.challenge, result: v.result, detail: v.detail[1] })),
      tasks: TASKS.map((t) => ({ id: t.id, group: t.group, challenge: t.challenge, status: r.status[t.id] ?? "pending", count: r.counts[t.id] ?? 0 })),
      skipped_groups: r.skippedGroups,
      performance: { samples: r.samples, latency_ms: latencySummary(r.latencies) },
      total_events: r.totalEvents,
      diagnostics: r.diagnostics,
      user_notes: r.userNotes,
      // Eventos brutos: viram testes de regressão do motor de gestos.
      events: r.events,
    },
    null,
    2,
  );
}

const esc = (s: string) => s.replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]!);

export function buildHtml(r: ReportInput): string {
  const L = (pt: string, en: string) => (r.lang === "pt" ? pt : en);
  const icon = { ok: "✅", warn: "⚠️", fail: "❌", untested: "➖" };
  const e = r.env?.env;
  const lat = latencySummary(r.latencies);
  const rows = (pairs: [string, string | null | undefined][]) =>
    pairs.filter(([, v]) => v).map(([k, v]) => `<tr><th>${esc(k)}</th><td>${esc(String(v))}</td></tr>`).join("");

  const envRows = rows([
    [L("Sistema", "System"), e?.os_name],
    [L("Versão", "Version"), e?.os_version],
    [L("Kernel", "Kernel"), e?.kernel_version],
    [L("Sessão", "Session"), e?.session_type],
    [L("Ambiente gráfico", "Desktop"), e?.desktop],
    [L("Arquitetura", "Architecture"), e?.arch],
    [L("Processador", "Processor"), r.env ? `${r.env.cpu_brand} (${r.env.cpu_logical} ${L("núcleos lógicos", "logical cores")})` : null],
    [L("Memória", "Memory"), r.env ? `${(r.env.total_ram_mb / 1024).toFixed(1)} GB` : null],
    ["WebView", r.env?.webview_version],
    [L("Monitores", "Monitors"), r.monitors.map((m) => `${m.width}×${m.height} @ ${Math.round(m.scale * 100)}%`).join(" · ")],
    ["Probe", e?.app_version],
  ]);
  const verdictRows = verdicts(r)
    .map((v) => `<tr class="${v.result}"><td class="ic">${icon[v.result]}</td><td><b>${L("Desafio", "Challenge")} ${v.challenge}</b> — ${esc(L(...v.name))}<br><small>${esc(L(...v.detail))}</small></td></tr>`)
    .join("");
  const taskRows = TASKS.map((t) => {
    const st = r.status[t.id] ?? "pending";
    const mark = st === "done" ? "✅" : st === "skipped" ? "➖" : "⬜";
    return `<tr class="${st}"><td class="ic">${mark}</td><td>${esc(L(t.pt, t.en))}</td><td class="n">${r.counts[t.id] ?? 0}×</td></tr>`;
  }).join("");
  const devRows = r.devices
    .map((d) => {
      const id = d.vendor_id != null ? `${d.vendor_id.toString(16).padStart(4, "0")}:${(d.product_id ?? 0).toString(16).padStart(4, "0")}` : "";
      return `<tr><td>${esc(deviceLabel(d.kind, r.lang))}</td><td>${esc(d.name)}</td><td><code>${id}</code></td><td><small>${esc(d.notes.join("; "))}</small></td></tr>`;
    })
    .join("");
  const evRows = r.events
    .slice(-300)
    .map((ev) => `<tr><td><code>${new Date(ev.t_ms).toLocaleTimeString()}</code></td><td>${esc(deviceLabel(ev.device_kind, r.lang))}</td><td>${esc(describe(ev, r.lang))}</td><td><small>${esc(ev.classification ?? "")}</small></td></tr>`)
    .join("");
  const diag = r.diagnostics.length ? `<h2>${L("Avisos", "Warnings")}</h2><ul class="diag">${r.diagnostics.map((d) => `<li>${esc(d)}</li>`).join("")}</ul>` : "";
  const notes = r.userNotes.trim() ? `<h2>${L("Comentários do testador", "Tester comments")}</h2><blockquote>${esc(r.userNotes)}</blockquote>` : "";

  return `<!doctype html><html lang="${r.lang === "pt" ? "pt-BR" : "en"}"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>${L("Relatório do Probe", "Probe report")} — ${esc(systemSlug(r.env))}</title>
<style>
:root{--bg:#f6f7fb;--card:#fff;--ink:#1d2433;--mute:#667085;--line:#e4e7ec;--ok:#12b76a;--warn:#f79009;--fail:#f04438;--acc:#5b5bd6}
@media (prefers-color-scheme:dark){:root{--bg:#0f1218;--card:#171b24;--ink:#e6e9f0;--mute:#98a2b3;--line:#2a3140}}
*{box-sizing:border-box}body{margin:0;font:15px/1.5 system-ui,Segoe UI,Roboto,sans-serif;background:var(--bg);color:var(--ink)}
main{max-width:980px;margin:auto;padding:32px 20px}
header{background:linear-gradient(135deg,#5b5bd6,#2e90fa);color:#fff;border-radius:18px;padding:26px 28px;margin-bottom:22px}
header h1{margin:0 0 4px;font-size:26px}header p{margin:0;opacity:.9}
h2{font-size:18px;margin:28px 0 10px}
table{width:100%;border-collapse:collapse;background:var(--card);border-radius:14px;overflow:hidden;box-shadow:0 1px 2px #0001}
th,td{padding:9px 12px;border-bottom:1px solid var(--line);text-align:left;vertical-align:top}th{width:32%;color:var(--mute);font-weight:600}
td.ic{width:40px;text-align:center;font-size:18px}td.n{width:60px;text-align:right;color:var(--mute)}
tr.skipped td{color:var(--mute)}small{color:var(--mute)}code{font-size:12px}
.diag li{color:var(--fail)}blockquote{background:var(--card);margin:0;padding:14px 18px;border-left:4px solid var(--acc);border-radius:8px;white-space:pre-wrap}
footer{margin-top:30px;color:var(--mute);font-size:13px;text-align:center}
</style></head><body><main>
<header><h1>${L("Relatório do My Path Probe", "My Path Probe report")}</h1>
<p>${esc(systemSlug(r.env))} · ${new Date().toLocaleString(r.lang === "pt" ? "pt-BR" : "en")} · ${r.totalEvents} ${L("eventos", "events")} · ${L("atraso médio", "avg delay")} ${lat.avg.toFixed(1)} ms (p95 ${lat.p95.toFixed(1)} ms)</p></header>
<h2>${L("Resultado por desafio", "Result per challenge")}</h2><table>${verdictRows}</table>
${diag}
<h2>${L("Sistema", "System")}</h2><table>${envRows}</table>
<h2>${L("Roteiro", "Script")}</h2><table>${taskRows}</table>
<h2>${L("Dispositivos", "Devices")}</h2><table>${devRows || `<tr><td>${L("Nenhum", "None")}</td></tr>`}</table>
${notes}
<h2>${L("Últimos eventos", "Latest events")}</h2><table>${evRows}</table>
<footer>${L("Letras e números digitados nunca são gravados. Envie também o arquivo .json ao desenvolvedor.", "Typed letters and digits are never recorded. Please also send the .json file to the developer.")}<br>My Path — MIT OR Apache-2.0</footer>
</main></body></html>`;
}
