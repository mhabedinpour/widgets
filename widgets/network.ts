// Network widget — standalone wrapper around the NetworkApp.
// Placement: 64×11 band at the bottom of the display.

import { Runner } from "./lib/widget";
import { NetworkApp } from "./lib/apps/network";

const runner = new Runner(new NetworkApp());

export function render(): void {
  runner.render();
}
