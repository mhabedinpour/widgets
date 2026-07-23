// Spotify widget — standalone wrapper around SpotifyApp.
// Placement: 64×23 band (same height as weather).

import { Runner } from "./lib/widget";
import { SpotifyApp } from "./lib/apps/spotify";

const runner = new Runner(new SpotifyApp());

export function render(): void {
  runner.render();
}
