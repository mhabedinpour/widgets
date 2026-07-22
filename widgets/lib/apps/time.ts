// Time app — date, HH:MM clock with pulsing colon, seconds.
// Designed for a 64×30 area.

import { Drawer } from "../bindings/drawer";
import { Time } from "../bindings/time";
import { Config } from "../config";
import { SubWidget } from "../widget";
import { Timers } from "../scheduler";
import { Point, Duration, Font, TextAlignment, Baseline } from "../bindings/types";
import { Palette } from "../palette";

const drawer = new Drawer();
const time = new Time();

let colonVisible: bool = true;

// --- Config (loaded once during setup) ---
let UTC_OFFSET_STD: i64 = 3600;  // standard offset in seconds (default: CET = UTC+1)
let UTC_OFFSET_DST: i64 = 7200;  // DST offset in seconds    (default: CEST = UTC+2)
let DST_START_MONTH: i32 = 3;    // month DST begins (default: March)
let DST_END_MONTH: i32 = 10;     // month DST ends   (default: October)

function loadConfig(): void {
  const cfg = new Config();
  UTC_OFFSET_STD  = <i64>parseInt(cfg.getOr("time_utc_offset",      "3600"));
  UTC_OFFSET_DST  = <i64>parseInt(cfg.getOr("time_utc_dst_offset",  "7200"));
  DST_START_MONTH = <i32>parseInt(cfg.getOr("time_dst_start_month", "3"));
  DST_END_MONTH   = <i32>parseInt(cfg.getOr("time_dst_end_month",   "10"));
}

// ─── Date / time helpers ──────────────────────────────────────────────────────

function pad2(n: i32): string {
  if (n < 10) return "0" + n.toString();
  return n.toString();
}

function isLeapYear(y: i32): bool {
  return (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
}

function dateToEpoch(year: i32, month: i32, day: i32): i64 {
  const y1: i64 = <i64>year - 1;
  const yearDays: i64 = (<i64>year - 1970) * 365 + y1 / 4 - y1 / 100 + y1 / 400 - 477;
  let monthDays: i64 = 0;
  const m = month - 1;
  if (m >= 1) monthDays += 31;
  if (m >= 2) monthDays += <i64>(isLeapYear(year) ? 29 : 28);
  if (m >= 3) monthDays += 31;
  if (m >= 4) monthDays += 30;
  if (m >= 5) monthDays += 31;
  if (m >= 6) monthDays += 30;
  if (m >= 7) monthDays += 31;
  if (m >= 8) monthDays += 31;
  if (m >= 9) monthDays += 30;
  if (m >= 10) monthDays += 31;
  if (m >= 11) monthDays += 30;
  return (yearDays + monthDays + <i64>day - 1) * 86400;
}

function dayOfWeek(epochSeconds: i64): i32 {
  return <i32>(((epochSeconds / 86400) + 4) % 7);
}

function lastSundayOf(year: i32, month: i32): i64 {
  let lastDay: i32;
  if (month == 2) lastDay = isLeapYear(year) ? 29 : 28;
  else if (month == 4 || month == 6 || month == 9 || month == 11) lastDay = 30;
  else lastDay = 31;
  const epochLastDay = dateToEpoch(year, month, lastDay);
  const dow = dayOfWeek(epochLastDay);
  return epochLastDay - <i64>dow * 86400 + 3600;
}

// Returns the UTC offset in seconds for the configured timezone.
// Uses the "last Sunday of DST_START_MONTH / DST_END_MONTH at 01:00 UTC"
// transition rule (covers all European timezones and many others).
// Set DST_START_MONTH == DST_END_MONTH to disable DST (fixed offset).
function localOffset(ts: i64): i64 {
  if (DST_START_MONTH == DST_END_MONTH) return UTC_OFFSET_STD;
  const d = new Date(ts * 1000);
  const year = d.getUTCFullYear();
  const month = d.getUTCMonth() + 1;
  if (month < DST_START_MONTH || month > DST_END_MONTH) return UTC_OFFSET_STD;
  if (month > DST_START_MONTH && month < DST_END_MONTH) return UTC_OFFSET_DST;
  const dstStart = lastSundayOf(year, DST_START_MONTH);
  const dstEnd   = lastSundayOf(year, DST_END_MONTH);
  if (ts >= dstStart && ts < dstEnd) return UTC_OFFSET_DST;
  return UTC_OFFSET_STD;
}

function getLocalDate(ts: i64): Date {
  return new Date((ts + localOffset(ts)) * 1000);
}

const DOW_NAMES = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];

function formatDate(ts: i64): string {
  const d = getLocalDate(ts);
  return DOW_NAMES[d.getUTCDay()] + " " + pad2(d.getUTCDate()) + "/" + pad2(d.getUTCMonth() + 1);
}

// HH MM with a blank middle cell — the colon is overlaid separately
// so it can pulse bright/dim instead of vanishing.
function formatTime(ts: i64): string {
  const d = getLocalDate(ts);
  return pad2(d.getUTCHours()) + " " + pad2(d.getUTCMinutes());
}

function formatSeconds(ts: i64): string {
  return pad2(getLocalDate(ts).getUTCSeconds());
}

// ─── App ──────────────────────────────────────────────────────────────────────

export class TimeApp extends SubWidget {
  setup(): void {
    loadConfig();
    Timers.every(Duration.fromSec(1), () => { colonVisible = !colonVisible; });
  }

  draw(): void {
    const ts = time.getUnixTimestamp().execute();
    drawer.clear().execute();

    // Date — "TUE 21/07", amber, centred
    drawer.text(formatDate(ts), new Point(32, 1))
      .color(Palette.AMBER)
      .font(Font.Font5x7).alignment(TextAlignment.Center).baseline(Baseline.Top).execute();

    // Big HH:MM clock — cyan, bold, centred
    drawer.text(formatTime(ts), new Point(32, 10))
      .color(Palette.CYAN)
      .font(Font.Font9x18Bold).alignment(TextAlignment.Center).baseline(Baseline.Top).execute();

    // Colon overlaid on the middle cell — pulses bright/dim once per second
    drawer.text(":", new Point(32, 10))
      .color(colonVisible ? Palette.CYAN : Palette.CYAN_DIM)
      .font(Font.Font9x18Bold).alignment(TextAlignment.Center).baseline(Baseline.Top).execute();

    // Seconds — small, dim cyan, bottom-aligned with the clock
    drawer.text(formatSeconds(ts), new Point(63, 22))
      .color(Palette.CYAN_DIM)
      .font(Font.Font4x6).alignment(TextAlignment.Right).baseline(Baseline.Top).execute();
  }
}
