// Network app — connectivity status, public + internal IP,
// with an animated spinning globe (dimmed + slashed when offline).
// Designed for a 64×11 area.

import { Drawer } from "../bindings/drawer";
import { Console } from "../bindings/console";
import { Config } from "../config";
import { Network } from "../bindings/network";
import { SubWidget } from "../widget";
import { Timers, Fetch } from "../scheduler";
import { HttpResponseEvent } from "../bindings/events";
import { Point, Rect, Duration, Font, TextAlignment, Baseline } from "../bindings/types";
import { Palette } from "../palette";

const drawer = new Drawer();
const console = new Console();
const network = new Network();

// --- State ---
let publicIp: string = "...";
let ipInFlight: bool = false;
let wasConnected: bool = false;
let animFrame: u32 = 0; // advances every 200ms — drives the globe animation
let pulseFrames: i32 = 0; // >0 → globe rim flashes bright (IP just refreshed)

// ─── Requests / helpers ───────────────────────────────────────────────────────

function requestIp(): void {
  if (!ipInFlight) {
    console.info("Fetching public IP address").execute();
    Fetch.get("https://api.ipify.org/", onIpResponse);
    ipInFlight = true;
  }
}

function onIpResponse(res: HttpResponseEvent): void {
  ipInFlight = false;
  if (res.success) {
    publicIp = res.body;
    pulseFrames = 4; // flash the globe rim for ~0.8s
    console.info("Public IP: " + publicIp).execute();
  } else {
    console.error("Public IP request failed").execute();
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
// and the land blobs slide across with it. Gray + slashed when offline.

function drawGlobe(cx: u32, cy: u32, online: bool): void {
  const ocean     = online ? Palette.OCEAN      : Palette.OFF_OCEAN;
  const oceanDeep = online ? Palette.OCEAN_DEEP : Palette.OFF_OCEAN_DEEP;
  const land      = online ? Palette.GREEN      : Palette.OFF_LAND;
  const grid      = online ? Palette.GLOBE_GRID : Palette.OFF_GRID;
  // Rim flashes white for ~0.8s after a successful IP refresh
  const rim = pulseFrames > 0 ? Palette.FLASH
            : online          ? Palette.GLOBE_RIM
            :                   Palette.OFF_RIM;

  // Ocean base (r=4) + lower-right shading for sphere depth
  drawer.circle(new Point(cx, cy), 4).fill_color(ocean).fill(true).execute();
  drawer.sector(new Point(cx, cy), 4).angle_start(0).angle_sweep(90)
    .fill_color(oceanDeep).fill(true).execute();

  // Land blobs sliding across the face (rotation keyframes; frozen when offline)
  const phase: i32 = online ? <i32>((animFrame >> 1) % 4) : 0;
  const landX: u32[] = [0, 1, 2, 1]; // sweep left → right → back
  drawer.ellipse(new Rect(cx - 3 + landX[phase], cy - 3, 4, 3))
    .fill_color(land).fill(true).execute();
  // Second, smaller land mass in the southern half, half a cycle behind
  const phase2: i32 = (phase + 2) % 4;
  drawer.ellipse(new Rect(cx - 1 + landX[phase2], cy + 1, 3, 2))
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
      .color(Palette.RED).thickness(2).execute();
  }
}

// ─── App ──────────────────────────────────────────────────────────────────────

export class NetworkApp extends SubWidget {
  setup(): void {
    // network_refresh_sec — public IP re-check interval (default 20s)
    const cfg = new Config();
    const refresh = <u64>parseInt(cfg.getOr("network_refresh_sec", "20"));
    Timers.every(Duration.fromSec(refresh > 0 ? refresh : 20), () => {
      if (network.isConnected().execute() != 0) requestIp();
    });
    Timers.every(Duration.fromMs(200), () => {
      animFrame++;
      if (pulseFrames > 0) pulseFrames--;
    });
  }

  draw(): void {
    const online = network.isConnected().execute() != 0;

    // Fetch public IP on first connect / reconnect
    if (online && !wasConnected) requestIp();
    wasConnected = online;

    drawer.clear().execute();

    // Globe — left, dimmed + slashed when offline
    drawGlobe(5, 5, online);

    if (!online) {
      drawer.text("OFFLINE", new Point(63, 3))
        .color(Palette.RED)
        .font(Font.Font5x8).alignment(TextAlignment.Right).baseline(Baseline.Top).execute();
      return;
    }

    // Public IP — top line, green, tiny 3×3 font.
    // While the first fetch is pending, show a growing "..." animation.
    let ipText = publicIp;
    if (publicIp == "...") {
      const n = 1 + <i32>((animFrame >> 2) % 3);
      ipText = ".".repeat(n);
    }
    drawer.text(ipText, new Point(63, 1))
      .color(Palette.GREEN)
      .font(Font.U8g2Font3x3).alignment(TextAlignment.Right).baseline(Baseline.Top).execute();

    // Internal IP — bottom line, faint blue-gray, tiny 3×3 font
    const internalIp = formatIpv4(network.getInternalIp().execute());
    drawer.text(internalIp, new Point(63, 7))
      .color(Palette.TEXT_FAINT)
      .font(Font.U8g2Font3x3).alignment(TextAlignment.Right).baseline(Baseline.Top).execute();
  }
}
