// Weather app — Open-Meteo current conditions.
// Animated icon + temperature + condition label.
// Designed for a 64×23 area.

import { Drawer } from "../bindings/drawer";
import { Console } from "../bindings/console";
import { Config } from "../config";
import { SubWidget } from "../widget";
import { Timers, Fetch } from "../scheduler";
import { HttpResponseEvent } from "../bindings/events";
import { Color, Point, Rect, Duration, Font, Baseline } from "../bindings/types";
import { Palette } from "../palette";

const drawer = new Drawer();
const console = new Console();

// --- Config (loaded during setup) ---
let CFG_LAT: string = "52.374";
let CFG_LON: string = "4.899";
let CFG_REFRESH_SEC: u64 = 30; // weather_refresh_sec — API poll interval

function loadConfig(): void {
  const cfg = new Config();
  CFG_LAT = cfg.getOr("weather_lat", "52.374");
  CFG_LON = cfg.getOr("weather_lon", "4.899");
  const refresh = <u64>parseInt(cfg.getOr("weather_refresh_sec", "30"));
  CFG_REFRESH_SEC = refresh > 0 ? refresh : 30;
}

// --- State ---
let weatherTemp: f64 = 0;
let weatherCode: i32 = -1; // -1 = not yet loaded; WMO codes 0-99
let weatherIsDay: bool = true;
let weatherInFlight: bool = false;
let animFrame: u32 = 0; // advances every 200ms — drives icon animations

// ─── Request / parsing ────────────────────────────────────────────────────────

function requestWeather(): void {
  if (!weatherInFlight) {
    console.info("Fetching weather data").execute();
    Fetch.get(
      "https://api.open-meteo.com/v1/forecast?latitude=" + CFG_LAT
        + "&longitude=" + CFG_LON
        + "&current=temperature_2m,weather_code,is_day",
      onWeatherResponse
    );
    weatherInFlight = true;
  }
}

function onWeatherResponse(res: HttpResponseEvent): void {
  weatherInFlight = false;
  if (res.success) {
    const temp = extractJsonNumber(res.body, "temperature_2m");
    const code = extractJsonNumber(res.body, "weather_code");
    const isDay = extractJsonNumber(res.body, "is_day");
    if (temp == temp) weatherTemp = temp; // NaN != NaN guard
    if (code == code) weatherCode = <i32>code;
    if (isDay == isDay) weatherIsDay = isDay != 0;
    console.info("Weather updated: " + formatTempNumber(weatherTemp) + "C " + weatherLabel(weatherCode)).execute();
  } else {
    console.error("Weather request failed").execute();
  }
}

// Find the first numeric value after `"key":` in a JSON string.
// Returns NaN if not found or value is a string.
function extractJsonNumber(body: string, key: string): f64 {
  const search = '"' + key + '":';
  let from: i32 = 0;
  while (true) {
    const idx = body.indexOf(search, from);
    if (idx < 0) return NaN;
    const vs = idx + search.length;
    if (vs >= body.length) return NaN;
    const c = body.charCodeAt(vs);
    // Numeric value starts with digit, '-', or '.'
    if ((c >= 48 && c <= 57) || c == 45 || c == 46) {
      let end = vs;
      while (end < body.length) {
        const ch = body.charCodeAt(end);
        if ((ch < 48 || ch > 57) && ch != 46 && ch != 45 && ch != 43 && ch != 101 && ch != 69) break;
        end++;
      }
      return parseFloat(body.substring(vs, end));
    }
    from = idx + 1; // this key had a string value; try next occurrence
  }
  return NaN; // unreachable; satisfies the type checker
}

function formatTempNumber(t: f64): string {
  return (<i32>Math.round(t)).toString();
}

function weatherLabel(code: i32): string {
  if (code == 0)                    return "Clear";
  if (code <= 2)                    return "P.Cloudy";
  if (code == 3)                    return "Overcast";
  if (code == 45 || code == 48)     return "Fog";
  if (code <= 57)                   return "Drizzle";
  if (code <= 67)                   return "Rain";
  if (code <= 77)                   return "Snow";
  if (code <= 82)                   return "Showers";
  if (code <= 99)                   return "Storm";
  return "...";
}

// ─── Icon drawing ─────────────────────────────────────────────────────────────
// Icon area: 14×14px, top-left at (x, y).
// All icons animate, driven by animFrame (advances every 200ms).

// Gentle horizontal drift: 0,1,2,1 repeating (slowed 2× → full cycle 1.6s)
function driftX(): u32 {
  const seq: u32[] = [0, 1, 2, 1];
  return seq[<i32>((animFrame >> 1) % 4)];
}

// Cloud: smooth ellipse base + two circle bumps.
// cx/cy = centre of the base ellipse.
function drawCloud(cx: u32, cy: u32, col: Color): void {
  drawer.ellipse(new Rect(cx - 6, cy - 2, 13, 5)).fill_color(col).fill(true).execute();
  drawer.circle(new Point(cx - 2, cy - 3), 3).fill_color(col).fill(true).execute();
  drawer.circle(new Point(cx + 3, cy - 2), 2).fill_color(col).fill(true).execute();
}

// Sun: r=3 core + 8 rays that slowly rotate around it.
function drawSun(cx: u32, cy: u32): void {
  const yel = Palette.SUN;
  drawer.circle(new Point(cx, cy), 3).fill_color(yel).fill(true).execute();
  const spin: f64 = <f64>(animFrame % 6) * 7.5; // 45° cycle in 6 frames
  for (let i = 0; i < 8; i++) {
    const a = (<f64>i * 45.0 + spin) * Math.PI / 180.0;
    const c = Math.cos(a);
    const s = Math.sin(a);
    const x1 = <u32>Math.round(<f64>cx + c * 4.5);
    const y1 = <u32>Math.round(<f64>cy + s * 4.5);
    const x2 = <u32>Math.round(<f64>cx + c * 6.0);
    const y2 = <u32>Math.round(<f64>cy + s * 6.0);
    drawer.line(new Point(x1, y1), new Point(x2, y2)).color(yel).execute();
  }
}

// Falling rain streaks — each column cycles down at its own phase.
function drawRain(cx: u32, cy: u32): void {
  const blu = Palette.RAIN;
  for (let i = 0; i < 3; i++) {
    const dx: u32 = cx - 3 + <u32>i * 3;
    const dy: u32 = (animFrame + <u32>i) % 3;
    drawer.line(new Point(dx, cy + dy), new Point(dx - 1, cy + dy + 2))
      .color(blu).thickness(2).execute();
  }
}

// Falling plus-shaped snowflakes with a slight sideways wobble.
function drawSnow(cx: u32, cy: u32): void {
  const wht = Palette.SNOW;
  for (let i = 0; i < 3; i++) {
    const fx: u32 = cx - 4 + <u32>i * 4 + ((animFrame + <u32>i) % 2);
    const fy: u32 = cy + ((animFrame + <u32>i * 2) % 4);
    drawer.line(new Point(fx - 1, fy), new Point(fx + 1, fy)).color(wht).execute();
    drawer.line(new Point(fx, fy - 1), new Point(fx, fy + 1)).color(wht).execute();
  }
}

// Light drizzle: sparse single-pixel droplets (vs. rain's thick streaks).
function drawDrizzle(cx: u32, cy: u32): void {
  const blu = Palette.RAIN;
  for (let i = 0; i < 3; i++) {
    const dx: u32 = cx - 3 + <u32>i * 3;
    const dy: u32 = (animFrame + <u32>i * 2) % 4;
    drawer.line(new Point(dx, cy + dy), new Point(dx, cy + dy)).color(blu).execute();
  }
}

// Crescent moon: full disc + offset background-colored disc bite,
// with twinkling stars around it.
function drawMoon(cx: u32, cy: u32): void {
  const moon = Palette.MOON;
  const star = Palette.STAR;

  drawer.circle(new Point(cx, cy), 4).fill_color(moon).fill(true).execute();
  // Bite: background-colored circle offset upper-right → crescent
  drawer.circle(new Point(cx + 2, cy - 2), 4).fill_color(Color.BLACK).fill(true).execute();

  // Two stars twinkling out of phase
  if ((animFrame >> 1) % 2 == 0) {
    drawer.line(new Point(cx + 4, cy - 1), new Point(cx + 4, cy - 1)).color(star).execute();
  }
  if ((animFrame >> 1) % 2 == 1) {
    drawer.line(new Point(cx + 2, cy + 4), new Point(cx + 2, cy + 4)).color(star).execute();
  }
}

// Lightning bolt: two filled triangles for a proper tapered zigzag.
function drawBolt(cx: u32, cy: u32, col: Color): void {
  drawer.triangle(new Point(cx - 1, cy), new Point(cx + 2, cy), new Point(cx - 2, cy + 4))
    .fill_color(col).fill(true).execute();
  drawer.triangle(new Point(cx, cy + 2), new Point(cx + 2, cy + 2), new Point(cx - 1, cy + 6))
    .fill_color(col).fill(true).execute();
}

function drawWeatherIcon(code: i32, x: u32, y: u32): void {
  const cx: u32 = x + 7;
  const gray     = Palette.CLOUD;
  const darkGray = Palette.CLOUD_DARK;
  const fogGray  = Palette.FOG;

  if (code == 0) {
    // Clear: rotating-ray sun by day, crescent moon by night
    if (weatherIsDay) drawSun(cx, y + 7);
    else              drawMoon(cx, y + 7);

  } else if (code <= 2) {
    // Partly cloudy: sun/moon peeking top-right, cloud drifting in front
    if (weatherIsDay) {
      drawer.circle(new Point(x + 9, y + 4), 3).fill_color(Palette.SUN).fill(true).execute();
    } else {
      drawer.circle(new Point(x + 9, y + 4), 3).fill_color(Palette.MOON).fill(true).execute();
      drawer.circle(new Point(x + 11, y + 2), 3).fill_color(Color.BLACK).fill(true).execute();
    }
    drawCloud(x + 4 + driftX(), y + 10, gray);

  } else if (code == 3) {
    // Overcast: big cloud drifting slowly
    drawCloud(x + 6 + driftX(), y + 8, gray);

  } else if (code == 45 || code == 48) {
    // Fog: three bands waving side to side out of phase
    for (let i = 0; i < 3; i++) {
      const seq: u32[] = [0, 1, 2, 1];
      const dx: u32 = seq[<i32>(((animFrame >> 1) + <u32>i) % 4)];
      const by: u32 = y + 3 + <u32>i * 4;
      drawer.line(new Point(x + 1 + dx, by), new Point(x + 11 + dx, by))
        .color(fogGray).thickness(2).execute();
    }

  } else if (code >= 51 && code <= 57) {
    // Drizzle: cloud + light droplets
    drawCloud(cx, y + 6, gray);
    drawDrizzle(cx, y + 10);

  } else if ((code >= 58 && code <= 67) || (code >= 80 && code <= 82)) {
    // Rain: cloud + falling streaks
    drawCloud(cx, y + 6, gray);
    drawRain(cx, y + 10);

  } else if (code >= 71 && code <= 77) {
    // Snow: cloud + falling flakes
    drawCloud(cx, y + 6, gray);
    drawSnow(cx, y + 10);

  } else {
    // Storm: dark cloud + flashing bolt (white flash → yellow → hidden)
    drawCloud(cx, y + 5, darkGray);
    const phase = animFrame % 6;
    if (phase == 0) {
      drawBolt(cx, y + 8, Palette.FLASH); // flash!
    } else if (phase < 4) {
      drawBolt(cx, y + 8, Palette.BOLT);
    }
    // phases 4–5: bolt hidden
  }
}

// ─── App ──────────────────────────────────────────────────────────────────────

export class WeatherApp extends SubWidget {
  setup(): void {
    loadConfig();
    Timers.every(Duration.fromSec(CFG_REFRESH_SEC), requestWeather);
    Timers.every(Duration.fromMs(200), () => { animFrame++; });
    requestWeather();
  }

  draw(): void {
    drawer.clear().execute();

    // Hairline divider on the band's last row
    drawer.line(new Point(0, 22), new Point(63, 22)).color(Palette.DIVIDER).execute();

    // Not loaded yet: three chasing dots in the band centre
    if (weatherCode < 0) {
      const active = <i32>((animFrame >> 1) % 3);
      for (let i = 0; i < 3; i++) {
        const col = i == active ? Palette.CLOUD : Palette.CLOUD_DARK;
        drawer.circle(new Point(26 + <u32>i * 6, 11), 1).fill_color(col).fill(true).execute();
      }
      return;
    }

    // Icon — 14×14 at (2,1)
    drawWeatherIcon(weatherCode, 2, 1);

    // Temperature with drawn ° symbol — orange, bold
    const tempCol = Palette.ORANGE;
    const tempNum = formatTempNumber(weatherTemp);
    const numW: u32 = <u32>tempNum.length * 7; // Font7x13Bold advance

    drawer.text(tempNum, new Point(20, 1))
      .color(tempCol).font(Font.Font7x13Bold).baseline(Baseline.Top).execute();
    // Degree symbol: small stroked circle at superscript position
    drawer.circle(new Point(20 + numW + 2, 3), 1)
      .fill(false).stroke_color(tempCol).stroke_width(1).execute();
    drawer.text("C", new Point(20 + numW + 5, 1))
      .color(tempCol).font(Font.Font7x13Bold).baseline(Baseline.Top).execute();

    // Condition label
    drawer.text(weatherLabel(weatherCode), new Point(20, 16))
      .color(Palette.TEXT_MUTED).font(Font.Font4x6).baseline(Baseline.Top).execute();
  }
}
