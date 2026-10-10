export type Lang = "pt" | "en";

function initialLang(): Lang {
  const saved = typeof localStorage !== "undefined" ? localStorage.getItem("mp-lang") : null;
  if (saved === "pt" || saved === "en") return saved;
  return typeof navigator !== "undefined" && !navigator.language.toLowerCase().startsWith("pt") ? "en" : "pt";
}

export const ui = $state({ lang: initialLang() });

export function setLang(lang: Lang) {
  ui.lang = lang;
  localStorage.setItem("mp-lang", lang);
  document.documentElement.lang = lang === "pt" ? "pt-BR" : "en";
}

const dict = {
  title: { pt: "My Path — Probe", en: "My Path — Probe" },
  subtitle: {
    pt: "Fase 1: testa se o seu computador entrega ao My Path tudo o que ele precisa.",
    en: "Phase 1: checks whether your computer gives My Path everything it needs.",
  },
  start: { pt: "▶ Iniciar teste", en: "▶ Start test" },
  stop: { pt: "■ Parar", en: "■ Stop" },
  save: { pt: "💾 Salvar relatório", en: "💾 Save report" },
  saved: { pt: "Relatório salvo em:", en: "Report saved to:" },
  open_folder: { pt: "Abrir pasta", en: "Open folder" },
  running: { pt: "Capturando", en: "Capturing" },
  stopped: { pt: "Parado", en: "Stopped" },
  env: { pt: "Seu sistema", en: "Your system" },
  os: { pt: "Sistema", en: "System" },
  version: { pt: "Versão", en: "Version" },
  kernel: { pt: "Kernel", en: "Kernel" },
  session: { pt: "Sessão", en: "Session" },
  desktop: { pt: "Ambiente gráfico", en: "Desktop" },
  arch: { pt: "Arquitetura", en: "Architecture" },
  cpu: { pt: "Processador", en: "Processor" },
  ram: { pt: "Memória", en: "Memory" },
  webview: { pt: "WebView", en: "WebView" },
  monitors: { pt: "Monitores", en: "Monitors" },
  notes: { pt: "Observações", en: "Notes" },
  devices: { pt: "Dispositivos de entrada", en: "Input devices" },
  refresh: { pt: "Atualizar", en: "Refresh" },
  no_devices: {
    pt: "Nenhum dispositivo encontrado. No Linux, isso costuma ser falta de permissão (veja os avisos).",
    en: "No devices found. On Linux this is usually a permission problem (see warnings).",
  },
  tasks: { pt: "Roteiro de teste", en: "Test script" },
  tasks_help: {
    pt: "Clique em “Iniciar teste” e faça cada item. Ele fica verde sozinho quando o Probe detecta certo.",
    en: "Click “Start test” and do each item. It turns green by itself when the Probe detects it correctly.",
  },
  no_touchpad: { pt: "Não tenho touchpad (notebook) neste computador", en: "This computer has no touchpad" },
  no_mouse: { pt: "Não tenho mouse externo neste computador", en: "This computer has no external mouse" },
  skip: { pt: "Pular", en: "Skip" },
  unskip: { pt: "Desfazer", en: "Undo" },
  skipped: { pt: "pulado", en: "skipped" },
  group_mouse: { pt: "Mouse", en: "Mouse" },
  group_touchpad: { pt: "Touchpad", en: "Touchpad" },
  group_keyboard: { pt: "Teclado", en: "Keyboard" },
  live: { pt: "Ao vivo", en: "Live" },
  live_empty: { pt: "Os eventos aparecem aqui assim que você clicar, rolar ou digitar.", en: "Events show up here as soon as you click, scroll or type." },
  clear: { pt: "Limpar", en: "Clear" },
  why: { pt: "por quê", en: "why" },
  perf: { pt: "Desempenho", en: "Performance" },
  perf_cpu: { pt: "CPU (Probe + WebView)", en: "CPU (Probe + WebView)" },
  perf_ram: { pt: "RAM (Probe + WebView)", en: "RAM (Probe + WebView)" },
  perf_lat: { pt: "Atraso até a tela", en: "Delay to screen" },
  perf_events: { pt: "Eventos", en: "Events" },
  perf_goal: { pt: "meta", en: "goal" },
  avg: { pt: "média", en: "avg" },
  max: { pt: "máx.", en: "max" },
  warnings: { pt: "Avisos", en: "Warnings" },
  user_notes: { pt: "Comentários para o desenvolvedor (opcional)", en: "Comments for the developer (optional)" },
  user_notes_ph: {
    pt: "Ex.: “o toque com 3 dedos abriu a pesquisa do Windows”, “usei um mouse Logitech sem fio”…",
    en: "E.g. “the 3-finger tap opened Windows search”, “I used a wireless Logitech mouse”…",
  },
  privacy: {
    pt: "Privacidade: letras e números digitados NUNCA são gravados — só “uma letra foi digitada”. O relatório fica no seu computador até você enviá-lo.",
    en: "Privacy: typed letters and digits are NEVER recorded — only “a letter was typed”. The report stays on your computer until you send it.",
  },
  progress: { pt: "concluídos", en: "done" },
  error: { pt: "Erro", en: "Error" },
} as const;

export type Key = keyof typeof dict;

export function t(key: Key, vars?: Record<string, string | number>): string {
  let s: string = dict[key][ui.lang];
  if (vars) for (const [k, v] of Object.entries(vars)) s = s.replaceAll(`{${k}}`, String(v));
  return s;
}

/** Escolhe o texto do idioma atual entre dois. */
export function tr(pt: string, en: string): string {
  return ui.lang === "pt" ? pt : en;
}
