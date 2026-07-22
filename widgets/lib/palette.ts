// Shared color palette for all widgets.
// Keep every color used on the display in here so the three bands
// stay visually consistent and tweaks happen in one place.
//
// Hierarchy:
//   hero    — the one big number in each band (clock, temperature)
//   support — secondary info (date, condition label, public IP)
//   faint   — tertiary info (seconds, internal IP)

import { Color } from "./bindings/types";

export class Palette {
  // ── Text hierarchy ─────────────────────────────────────────────
  /** Muted blue-gray — support text (condition label). */
  static readonly TEXT_MUTED: Color = new Color(150, 165, 195);
  /** Faint blue-gray — tertiary text (internal IP). */
  static readonly TEXT_FAINT: Color = new Color(110, 130, 170);
  /** Near-black hairline that separates the widget bands. */
  static readonly DIVIDER: Color = new Color(25, 30, 45);

  // ── Accents ────────────────────────────────────────────────────
  /** Hero cyan — the big clock. */
  static readonly CYAN: Color = new Color(0, 220, 255);
  /** Dim cyan — the small seconds counter. */
  static readonly CYAN_DIM: Color = new Color(0, 140, 170);
  /** Amber — the date line. */
  static readonly AMBER: Color = new Color(255, 170, 0);
  /** Warm orange — the temperature. */
  static readonly ORANGE: Color = new Color(255, 130, 40);
  /** Green — public IP text and globe land (shared on purpose). */
  static readonly GREEN: Color = new Color(50, 210, 90);
  /** Alert red — offline slash + OFFLINE text. */
  static readonly RED: Color = new Color(255, 60, 50);

  // ── Weather icon colors ────────────────────────────────────────
  static readonly SUN: Color = new Color(255, 200, 20);
  static readonly MOON: Color = new Color(230, 230, 180);
  static readonly STAR: Color = new Color(255, 255, 220);
  static readonly CLOUD: Color = new Color(190, 195, 210);
  static readonly CLOUD_DARK: Color = new Color(105, 105, 130);
  static readonly FOG: Color = new Color(150, 160, 175);
  static readonly RAIN: Color = new Color(60, 150, 255);
  static readonly SNOW: Color = new Color(220, 240, 255);
  static readonly BOLT: Color = new Color(255, 220, 0);
  static readonly FLASH: Color = new Color(255, 255, 255);

  // ── Globe (online) ─────────────────────────────────────────────
  static readonly OCEAN: Color = new Color(20, 90, 200);
  static readonly OCEAN_DEEP: Color = new Color(10, 55, 140);
  static readonly GLOBE_RIM: Color = new Color(130, 200, 255);
  static readonly GLOBE_GRID: Color = new Color(90, 160, 240);

  // ── Globe (offline, desaturated) ───────────────────────────────
  static readonly OFF_OCEAN: Color = new Color(45, 55, 70);
  static readonly OFF_OCEAN_DEEP: Color = new Color(30, 38, 50);
  static readonly OFF_LAND: Color = new Color(75, 85, 95);
  static readonly OFF_RIM: Color = new Color(100, 110, 125);
  static readonly OFF_GRID: Color = new Color(70, 80, 95);
}
