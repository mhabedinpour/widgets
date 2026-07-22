// Time widget — standalone wrapper around the TimeApp.
// Placement: 64×30 band at the top of the display.

import { Runner } from "./lib/widget";
import { TimeApp } from "./lib/apps/time";

const runner = new Runner(new TimeApp());

export function render(): void {
  runner.render();
}
