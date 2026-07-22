// Carousel widget — standalone wrapper around the CarouselApp.
// Rotates the apps listed in the "slides" config in a single placement.

import { Runner } from "./lib/widget";
import { CarouselApp } from "./lib/apps/carousel";

const runner = new Runner(new CarouselApp());

export function render(): void {
  runner.render();
}
