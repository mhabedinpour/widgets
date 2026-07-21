// Network widget — connectivity status, public + internal IP,
// with an animated spinning globe (dimmed + slashed when offline).
// Placement: 64×11 band at the bottom of the display.

import { Drawer } from "./lib/drawer";
import { Http } from "./lib/http";
import { Time } from "./lib/time";
import { Console } from "./lib/console";
import { Network } from "./lib/network";
import { pollEvent, EVENT_HTTP_RESPONSE, HttpResponseEvent, EVENT_TIMER_INTERRUPT, TimerInterruptEvent } from "./lib/events";
import { Color, Point, Rect, Duration, Font, TextAlignment, Baseline } from "./lib/types";

const drawer = new Drawer();
const http = new Http();
const time = new Time();
const console = new Console();
const network = new Network();

// --- State ---
let publicIp: string = "...";
let IP_REQUEST_ID: u32 = 0;
let ipInFlight: bool = false;
let wasConnected: bool = false;

// --- Timers ---
let FETCH_TIMER_ID: u32 = 0;
let ANIM_TIMER_ID: u32 = 0;
let timersSetup: bool = false;
let animFrame: u32 = 0; // advances every 200ms — drives the globe animation

// ─── Requests / helpers ───────────────────────────────────────────────────────

function requestIp(): void {
  if (!ipInFlight) {
    console.info("Fetching public IP address").execute();
    IP_REQUEST_ID = http.fetch("GET", "https://api.ipify.org/", "", []).execute();
    ipInFlight = true;
  }
}

// Decode a big-endian packed IPv4 (Ipv4Address::to_bits) into dotted form.
function formatIpv4(bits: u32): string {
  if (bits == 0) return "...";
  return ((bits >> 24) & 0xFF).toString() + "." +
         ((bits >> 16) & 0xFF).toString() + "." +
         ((bits >> 8) & 0xFF).toString() + "." +
         (bits & 0xFF).toString();
}

// ─── Globe ────────────────────────────────────────────────────────────────────
// Spinning earth: a meridian ellipse narrows/widens to suggest rotation,
// and the land blob slides across with it. Gray + slashed when offline.

function drawGlobe(cx: u32, cy: u32, online: bool): void {
  const ocean     = online ? new Color(20, 90, 200)   : new Color(45, 55, 70);
  const oceanDeep = online ? new Color(10, 55, 140)   : new Color(30, 38, 50);
  const land      = online ? new Color(60, 210, 90)   : new Color(75, 85, 95);
  const rim       = online ? new Color(130, 200, 255) : new Color(100, 110, 125);
  const grid      = online ? new Color(90, 160, 240)  : new Color(70, 80, 95);

  // Ocean base (r=4) + lower-right shading for sphere depth
  drawer.circle(new Point(cx, cy), 4).fill_color(ocean).fill(true).execute();
  drawer.sector(new Point(cx, cy), 4).angle_start(0).angle_sweep(90)
    .fill_color(oceanDeep).fill(true).execute();

  // Land blob sliding across the face (rotation keyframes; frozen when offline)
  const phase: i32 = online ? <i32>((animFrame >> 1) % 4) : 0;
  const landX: u32[] = [0, 1, 2, 1]; // sweep left → right → back
  drawer.ellipse(new Rect(cx - 3 + landX[phase], cy - 3, 4, 3))
    .fill_color(land).fill(true).execute();

  // Equator
  drawer.line(new Point(cx - 3, cy), new Point(cx + 3, cy)).color(grid).execute();

  // Rotating meridian: ellipse narrows to a line and back
  const mw: u32[] = [1, 4, 7, 4]; // meridian ellipse width per phase
  const w = mw[phase];
  if (w <= 1) {
    drawer.line(new Point(cx, cy - 3), new Point(cx, cy + 3)).color(grid).execute();
  } else {
    drawer.ellipse(new Rect(cx - w / 2, cy - 3, w, 7))
      .fill(false).stroke_color(grid).stroke_width(1).execute();
  }

  // Rim
  drawer.circle(new Point(cx, cy), 4).fill(false).stroke_color(rim).stroke_width(1).execute();

  // Offline: red slash across the globe
  if (!online) {
    drawer.line(new Point(cx - 4, cy + 4), new Point(cx + 4, cy - 4))
      .color(new Color(255, 60, 50)).thickness(2).execute();
  }
}

// ─── Render ───────────────────────────────────────────────────────────────────

export function render(): void {
  if (!timersSetup) {
    FETCH_TIMER_ID = time.setTimeout(Duration.fromSec(20)).recurring(true).execute();
    ANIM_TIMER_ID  = time.setTimeout(Duration.fromMs(200)).recurring(true).execute();
    timersSetup = true;
  }

  const online = network.isConnected().execute() != 0;

  // Fetch public IP on first connect / reconnect
  if (online && !wasConnected) requestIp();
  wasConnected = online;

  let ev = pollEvent();
  while (ev !== null) {
    if (ev.type == EVENT_TIMER_INTERRUPT) {
      const t = ev as TimerInterruptEvent;
      if      (t.timerId == FETCH_TIMER_ID) { if (online) requestIp(); }
      else if (t.timerId == ANIM_TIMER_ID)  animFrame++;

    } else if (ev.type == EVENT_HTTP_RESPONSE) {
      const res = ev as HttpResponseEvent;
      if (res.requestId == IP_REQUEST_ID) {
        ipInFlight = false;
        if (res.success) {
          publicIp = res.body;
          console.info("Public IP: " + publicIp).execute();
        } else {
          console.error("Public IP request failed").execute();
        }
      }
    }
    ev = pollEvent();
  }

  drawer.clear().execute();

  // Globe — left, dimmed + slashed when offline
  drawGlobe(5, 5, online);

  if (!online) {
    drawer.text("OFFLINE", new Point(63, 3))
      .color(new Color(255, 60, 50))
      .font(Font.Font5x8).alignment(TextAlignment.Right).baseline(Baseline.Top).execute();
    return;
  }

  // Public IP — top line, green, tiny 3×5 font
  drawer.text(publicIp, new Point(63, 0))
    .color(new Color(0, 200, 90))
    .font(Font.U8g2Font3x3).alignment(TextAlignment.Right).baseline(Baseline.Top).execute();

  // Internal IP — bottom line, dim blue-gray, tiny 3×5 font
  const internalIp = formatIpv4(network.getInternalIp().execute());
  drawer.text(internalIp, new Point(63, 6))
    .color(new Color(110, 130, 170))
    .font(Font.U8g2Font3x3).alignment(TextAlignment.Right).baseline(Baseline.Top).execute();
}
