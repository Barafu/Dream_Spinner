import { exit } from '@tauri-apps/plugin-process';

import { clock } from "./dreams/clocks";
import { dendraClock } from "./dreams/dendraclock";

let canvas: HTMLCanvasElement = document.getElementById("dreamCanvas") as HTMLCanvasElement;

let dream: string = "clock";

addEventListener("mouseup", (event) => {
  if (event.button == 0) {
    exit(0);
  }
});

window.addEventListener('resize', resizeCanvas);

function resizeCanvas() {
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;
}

function animate() {
  if (dream == "clock") {
    clock(canvas);
  }
  if (dream == "dendraclock") {
    dendraClock(canvas);
  }
  window.requestAnimationFrame(animate);
}

window.addEventListener("DOMContentLoaded", () => {
  resizeCanvas();
  animate();
});


