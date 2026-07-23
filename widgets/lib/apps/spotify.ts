// Spotify app — currently playing track via Spotify Web API.
// Uses the OAuth refresh-token flow: trades a refresh token for an access
// token on startup, and re-fetches one whenever the current token expires.
//
// Config keys:
//   spotify_client_id      — OAuth app client ID
//   spotify_client_secret  — OAuth app client secret
//   spotify_refresh_token  — long-lived refresh token
//   spotify_refresh_sec    — track poll interval, default 5
//
// Layout (64×19):
//   x=0..6   animated equalizer bars (pause icon when paused)
//   x=9 y=1  track name  Font5x7 scrolling
//   x=9 y=9  artist name Font4x6 truncated
//   y=16     2px progress bar with bright head dot (interpolated between polls)

import { valueStart, valueEnd, getString, getNumber, getBool } from "../jsonlite";
import { Drawer } from "../bindings/drawer";
import { Console } from "../bindings/console";
import { Config } from "../config";
import { SubWidget } from "../widget";
import { Timers, Fetch } from "../scheduler";
import { HttpResponseEvent } from "../bindings/events";
import { Color, Point, Rect, Duration, Font, Baseline } from "../bindings/types";
import { Palette } from "../palette";

const drawer  = new Drawer();
const console = new Console();

const SPOTIFY_GREEN: Color     = new Color(30, 215, 96);
const SPOTIFY_GREEN_DIM: Color = new Color(15, 110, 50);
const PROGRESS_BG: Color       = new Color(40, 45, 60);
const PROGRESS_HEAD: Color     = new Color(235, 255, 240);

// ─── Config ───────────────────────────────────────────────────────────────────

let CFG_CLIENT_ID: string     = "";
let CFG_CLIENT_SECRET: string = "";
let CFG_REFRESH_TOKEN: string = "";
let CFG_POLL_SEC: u64         = 5;

function loadConfig(): void {
  const cfg         = new Config();
  CFG_CLIENT_ID     = cfg.getOr("spotify_client_id",     "");
  CFG_CLIENT_SECRET = cfg.getOr("spotify_client_secret", "");
  CFG_REFRESH_TOKEN = cfg.getOr("spotify_refresh_token", "");
  const r           = <u64>parseInt(cfg.getOr("spotify_refresh_sec", "5"));
  CFG_POLL_SEC      = r > 0 ? r : 5;
}

function hasCredentials(): bool {
  return CFG_CLIENT_ID.length > 0
      && CFG_CLIENT_SECRET.length > 0
      && CFG_REFRESH_TOKEN.length > 0;
}

// ─── Token state ──────────────────────────────────────────────────────────────

let accessToken: string  = "";
let tokenInFlight: bool  = false;
let tokenReady: bool     = false;
let tokenRetries: i32    = 0;
const MAX_TOKEN_RETRIES: i32 = 3;

// ─── Track state ──────────────────────────────────────────────────────────────

let trackName: string   = "";
let artistName: string  = "";
let progressMs: f64     = 0;
let durationMs: f64     = 1;
let isPlaying: bool     = false;
let progressFrame: u32  = 0; // animFrame at last poll — for progress interpolation

// 0=init  1=ok  2=nothing playing  3=error
let loadState: i32      = 0;
let trackInFlight: bool = false;

// ─── Animation / scroll ───────────────────────────────────────────────────────

let animFrame: u32       = 0;
let trackStartFrame: u32 = 0;

const TRACK_MAX_CH: i32  = 11;
const SCROLL_FRAMES: u32 = 3;
const PAUSE_FRAMES: u32  = 20;

// ─── Base64 ───────────────────────────────────────────────────────────────────

function base64Encode(s: string): string {
  const B64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  let out = "";
  const n = s.length;
  let i = 0;
  while (i < n) {
    const b0 =            s.charCodeAt(i)     & 0xFF;
    const b1 = i+1 < n ? (s.charCodeAt(i+1) & 0xFF) : 0;
    const b2 = i+2 < n ? (s.charCodeAt(i+2) & 0xFF) : 0;
    out += B64.charAt( (b0 >> 2) & 0x3F);
    out += B64.charAt(((b0 &  3) << 4) | ((b1 >> 4) & 0x0F));
    out += i+1 < n ? B64.charAt(((b1 & 0xF) << 2) | ((b2 >> 6) & 0x03)) : "=";
    out += i+2 < n ? B64.charAt(  b2          & 0x3F)                    : "=";
    i += 3;
  }
  return out;
}

// ─── Token fetch ──────────────────────────────────────────────────────────────

function refreshAccessToken(): void {
  if (tokenInFlight || !hasCredentials()) return;
  tokenReady = false;

  const creds = base64Encode(CFG_CLIENT_ID + ":" + CFG_CLIENT_SECRET);
  const body  = "grant_type=refresh_token&refresh_token=" + CFG_REFRESH_TOKEN;

  Fetch.request(
    "POST",
    "https://accounts.spotify.com/api/token",
    body,
    [
      ["Authorization", "Basic " + creds],
      ["Content-Type",  "application/x-www-form-urlencoded"],
    ],
    onTokenResponse
  );
  tokenInFlight = true;
  console.info("Spotify: refreshing token").execute();
}

function onTokenResponse(res: HttpResponseEvent): void {
  tokenInFlight = false;

  if (!res.success || res.body.length == 0) {
    if (tokenRetries < MAX_TOKEN_RETRIES) {
      tokenRetries++;
      console.info("Spotify: token failed, retry " + tokenRetries.toString() + "/" + MAX_TOKEN_RETRIES.toString()).execute();
      refreshAccessToken();
    } else {
      tokenRetries = 0;
      loadState = 3;
      console.error("Spotify: token request failed after retries").execute();
    }
    return;
  }

  const token = getString(res.body, "access_token", 0, res.body.length);
  if (token == null || token.length == 0) {
    loadState = 3;
    console.error("Spotify: token parse failed: " + res.body.substring(0, 80)).execute();
    return;
  }

  tokenRetries = 0;
  accessToken  = token;
  tokenReady   = true;
  console.info("Spotify: token ready").execute();
  requestNowPlaying();
}

// ─── Track fetch ──────────────────────────────────────────────────────────────

function requestNowPlaying(): void {
  if (trackInFlight || !tokenReady) return;

  Fetch.request(
    "GET",
    "https://api.spotify.com/v1/me/player/currently-playing?market=from_token",
    "",
    [["Authorization", "Bearer " + accessToken]],
    onNowPlayingResponse
  );
  trackInFlight = true;
}

function onNowPlayingResponse(res: HttpResponseEvent): void {
  trackInFlight = false;

  if (!res.success) {
    loadState = 3;
    console.error("Spotify: track fetch failed").execute();
    return;
  }

  // 204 No Content → nothing is playing
  if (res.body.length == 0) {
    loadState = 2;
    return;
  }

  // 401 → token expired
  if (res.body.indexOf('"status":401') >= 0 || res.body.indexOf('"status": 401') >= 0) {
    console.info("Spotify: 401 — refreshing token").execute();
    refreshAccessToken();
    return;
  }

  const body = res.body;

  // "item" is null when nothing is loaded in the player
  const itemStart = valueStart(body, "item", 0, body.length);
  if (itemStart < 0 || body.charCodeAt(itemStart) != 0x7B) { // not '{'
    loadState = 2;
    return;
  }
  const itemEnd = valueEnd(body, itemStart);

  // item = { album: {...}, artists: [...], disc_number, duration_ms, ..., name, ... }
  // The album object contains its own "name" and "artists" keys, so skip
  // past it before searching for the track-level fields.
  let scan = itemStart + 1;
  const albumStart = valueStart(body, "album", itemStart, itemEnd);
  if (albumStart >= 0) scan = valueEnd(body, albumStart);

  // Track artists array — take the first artist's name
  artistName = "";
  let artistsEnd = scan;
  const artistsStart = valueStart(body, "artists", scan, itemEnd);
  if (artistsStart >= 0 && body.charCodeAt(artistsStart) == 0x5B) { // '['
    artistsEnd = valueEnd(body, artistsStart);
    const firstObj = body.indexOf("{", artistsStart);
    if (firstObj >= 0 && firstObj < artistsEnd) {
      const firstEnd = valueEnd(body, firstObj);
      const an = getString(body, "name", firstObj, firstEnd);
      if (an != null) artistName = an;
    }
  }

  // duration_ms and the track's own "name" come after the artists array
  const nameStr  = getString(body, "name", artistsEnd, itemEnd);
  const duration = getNumber(body, "duration_ms", artistsEnd, itemEnd);

  // Top-level fields — keys are unique in the document, search everywhere
  const progress = getNumber(body, "progress_ms", 0, body.length);
  isPlaying      = getBool(body, "is_playing", false, 0, body.length);

  const newTrack = nameStr != null ? nameStr : "";
  if (newTrack != trackName) trackStartFrame = animFrame;
  trackName = newTrack;

  progressMs    = progress == progress ? progress : 0; // x == x → not NaN
  durationMs    = duration == duration ? duration : 1;
  progressFrame = animFrame;
  if (!(durationMs > 0)) durationMs = 1;

  if (trackName.length == 0) {
    console.error("Spotify: could not parse track name").execute();
    loadState = 2;
    return;
  }

  loadState = 1;
  console.info("Spotify: " + trackName + " – " + artistName).execute();
}

// ─── Text helpers ─────────────────────────────────────────────────────────────

function trunc(s: string, maxLen: i32): string {
  if (s.length <= maxLen) return s;
  return s.substring(0, maxLen - 1) + "~";
}

function tickerWindow(name: string): string {
  if (name.length <= TRACK_MAX_CH) return name;
  const elapsed = animFrame >= trackStartFrame ? animFrame - trackStartFrame : 0;
  if (elapsed < PAUSE_FRAMES) return name.substring(0, TRACK_MAX_CH);
  const scrolled = (elapsed - PAUSE_FRAMES) / SCROLL_FRAMES;
  const loopLen  = <u32>(name.length + 3);
  const offset   = <i32>(scrolled % loopLen);
  if (offset >= name.length) return "";
  const end = offset + TRACK_MAX_CH;
  if (end <= name.length) return name.substring(offset, end);
  return name.substring(offset);
}

// ─── Draw helpers ─────────────────────────────────────────────────────────────

// Equalizer: 4 bars at x, x+2, x+4, x+6 bouncing to their own rhythm.
// Height sequences are hand-tuned so no two bars peak together.
const EQ0: u32[] = [3, 5, 7, 4, 6, 3, 5, 6];
const EQ1: u32[] = [6, 3, 5, 7, 4, 6, 3, 5];
const EQ2: u32[] = [4, 6, 3, 5, 7, 4, 6, 3];
const EQ3: u32[] = [7, 4, 6, 3, 5, 6, 4, 7];

function eqHeight(bar: i32): u32 {
  const idx = <i32>(animFrame % 8);
  if (bar == 0) return EQ0[idx];
  if (bar == 1) return EQ1[idx];
  if (bar == 2) return EQ2[idx];
  return EQ3[idx];
}

// Bars rest on yBase and grow upward.
function drawEqualizer(x: u32, yBase: u32): void {
  for (let b = 0; b < 4; b++) {
    const h  = eqHeight(b);
    const bx = x + <u32>b * 2;
    drawer.line(new Point(bx, yBase - h + 1), new Point(bx, yBase))
      .color(SPOTIFY_GREEN).execute();
    // Dim tip pixel — gives the bars a soft "glow" cap
    drawer.line(new Point(bx, yBase - h + 1), new Point(bx, yBase - h + 1))
      .color(SPOTIFY_GREEN_DIM).execute();
  }
}

// Paused: two muted vertical bars in the equalizer slot.
function drawPausedIcon(x: u32, y: u32): void {
  drawer.rect(new Rect(x + 1, y + 1, 2, 6)).fill_color(Palette.TEXT_MUTED).fill(true).execute();
  drawer.rect(new Rect(x + 4, y + 1, 2, 6)).fill_color(Palette.TEXT_MUTED).fill(true).execute();
}

// Eighth note: head + stem + flag. Used in the "not playing" state.
function drawNote(x: u32, y: u32, col: Color): void {
  drawer.ellipse(new Rect(x, y + 7, 4, 3)).fill_color(col).fill(true).execute();
  drawer.line(new Point(x + 3, y), new Point(x + 3, y + 7)).color(col).execute();
  drawer.line(new Point(x + 3, y), new Point(x + 5, y + 2)).color(col).execute();
  drawer.line(new Point(x + 5, y + 2), new Point(x + 5, y + 4)).color(col).execute();
}

// ─── App ──────────────────────────────────────────────────────────────────────

export class SpotifyApp extends SubWidget {
  setup(): void {
    loadConfig();
    Timers.every(Duration.fromMs(200),           () => { animFrame++; });
    Timers.every(Duration.fromSec(CFG_POLL_SEC), requestNowPlaying);
    Timers.every(Duration.fromMin(55),           refreshAccessToken);
    refreshAccessToken();
  }

  draw(): void {
    drawer.clear().execute();

    if (!hasCredentials()) {
      drawer.text("spotify", new Point(0, 1))
        .color(SPOTIFY_GREEN).font(Font.Font5x7).baseline(Baseline.Top).execute();
      drawer.text("setup needed", new Point(0, 10))
        .color(Palette.TEXT_FAINT).font(Font.Font4x6).baseline(Baseline.Top).execute();
      return;
    }

    if (!tokenReady && loadState == 0) {
      const active = <i32>((animFrame >> 1) % 3);
      for (let i = 0; i < 3; i++) {
        const col = i == active ? SPOTIFY_GREEN : Palette.TEXT_FAINT;
        drawer.circle(new Point(26 + <u32>i * 6, 9), 1)
          .fill_color(col).fill(true).execute();
      }
      return;
    }

    if (loadState == 3) {
      drawer.text("spotify", new Point(0, 1))
        .color(SPOTIFY_GREEN).font(Font.Font5x7).baseline(Baseline.Top).execute();
      drawer.text("auth error", new Point(0, 10))
        .color(Palette.RED).font(Font.Font4x6).baseline(Baseline.Top).execute();
      return;
    }

    if (loadState == 2) {
      // Gently bobbing note + label
      const bob: u32 = (animFrame >> 2) % 2;
      drawNote(6, 4 + bob, Palette.TEXT_FAINT);
      drawer.text("not playing", new Point(16, 7))
        .color(Palette.TEXT_FAINT).font(Font.Font4x6).baseline(Baseline.Top).execute();
      return;
    }

    // Left icon slot: bouncing equalizer while playing, pause bars otherwise
    if (isPlaying) {
      drawEqualizer(0, 8);
    } else {
      drawPausedIcon(0, 1);
    }

    drawer.text(tickerWindow(trackName), new Point(9, 1))
      .color(Color.WHITE).font(Font.Font5x7).baseline(Baseline.Top).execute();

    drawer.text(trunc(artistName, 13), new Point(9, 9))
      .color(SPOTIFY_GREEN_DIM).font(Font.Font4x6).baseline(Baseline.Top).execute();

    // Progress bar — interpolate between polls (animFrame ticks every 200ms)
    let shownMs = progressMs;
    if (isPlaying && animFrame > progressFrame) {
      shownMs += <f64>(animFrame - progressFrame) * 200.0;
    }
    if (shownMs > durationMs) shownMs = durationMs;

    drawer.rect(new Rect(0, 16, 64, 2)).fill_color(PROGRESS_BG).fill(true).execute();
    if (durationMs > 0 && shownMs >= 0 && shownMs == shownMs) {
      let w = <u32>Math.round((shownMs / durationMs) * 64.0);
      if (w > 64) w = 64;
      if (w > 0) {
        drawer.rect(new Rect(0, 16, w, 2)).fill_color(SPOTIFY_GREEN).fill(true).execute();
        // Bright head dot at the leading edge
        drawer.rect(new Rect(w > 1 ? w - 2 : 0, 16, 2, 2))
          .fill_color(PROGRESS_HEAD).fill(true).execute();
      }
    }
  }
}
