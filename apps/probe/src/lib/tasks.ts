// Modo roteiro: cada tarefa sabe reconhecer o evento que a cumpre.
import type { InputEvent } from "./types";

export type Group = "mouse" | "touchpad" | "keyboard";

export interface Ctx {
  /** Teclas modificadoras seguradas no momento (nomes como "Ctrl (esq.)"). */
  modifiers: Set<string>;
}

export interface Task {
  id: string;
  group: Group;
  pt: string;
  en: string;
  hintPt?: string;
  hintEn?: string;
  /** Desafio da Fase 1 que esta tarefa verifica (PROMPT_MESTRE §5). */
  challenge: number;
  match: (ev: InputEvent, ctx: Ctx) => boolean;
}

const k = (ev: InputEvent) => ev.kind;
const down = (ev: InputEvent, button: string, kind: string) =>
  k(ev).type === "button_down" && (k(ev) as { button: string }).button === button && ev.device_kind === kind;
const gesture = (ev: InputEvent, g: string, fingers?: number, dir?: string) => {
  const e = k(ev) as { type: string; gesture?: string; fingers?: number; direction?: string };
  return e.type === "gesture" && e.gesture === g && (fingers === undefined || e.fingers === fingers) && (dir === undefined || e.direction === dir);
};
const wheel = (ev: InputEvent, up: boolean) => {
  const e = k(ev);
  return e.type === "wheel" && !e.horizontal && (up ? e.delta > 0 : e.delta < 0) && ev.device_kind === "mouse";
};

export const TASKS: Task[] = [
  // ---- Mouse ----
  { id: "m_left", group: "mouse", challenge: 2, pt: "Clique com o botão ESQUERDO do mouse", en: "Click the LEFT mouse button",
    match: (e) => down(e, "left", "mouse") },
  { id: "m_right", group: "mouse", challenge: 2, pt: "Clique com o botão DIREITO do mouse", en: "Click the RIGHT mouse button",
    match: (e) => down(e, "right", "mouse") },
  { id: "m_middle", group: "mouse", challenge: 1, pt: "Aperte a RODINHA do mouse (botão do meio)", en: "Press the mouse WHEEL (middle button)",
    match: (e) => down(e, "middle", "mouse") },
  { id: "m_wheel_up", group: "mouse", challenge: 1, pt: "Gire a rodinha do mouse para CIMA", en: "Scroll the mouse wheel UP",
    match: (e) => wheel(e, true) },
  { id: "m_wheel_down", group: "mouse", challenge: 1, pt: "Gire a rodinha do mouse para BAIXO", en: "Scroll the mouse wheel DOWN",
    match: (e) => wheel(e, false) },
  // ---- Touchpad ----
  { id: "t_tap1", group: "touchpad", challenge: 3, pt: "Dê um TOQUE leve com 1 dedo no touchpad (sem apertar)", en: "Lightly TAP the touchpad with 1 finger (don't press)",
    match: (e) => gesture(e, "tap", 1) },
  { id: "t_tap1_kind", group: "touchpad", challenge: 2, pt: "Esse toque deve virar um clique identificado como TOUCHPAD (não como mouse)",
    en: "That tap must become a click identified as TOUCHPAD (not mouse)",
    hintPt: "Precisa do “toque para clicar” ligado nas configurações do touchpad.", hintEn: "Needs “tap to click” enabled in touchpad settings.",
    match: (e) => down(e, "left", "touchpad") },
  { id: "t_click", group: "touchpad", challenge: 3, pt: "APERTE a superfície do touchpad até fazer “clique”", en: "PRESS the touchpad surface until it “clicks”",
    match: (e) => gesture(e, "physical_click") },
  { id: "t_double", group: "touchpad", challenge: 3, pt: "Dê dois toques rápidos com 1 dedo (toque duplo)", en: "Tap twice quickly with 1 finger (double tap)",
    match: (e) => gesture(e, "double_tap", 1) },
  { id: "t_tap2", group: "touchpad", challenge: 3, pt: "Toque com 2 dedos ao mesmo tempo", en: "Tap with 2 fingers at the same time",
    match: (e) => gesture(e, "tap", 2) },
  { id: "t_right_kind", group: "touchpad", challenge: 2, pt: "Faça um clique DIREITO pelo touchpad (2 dedos ou canto inferior direito)",
    en: "Right-click using the touchpad (2 fingers or lower-right corner)",
    match: (e) => down(e, "right", "touchpad") },
  { id: "t_scroll_up", group: "touchpad", challenge: 3, pt: "Arraste 2 dedos para CIMA (rolagem)", en: "Drag 2 fingers UP (scroll)",
    match: (e) => gesture(e, "scroll", 2, "up") },
  { id: "t_scroll_down", group: "touchpad", challenge: 3, pt: "Arraste 2 dedos para BAIXO (rolagem)", en: "Drag 2 fingers DOWN (scroll)",
    match: (e) => gesture(e, "scroll", 2, "down") },
  { id: "t_pinch", group: "touchpad", challenge: 3, pt: "Faça PINÇA: afaste ou aproxime 2 dedos", en: "PINCH: spread or bring 2 fingers together",
    match: (e) => gesture(e, "pinch") },
  { id: "t_tap3", group: "touchpad", challenge: 3, pt: "Toque com 3 dedos", en: "Tap with 3 fingers",
    hintPt: "O sistema pode abrir algo (ex.: pesquisa). Tudo bem, só volte ao Probe.", hintEn: "The system may open something (e.g. search). That's fine, just come back.",
    match: (e) => gesture(e, "tap", 3) },
  { id: "t_swipe3", group: "touchpad", challenge: 3, pt: "Deslize 3 dedos para os lados ou para cima/baixo", en: "Swipe 3 fingers sideways or up/down",
    hintPt: "Pode trocar de janela ou mostrar a área de trabalho. Depois volte ao Probe.", hintEn: "It may switch windows or show the desktop. Then come back.",
    match: (e) => gesture(e, "swipe", 3) },
  { id: "t_swipe4", group: "touchpad", challenge: 3, pt: "Deslize 4 dedos (se o touchpad aceitar)", en: "Swipe 4 fingers (if the touchpad supports it)",
    match: (e) => gesture(e, "swipe", 4) },
  // ---- Teclado ----
  { id: "k_letter", group: "keyboard", challenge: 1, pt: "Digite qualquer palavra (as letras NÃO são gravadas)", en: "Type any word (letters are NOT recorded)",
    match: (e) => k(e).type === "key" && (k(e) as { class: string; down: boolean }).class === "letter" && (k(e) as { down: boolean }).down },
  { id: "k_enter", group: "keyboard", challenge: 1, pt: "Aperte Enter", en: "Press Enter",
    match: (e) => k(e).type === "key" && (k(e) as { name: string | null; down: boolean }).name === "Enter" && (k(e) as { down: boolean }).down },
  { id: "k_shortcut", group: "keyboard", challenge: 1, pt: "Faça um atalho com Ctrl (ex.: Ctrl + C)", en: "Use a Ctrl shortcut (e.g. Ctrl + C)",
    match: (e, ctx) =>
      k(e).type === "key" && (k(e) as { down: boolean }).down && (k(e) as { class: string }).class === "letter" &&
      [...ctx.modifiers].some((m) => m.startsWith("Ctrl")) },
];

/** Atualiza as modificadoras seguradas a partir de um evento de tecla. */
export function trackModifiers(ev: InputEvent, ctx: Ctx) {
  const e = ev.kind;
  if (e.type === "key" && e.class === "modifier" && e.name) {
    if (e.down) ctx.modifiers.add(e.name);
    else ctx.modifiers.delete(e.name);
  }
}
