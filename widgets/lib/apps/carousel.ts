// Carousel app — rotates any number of other apps in a single placement.
//
// Config:
//   carousel_slides      comma-separated app names, e.g. "weather,network" (order = rotation order)
//   carousel_period_sec  seconds each slide stays visible (default "10")
//
// Timers and HTTP responses are routed through lib/scheduler callbacks, so
// hidden apps keep their timers, animations, and in-flight requests warm —
// a slide swaps in with current data, not stale state. Only the active app
// draws; its clear() wipes the previous slide since all apps share this
// widget's placement.

import { Config } from "../config";
import { Console } from "../bindings/console";
import { SubWidget } from "../widget";
import { Timers } from "../scheduler";
import { Duration } from "../bindings/types";
import { TimeApp } from "./time";
import { WeatherApp } from "./weather";
import { NetworkApp } from "./network";

const console = new Console();

const apps: SubWidget[] = [];
let active: i32 = 0;

// App registry — add new apps here to make them available as slides.
function createApp(name: string): SubWidget | null {
  if (name == "time")    return new TimeApp();
  if (name == "weather") return new WeatherApp();
  if (name == "network") return new NetworkApp();
  return null;
}

function rotate(): void {
  if (apps.length > 0) active = (active + 1) % apps.length;
}

export class CarouselApp extends SubWidget {
  setup(): void {
    const cfg = new Config();
    const slides = cfg.getOr("carousel_slides", "weather,network").split(",");
    for (let i = 0; i < slides.length; i++) {
      const name = slides[i].trim();
      const app = createApp(name);
      if (app !== null) {
        apps.push(app);
      } else {
        console.error("carousel: unknown slide '" + name + "'").execute();
      }
    }

    const period = <u64>parseInt(cfg.getOr("carousel_period_sec", "10"));
    Timers.every(Duration.fromSec(period > 0 ? period : 10), rotate);

    for (let i = 0; i < apps.length; i++) apps[i].setup();
  }

  draw(): void {
    if (apps.length == 0) return;
    apps[active].draw();
  }
}
