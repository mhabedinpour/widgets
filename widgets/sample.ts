import { Drawer } from "./lib/drawer";
import { Http } from "./lib/http";
import { pollEvent, EVENT_HTTP_RESPONSE, HttpResponseEvent } from "./lib/events";
import { Color, Point } from "./lib/types";

const drawer = new Drawer();
const http = new Http();

const IP_REQUEST_ID: u32 = 1;
let requested: bool = false;
let ip: string = "Loading...";

export function render(): void {
  if (!requested) {
    http.fetch(IP_REQUEST_ID, "GET", "1.1.1.1", "", "").execute();
    requested = true;
  }

  let ev = pollEvent();
  while (ev !== null) {
    if (ev.type == EVENT_HTTP_RESPONSE) {
      const res = ev as HttpResponseEvent;
      if (res.requestId == IP_REQUEST_ID) {
        ip = res.body;
      }
    }
    ev = pollEvent();
  }

  drawer.clear().execute();
  drawer.text("Public IP:", new Point(10, 20)).color(Color.WHITE).execute();
  drawer.text(ip, new Point(10, 35)).color(new Color(0, 255, 0)).execute();
}
