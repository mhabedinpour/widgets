import { Color, Drawer } from "./drawer";

const drawer = new Drawer();

export function render(): void {
  drawer.rect(10, 10, 50, 50).color(new Color(255, 0, 0)).draw();
  drawer.circle(80, 80, 20).color(new Color(0, 255, 0)).draw();
  drawer.line(0, 0, 100, 100).color(new Color(0, 0, 255)).thickness(2).draw();
  drawer.text("Hello WASM!", 10, 100).color(Color.WHITE).draw();
}
