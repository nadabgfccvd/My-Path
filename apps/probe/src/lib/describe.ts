// Prévia do "motor de frases" do My Path: transforma um evento em texto de passo.
// (Na Fase 2 isso vira o motor completo com modelos editáveis.)
import type { Direction, InputEvent, KeyClass, MouseButton, DeviceKind } from "./types";
import type { Lang } from "./i18n.svelte";

const BUTTON: Record<MouseButton, [string, string]> = {
  left: ["esquerdo", "left"],
  right: ["direito", "right"],
  middle: ["do meio", "middle"],
  x1: ["lateral 1 (voltar)", "side 1 (back)"],
  x2: ["lateral 2 (avançar)", "side 2 (forward)"],
};
const DIR: Record<Direction, [string, string]> = {
  up: ["cima", "up"],
  down: ["baixo", "down"],
  left: ["a esquerda", "left"],
  right: ["a direita", "right"],
};
const DEVICE: Record<DeviceKind, [string, string]> = {
  mouse: ["mouse", "mouse"],
  touchpad: ["touchpad", "touchpad"],
  keyboard: ["teclado", "keyboard"],
  touchscreen: ["tela de toque", "touchscreen"],
  pen: ["caneta", "pen"],
  synthetic: ["dispositivo virtual", "virtual device"],
  unknown: ["dispositivo desconhecido", "unknown device"],
};
const KEYCLASS: Record<KeyClass, [string, string]> = {
  letter: ["uma letra", "a letter"],
  digit: ["um número", "a digit"],
  modifier: ["uma tecla modificadora", "a modifier key"],
  navigation: ["uma tecla de navegação", "a navigation key"],
  function: ["uma tecla de função", "a function key"],
  editing: ["uma tecla de edição", "an editing key"],
  other: ["uma tecla", "a key"],
};

const L = (pair: [string, string], lang: Lang) => (lang === "pt" ? pair[0] : pair[1]);
const fingers = (n: number, lang: Lang) =>
  lang === "pt" ? `[${n}] ${n === 1 ? "dedo" : "dedos"}` : `[${n}] ${n === 1 ? "finger" : "fingers"}`;

function where(ev: InputEvent, lang: Lang): string {
  const w = ev.window;
  if (!w || (!w.title && !w.process)) return "";
  const title = w.title ? `"${w.title}"` : "";
  const proc = w.process ? ` (${w.process})` : "";
  return lang === "pt" ? ` na janela ${title}${proc}` : ` in window ${title}${proc}`;
}

export function describe(ev: InputEvent, lang: Lang): string {
  const k = ev.kind;
  const dev = L(DEVICE[ev.device_kind], lang);
  switch (k.type) {
    case "button_down":
      return lang === "pt"
        ? `Usuário pressionou o botão [${L(BUTTON[k.button], lang)}] do [${dev}]${where(ev, lang)}`
        : `User pressed the [${L(BUTTON[k.button], lang)}] button of the [${dev}]${where(ev, lang)}`;
    case "button_up":
      return lang === "pt"
        ? `Usuário soltou o botão [${L(BUTTON[k.button], lang)}] do [${dev}]`
        : `User released the [${L(BUTTON[k.button], lang)}] button of the [${dev}]`;
    case "wheel": {
      const n = Math.abs(k.delta);
      const amount = Number.isInteger(n) ? String(n) : n.toFixed(2);
      const dir = k.horizontal ? (k.delta > 0 ? "right" : "left") : k.delta > 0 ? "up" : "down";
      return lang === "pt"
        ? `Usuário rolou a roda do [${dev}] para [${L(DIR[dir], lang)}] (${amount} ${n === 1 ? "passo" : "passos"})`
        : `User scrolled the [${dev}] wheel [${L(DIR[dir], lang)}] (${amount} ${n === 1 ? "notch" : "notches"})`;
    }
    case "key": {
      const what = k.name ? `[${k.name}]` : L(KEYCLASS[k.class], lang);
      if (!k.down) return lang === "pt" ? `Tecla solta: ${what}` : `Key released: ${what}`;
      return lang === "pt" ? `Usuário pressionou ${what}${where(ev, lang)}` : `User pressed ${what}${where(ev, lang)}`;
    }
    case "contacts":
      if (k.count === 0)
        return lang === "pt"
          ? `Nenhum dedo no touchpad${k.button ? " (botão pressionado)" : ""}`
          : `No fingers on the touchpad${k.button ? " (button pressed)" : ""}`;
      return lang === "pt"
        ? `${fingers(k.count, lang)} no touchpad${k.button ? " + botão pressionado" : ""}`
        : `${fingers(k.count, lang)} on the touchpad${k.button ? " + button pressed" : ""}`;
    case "gesture":
      switch (k.gesture) {
        case "tap":
          return lang === "pt" ? `Toque com ${fingers(k.fingers, lang)} no touchpad` : `Tap with ${fingers(k.fingers, lang)} on the touchpad`;
        case "double_tap":
          return lang === "pt" ? `Toque duplo com ${fingers(k.fingers, lang)} no touchpad` : `Double tap with ${fingers(k.fingers, lang)} on the touchpad`;
        case "physical_click":
          return lang === "pt"
            ? `Clique físico no touchpad (apertou a superfície) com ${fingers(k.fingers, lang)}`
            : `Physical touchpad click (pressed the surface) with ${fingers(k.fingers, lang)}`;
        case "scroll":
          return lang === "pt"
            ? `Rolagem com ${fingers(k.fingers, lang)} para [${L(DIR[k.direction], lang)}] no touchpad`
            : `Scroll with ${fingers(k.fingers, lang)} [${L(DIR[k.direction], lang)}] on the touchpad`;
        case "swipe":
          return lang === "pt"
            ? `Deslizou ${fingers(k.fingers, lang)} para [${L(DIR[k.direction], lang)}] no touchpad`
            : `Swiped ${fingers(k.fingers, lang)} [${L(DIR[k.direction], lang)}] on the touchpad`;
        case "pinch":
          return lang === "pt"
            ? `Pinça com ${fingers(k.fingers, lang)} para [${k.kind === "out" ? "ampliar" : "reduzir"}]`
            : `Pinch with ${fingers(k.fingers, lang)} to [${k.kind === "out" ? "zoom in" : "zoom out"}]`;
        case "hold":
          return lang === "pt" ? `Manteve ${fingers(k.fingers, lang)} parados no touchpad` : `Held ${fingers(k.fingers, lang)} still on the touchpad`;
      }
      break;
    case "diagnostic":
      return k.message;
  }
  return JSON.stringify(k);
}

export function deviceLabel(kind: DeviceKind, lang: Lang): string {
  return L(DEVICE[kind], lang);
}

export const DEVICE_ICON: Record<DeviceKind, string> = {
  mouse: "🖱️",
  touchpad: "▭",
  keyboard: "⌨️",
  touchscreen: "📱",
  pen: "🖊️",
  synthetic: "⚙️",
  unknown: "❔",
};
