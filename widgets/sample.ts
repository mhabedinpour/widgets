import { Drawer } from "./drawer";
import { Color, Point, Rect } from "./types";

const drawer = new Drawer();

export function render(): void {
  drawer.rect(new Rect(0, 0, 10, 10)).color(new Color(255, 0, 0)).execute();
  drawer.circle(new Point(20, 20), 6).color(new Color(0, 255, 0)).execute();
  drawer.line(new Point(30, 30), new Point(40, 40)).color(new Color(0, 0, 255)).thickness(2).execute();
  drawer.text("Hello WASM!", new Point(20, 25)).color(Color.WHITE).execute();

  drawer.line(new Point(50, 50), new Point(60, 60)).color(new Color(255, 255, 0)).thickness(2).execute();
  drawer.triangle(new Point(0, 0), new Point(0, 15), new Point(15, 0)).color(new Color(255, 0, 0)).execute();
}
