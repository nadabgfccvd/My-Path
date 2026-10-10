// Espelho dos tipos de `crates/mp-core` (serializados pelo serde).

export type DeviceKind = "mouse" | "touchpad" | "keyboard" | "touchscreen" | "pen" | "synthetic" | "unknown";
export type MouseButton = "left" | "right" | "middle" | "x1" | "x2";
export type Direction = "up" | "down" | "left" | "right";
export type KeyClass = "letter" | "digit" | "modifier" | "navigation" | "function" | "editing" | "other";

export interface Contact { id: number; x: number; y: number }

export type Gesture =
  | { gesture: "tap"; fingers: number }
  | { gesture: "double_tap"; fingers: number }
  | { gesture: "physical_click"; fingers: number }
  | { gesture: "scroll"; fingers: number; direction: Direction }
  | { gesture: "swipe"; fingers: number; direction: Direction }
  | { gesture: "pinch"; fingers: number; kind: "out" | "in" }
  | { gesture: "hold"; fingers: number };

export type EventKind =
  | { type: "button_down"; button: MouseButton }
  | { type: "button_up"; button: MouseButton }
  | { type: "wheel"; delta: number; horizontal: boolean }
  | { type: "key"; class: KeyClass; down: boolean; name: string | null }
  | { type: "contacts"; count: number; contacts: Contact[]; button: boolean }
  | ({ type: "gesture" } & Gesture)
  | { type: "diagnostic"; level: string; message: string };

export interface WindowInfo { title: string | null; process: string | null }

export interface InputEvent {
  t_ms: number;
  device_id: string;
  device_kind: DeviceKind;
  kind: EventKind;
  cursor: [number, number] | null;
  window: WindowInfo | null;
  classification: string | null;
}

export interface DeviceInfo {
  id: string;
  kind: DeviceKind;
  name: string;
  vendor_id: number | null;
  product_id: number | null;
  notes: string[];
}

export interface Environment {
  os_family: string;
  os_name: string | null;
  os_version: string | null;
  kernel_version: string | null;
  arch: string;
  hostname_hash: string | null;
  session_type: string | null;
  desktop: string | null;
  app_version: string;
  notes: string[];
}

export interface EnvReport {
  env: Environment;
  cpu_brand: string;
  cpu_logical: number;
  cpu_physical: number | null;
  total_ram_mb: number;
  webview_version: string | null;
}

export interface Stats {
  cpu_self: number;
  cpu_tree: number;
  ram_self_mb: number;
  ram_tree_mb: number;
  processes_in_tree: number;
}

export interface MonitorInfo { name: string | null; width: number; height: number; scale: number; x: number; y: number }

/** Evento recebido pela interface, com a latência medida na chegada. */
export interface LiveEvent extends InputEvent { seq: number; latency_ms: number }
