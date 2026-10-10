<script lang="ts">
  import { onMount } from "svelte";
  import { invoke, isTauri } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { availableMonitors } from "@tauri-apps/api/window";
  import { ui, setLang, t, tr } from "./lib/i18n.svelte";
  import { describe, deviceLabel, DEVICE_ICON } from "./lib/describe";
  import { TASKS, trackModifiers, type Ctx, type Group } from "./lib/tasks";
  import { buildHtml, buildJson, latencySummary, systemSlug, verdicts, type ReportInput, type TaskStatus } from "./lib/report";
  import type { DeviceInfo, EnvReport, InputEvent, LiveEvent, MonitorInfo, Stats } from "./lib/types";

  const inApp = isTauri();
  const MAX_KEEP = 3000; // eventos guardados para o relatório
  const MAX_SHOW = 120; // eventos mostrados na tela

  let env = $state<EnvReport | null>(null);
  let monitors = $state<MonitorInfo[]>([]);
  let devices = $state<DeviceInfo[]>([]);
  let running = $state(false);
  let busy = $state(false);
  let error = $state<string | null>(null);

  let shown = $state.raw<LiveEvent[]>([]);
  let kept: LiveEvent[] = [];
  let seq = 0;
  let totalEvents = $state(0);
  let latencies: number[] = [];
  let lat = $state({ n: 0, avg: 0, p95: 0, max: 0 });
  let samples = $state.raw<Stats[]>([]);
  let diagnostics = $state<string[]>([]);

  let status = $state<Record<string, TaskStatus>>({});
  let counts = $state<Record<string, number>>({});
  let skippedGroups = $state<Group[]>([]);
  let userNotes = $state("");
  let savedPaths = $state<string[]>([]);
  let flashId = $state<string | null>(null);

  const ctx: Ctx = { modifiers: new Set() };
  const groups: Group[] = ["mouse", "touchpad", "keyboard"];

  const lastStats = $derived(samples[samples.length - 1] as Stats | undefined);
  const doneCount = $derived(TASKS.filter((x) => status[x.id] === "done").length);
  const activeCount = $derived(TASKS.filter((x) => status[x.id] !== "skipped").length);

  function taskStatus(id: string): TaskStatus {
    return status[id] ?? "pending";
  }

  function onEvent(ev: InputEvent) {
    const latency = Math.max(0, Date.now() - ev.t_ms);
    const live: LiveEvent = { ...ev, seq: ++seq, latency_ms: latency };
    totalEvents++;
    kept.push(live);
    if (kept.length > MAX_KEEP) kept.splice(0, kept.length - MAX_KEEP);
    shown = [live, ...shown].slice(0, MAX_SHOW);

    if (ev.kind.type === "diagnostic") {
      if (!diagnostics.includes(ev.kind.message)) diagnostics.push(ev.kind.message);
      return;
    }
    latencies.push(latency);
    if (latencies.length > 2000) latencies.splice(0, latencies.length - 2000);
    if (seq % 5 === 0 || latencies.length < 20) lat = latencySummary(latencies);

    trackModifiers(ev, ctx);
    for (const task of TASKS) {
      if (taskStatus(task.id) === "skipped") continue;
      if (task.match(ev, ctx)) {
        counts[task.id] = (counts[task.id] ?? 0) + 1;
        if (status[task.id] !== "done") {
          status[task.id] = "done";
          flashId = task.id;
          setTimeout(() => flashId === task.id && (flashId = null), 900);
        }
      }
    }
  }

  async function loadInfo() {
    if (!inApp) return;
    try {
      env = await invoke<EnvReport>("environment");
      devices = await invoke<DeviceInfo[]>("devices");
      monitors = (await availableMonitors()).map((m) => ({
        name: m.name,
        width: m.size.width,
        height: m.size.height,
        scale: m.scaleFactor,
        x: m.position.x,
        y: m.position.y,
      }));
      for (const n of env.env.notes) if (/permiss|grupo|group/i.test(n) && !diagnostics.includes(n)) diagnostics.push(n);
    } catch (e) {
      error = String(e);
    }
  }

  async function toggle() {
    busy = true;
    error = null;
    try {
      if (running) {
        await invoke("stop_capture");
        running = false;
      } else if (inApp) {
        await invoke("start_capture");
        running = true;
      } else {
        running = true;
        demo();
      }
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  function setGroupSkipped(g: Group, skip: boolean) {
    skippedGroups = skip ? [...skippedGroups, g] : skippedGroups.filter((x) => x !== g);
    for (const task of TASKS.filter((x) => x.group === g)) {
      if (skip && status[task.id] !== "done") status[task.id] = "skipped";
      if (!skip && status[task.id] === "skipped") status[task.id] = "pending";
    }
  }

  function toggleSkip(id: string) {
    status[id] = taskStatus(id) === "skipped" ? "pending" : "skipped";
  }

  function input(): ReportInput {
    return {
      lang: ui.lang, env, monitors, devices,
      status: { ...status }, counts: { ...counts }, skippedGroups: [...skippedGroups],
      samples, latencies, events: kept, totalEvents, diagnostics: [...diagnostics], userNotes,
    };
  }

  async function save() {
    error = null;
    const r = input();
    try {
      if (inApp) {
        savedPaths = await invoke<string[]>("save_report", { system: systemSlug(env), html: buildHtml(r), json: buildJson(r) });
      } else {
        const blob = new Blob([buildHtml(r)], { type: "text/html" });
        window.open(URL.createObjectURL(blob));
      }
    } catch (e) {
      error = String(e);
    }
  }

  // Modo demonstração (só quando aberto no navegador, fora do app).
  function demo() {
    const fake: InputEvent[] = [
      { t_ms: 0, device_id: "demo", device_kind: "mouse", kind: { type: "button_down", button: "left" }, cursor: [300, 200], window: { title: "Sem título – Bloco de notas", process: "notepad.exe" }, classification: "demonstração" },
      { t_ms: 0, device_id: "demo", device_kind: "mouse", kind: { type: "wheel", delta: -1, horizontal: false }, cursor: null, window: null, classification: "demonstração" },
      { t_ms: 0, device_id: "demo", device_kind: "touchpad", kind: { type: "gesture", gesture: "swipe", fingers: 3, direction: "left" }, cursor: null, window: null, classification: "demonstração" },
      { t_ms: 0, device_id: "demo", device_kind: "keyboard", kind: { type: "key", class: "letter", down: true, name: null }, cursor: null, window: { title: "Documento – Writer", process: "soffice" }, classification: "demonstração" },
      { t_ms: 0, device_id: "demo", device_kind: "touchpad", kind: { type: "gesture", gesture: "tap", fingers: 2 }, cursor: null, window: null, classification: "demonstração" },
    ];
    let i = 0;
    const timer = setInterval(() => {
      if (!running) return clearInterval(timer);
      onEvent({ ...fake[i++ % fake.length], t_ms: Date.now() - Math.round(Math.random() * 4) });
    }, 900);
  }

  onMount(() => {
    document.documentElement.lang = ui.lang === "pt" ? "pt-BR" : "en";
    if (!inApp) return;
    loadInfo();
    invoke<boolean>("capture_running").then((r) => (running = r));
    const unlisten = listen<InputEvent>("mp-input", (e) => onEvent(e.payload));
    const timer = setInterval(async () => {
      if (!running) return;
      try {
        samples = [...samples, await invoke<Stats>("stats")].slice(-600);
      } catch {
        /* ignorar */
      }
    }, 2000);
    return () => {
      clearInterval(timer);
      unlisten.then((f) => f());
    };
  });

  const fmtTime = (ms: number) => new Date(ms).toLocaleTimeString(ui.lang === "pt" ? "pt-BR" : "en", { hour12: false }) + "." + String(ms % 1000).padStart(3, "0");
  const hex = (n: number | null) => (n == null ? "" : n.toString(16).padStart(4, "0"));
</script>

<header class="top">
  <div class="brand">
    <div class="logo" aria-hidden="true">
      <svg viewBox="0 0 32 32"><path d="M6 25c4-1 5-6 9-8s8 1 11-4" /><circle cx="6" cy="25" r="2.6" /><circle cx="26" cy="9" r="2.6" /></svg>
    </div>
    <div>
      <h1>{t("title")}</h1>
      <p>{t("subtitle")}</p>
    </div>
  </div>
  <div class="actions">
    <div class="lang" role="group" aria-label="Idioma / Language">
      <button class:on={ui.lang === "pt"} onclick={() => setLang("pt")}>PT</button>
      <button class:on={ui.lang === "en"} onclick={() => setLang("en")}>EN</button>
    </div>
    <span class="pill" class:live={running}>{running ? t("running") : t("stopped")}</span>
    <button class="primary" class:danger={running} disabled={busy} onclick={toggle}>{running ? t("stop") : t("start")}</button>
    <button onclick={save}>{t("save")}</button>
  </div>
</header>

{#if !inApp}
  <div class="banner">{tr("Você está vendo a interface no navegador (modo demonstração). Para testar de verdade, abra o app My Path Probe.", "You are viewing the UI in a browser (demo mode). To test for real, open the My Path Probe app.")}</div>
{/if}
{#if error}<div class="banner err">{t("error")}: {error}</div>{/if}
{#if diagnostics.length}
  <div class="banner warn">
    <b>{t("warnings")}:</b>
    <ul>{#each diagnostics as d}<li>{d}</li>{/each}</ul>
  </div>
{/if}
{#if savedPaths.length}
  <div class="banner ok">
    {t("saved")} <code>{savedPaths[0]}</code>
    <button class="small" onclick={() => invoke("reveal", { path: savedPaths[0] })}>{t("open_folder")}</button>
  </div>
{/if}

<main class="grid">
  <section class="card tasks">
    <div class="card-head">
      <h2>{t("tasks")}</h2>
      <span class="muted">{doneCount}/{activeCount} {t("progress")}</span>
    </div>
    <div class="bar"><div style:width="{activeCount ? (100 * doneCount) / activeCount : 0}%"></div></div>
    <p class="muted">{t("tasks_help")}</p>
    {#each groups as g}
      <div class="group">
        <div class="group-head">
          <h3>{t(`group_${g}`)}</h3>
          {#if g !== "keyboard"}
            <label class="check">
              <input type="checkbox" checked={skippedGroups.includes(g)} onchange={(e) => setGroupSkipped(g, e.currentTarget.checked)} />
              {g === "mouse" ? t("no_mouse") : t("no_touchpad")}
            </label>
          {/if}
        </div>
        <ul>
          {#each TASKS.filter((x) => x.group === g) as task (task.id)}
            {@const st = taskStatus(task.id)}
            <li class={st} class:flash={flashId === task.id}>
              <span class="mark">{st === "done" ? "✓" : st === "skipped" ? "–" : ""}</span>
              <span class="txt">
                {tr(task.pt, task.en)}
                {#if task.hintPt && st === "pending"}<small>{tr(task.hintPt, task.hintEn ?? "")}</small>{/if}
              </span>
              {#if counts[task.id]}<span class="count">{counts[task.id]}×</span>{/if}
              {#if st !== "done"}
                <button class="link" onclick={() => toggleSkip(task.id)}>{st === "skipped" ? t("unskip") : t("skip")}</button>
              {/if}
            </li>
          {/each}
        </ul>
      </div>
    {/each}
  </section>

  <div class="col">
    <section class="card perf">
      <h2>{t("perf")}</h2>
      <div class="kpis">
        <div class="kpi" class:bad={(lastStats?.cpu_tree ?? 0) >= 3}>
          <b>{lastStats ? lastStats.cpu_tree.toFixed(1) : "—"}%</b><span>{t("perf_cpu")}</span><small>{t("perf_goal")} &lt; 3%</small>
        </div>
        <div class="kpi" class:bad={(lastStats?.ram_tree_mb ?? 0) >= 150}>
          <b>{lastStats ? lastStats.ram_tree_mb.toFixed(0) : "—"} MB</b><span>{t("perf_ram")}</span><small>{t("perf_goal")} &lt; 150 MB</small>
        </div>
        <div class="kpi">
          <b>{lat.n ? lat.avg.toFixed(1) : "—"} ms</b><span>{t("perf_lat")}</span><small>{t("max")} {lat.max.toFixed(0)} ms</small>
        </div>
        <div class="kpi"><b>{totalEvents}</b><span>{t("perf_events")}</span><small>&nbsp;</small></div>
      </div>
      {#if activeCount}
        <ul class="verdicts">
          {#each verdicts(input()) as v}
            <li class={v.result}>
              <span>{({ ok: "✅", warn: "⚠️", fail: "❌", untested: "➖" })[v.result]}</span>
              <b>{v.challenge}.</b> {tr(...v.name)} <small>{tr(...v.detail)}</small>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <section class="card live">
      <div class="card-head">
        <h2>{t("live")}</h2>
        <button class="link" onclick={() => (shown = [])}>{t("clear")}</button>
      </div>
      <p class="privacy">🔒 {t("privacy")}</p>
      {#if shown.length === 0}
        <p class="muted empty">{t("live_empty")}</p>
      {:else}
        <ol class="events">
          {#each shown as ev (ev.seq)}
            <li class="ev {ev.device_kind}" class:diag={ev.kind.type === "diagnostic"}>
              <span class="ico" title={deviceLabel(ev.device_kind, ui.lang)}>{DEVICE_ICON[ev.device_kind]}</span>
              <span class="what">
                {describe(ev, ui.lang)}
                {#if ev.classification}<small>{t("why")}: {ev.classification}{ev.cursor ? ` · (${ev.cursor[0]}, ${ev.cursor[1]})` : ""}</small>{/if}
              </span>
              <span class="when"><code>{fmtTime(ev.t_ms)}</code><small>{ev.latency_ms} ms</small></span>
            </li>
          {/each}
        </ol>
      {/if}
    </section>
  </div>

  <section class="card">
    <div class="card-head"><h2>{t("env")}</h2></div>
    {#if env}
      <dl class="kv">
        <dt>{t("os")}</dt><dd>{env.env.os_name ?? env.env.os_family}</dd>
        {#if env.env.os_version}<dt>{t("version")}</dt><dd>{env.env.os_version}</dd>{/if}
        {#if env.env.kernel_version}<dt>{t("kernel")}</dt><dd>{env.env.kernel_version}</dd>{/if}
        {#if env.env.session_type}<dt>{t("session")}</dt><dd>{env.env.session_type}</dd>{/if}
        {#if env.env.desktop}<dt>{t("desktop")}</dt><dd>{env.env.desktop}</dd>{/if}
        <dt>{t("arch")}</dt><dd>{env.env.arch}</dd>
        <dt>{t("cpu")}</dt><dd>{env.cpu_brand} · {env.cpu_logical}</dd>
        <dt>{t("ram")}</dt><dd>{(env.total_ram_mb / 1024).toFixed(1)} GB</dd>
        {#if env.webview_version}<dt>{t("webview")}</dt><dd>{env.webview_version}</dd>{/if}
        <dt>{t("monitors")}</dt>
        <dd>{monitors.map((m) => `${m.width}×${m.height} @ ${Math.round(m.scale * 100)}%`).join(" · ") || "—"}</dd>
        {#if env.env.notes.length}<dt>{t("notes")}</dt><dd>{env.env.notes.join(" · ")}</dd>{/if}
      </dl>
    {:else}
      <p class="muted">—</p>
    {/if}
  </section>

  <section class="card">
    <div class="card-head">
      <h2>{t("devices")}</h2>
      <button class="link" onclick={loadInfo}>{t("refresh")}</button>
    </div>
    {#if devices.length === 0}
      <p class="muted">{t("no_devices")}</p>
    {:else}
      <ul class="devices">
        {#each devices as d (d.id)}
          <li>
            <span class="ico">{DEVICE_ICON[d.kind]}</span>
            <span>
              <b>{deviceLabel(d.kind, ui.lang)}</b>
              {#if d.vendor_id != null}<code>{hex(d.vendor_id)}:{hex(d.product_id)}</code>{/if}
              <small>{d.name}{d.notes.length ? ` — ${d.notes.join("; ")}` : ""}</small>
            </span>
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <section class="card wide">
    <h2>{t("user_notes")}</h2>
    <textarea bind:value={userNotes} rows="3" placeholder={t("user_notes_ph")}></textarea>
    <div class="right"><button class="primary" onclick={save}>{t("save")}</button></div>
  </section>
</main>
