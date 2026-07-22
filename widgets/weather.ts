// Weather widget — standalone wrapper around the WeatherApp.
// Placement: 64×23 band in the middle of the display.

import { Runner } from "./lib/widget";
import { WeatherApp } from "./lib/apps/weather";

const runner = new Runner(new WeatherApp());

export function render(): void {
  runner.render();
}
