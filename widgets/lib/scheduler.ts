// Callback-based wrapper around timers and HTTP requests.
//
// Instead of storing timer/request ids and matching them against events by
// hand, register a callback; dispatchEvent() routes each host event to the
// right one. All ids come from a single per-widget counter, so one global
// callback table serves every app sharing this widget scope (e.g. all
// carousel slides).
//
// AssemblyScript note: closures cannot capture locals — use module-level
// functions or arrows that only touch module-level state as callbacks.
//
//   Timers.every(Duration.fromMs(200), () => { animFrame++; });
//   Fetch.get(url, onResponse);

import { Time } from "./bindings/time";
import { Http } from "./bindings/http";
import { Duration } from "./bindings/types";
import {
  WidgetEvent,
  EVENT_TIMER_INTERRUPT,
  TimerInterruptEvent,
  EVENT_HTTP_RESPONSE,
  HttpResponseEvent,
} from "./bindings/events";

const time = new Time();
const http = new Http();

class TimerEntry {
  cb: () => void;
  recurring: bool;
  constructor(cb: () => void, recurring: bool) {
    this.cb = cb;
    this.recurring = recurring;
  }
}

const timerCallbacks = new Map<u32, TimerEntry>();
const httpCallbacks = new Map<u32, (res: HttpResponseEvent) => void>();

export class Timers {
  /** Recurring timer — cb fires every `duration` until cancelled. */
  static every(duration: Duration, cb: () => void): u32 {
    const id = time.setTimeout(duration).recurring(true).execute();
    timerCallbacks.set(id, new TimerEntry(cb, true));
    return id;
  }

  /** One-shot timer — cb fires once after `duration`. */
  static once(duration: Duration, cb: () => void): u32 {
    const id = time.setTimeout(duration).recurring(false).execute();
    timerCallbacks.set(id, new TimerEntry(cb, false));
    return id;
  }

  /** Cancel a timer created with every() or once(). */
  static cancel(id: u32): void {
    if (timerCallbacks.has(id)) {
      timerCallbacks.delete(id);
      time.clearTimeout(id).execute();
    }
  }
}

export class Fetch {
  /** Send a request; cb receives the HttpResponseEvent (check res.success). */
  static request(
    method: string,
    url: string,
    body: string,
    headers: string[][],
    cb: (res: HttpResponseEvent) => void
  ): u32 {
    const id = http.fetch(method, url, body, headers).execute();
    httpCallbacks.set(id, cb);
    return id;
  }

  /** Convenience GET. */
  static get(url: string, cb: (res: HttpResponseEvent) => void): u32 {
    return Fetch.request("GET", url, "", [], cb);
  }
}

/** Route one host event to its registered callback (if any). */
export function dispatchEvent(ev: WidgetEvent): void {
  if (ev.type == EVENT_TIMER_INTERRUPT) {
    const t = ev as TimerInterruptEvent;
    if (timerCallbacks.has(t.timerId)) {
      const entry = timerCallbacks.get(t.timerId);
      if (!entry.recurring) timerCallbacks.delete(t.timerId);
      entry.cb();
    }
  } else if (ev.type == EVENT_HTTP_RESPONSE) {
    const res = ev as HttpResponseEvent;
    if (httpCallbacks.has(res.requestId)) {
      const cb = httpCallbacks.get(res.requestId);
      httpCallbacks.delete(res.requestId);
      cb(res);
    }
  }
}
