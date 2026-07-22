// Base class for composable widget apps.
//
// An app can run standalone (via a thin top-level wrapper that owns the
// event loop) or inside the carousel widget, which only draws the active
// app. Timers and HTTP responses are handled through callbacks registered
// with lib/scheduler — apps never see raw events, so hidden carousel
// slides keep their state warm automatically.

import { pollEvent } from "./bindings/events";
import { dispatchEvent } from "./scheduler";

export abstract class SubWidget {
  /** One-time setup: register timer/HTTP callbacks, kick off fetches. */
  abstract setup(): void;
  /** Draw a full frame, including the clear. Only called when visible. */
  abstract draw(): void;
}

/** Standalone runner: drives a single app as a top-level widget. */
export class Runner {
  private app: SubWidget;
  private setupDone: bool = false;

  constructor(app: SubWidget) {
    this.app = app;
  }

  render(): void {
    if (!this.setupDone) {
      this.app.setup();
      this.setupDone = true;
    }
    let ev = pollEvent();
    while (ev !== null) {
      dispatchEvent(ev);
      ev = pollEvent();
    }
    this.app.draw();
  }
}
