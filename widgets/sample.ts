import { Color, Drawer } from "./drawer";

const drawer = new Drawer();

export function render(): void {
  drawer.rect(0, 0, 10, 10).color(new Color(255, 0, 0)).draw();
  drawer.circle(20, 20, 6).color(new Color(0, 255, 0)).draw();
  drawer.line(30, 30, 40, 40).color(new Color(0, 0, 255)).thickness(2).draw();
  drawer.text("Hello WASM!", 20, 25).color(Color.WHITE).draw();

  drawer.line(50, 50, 60, 60).color(new Color(255, 255, 0)).thickness(2).draw();
  drawer.triangle(0,0, 0, 15, 15, 0).color(new Color(255, 0, 0)).draw();

  __collect();
}
