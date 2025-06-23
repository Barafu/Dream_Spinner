
import { clock } from "./dreams/clocks";
import { dendraClock } from "./dreams/dendraclock";

//import { event, window, path } from '@tauri-apps/api'
import { exit } from '@tauri-apps/plugin-process';
import { getCurrentWindow } from "@tauri-apps/api/window";

let canvas: HTMLCanvasElement | null = null;
const dreams = ["clock", "dendraclock"];
let dream = dreams[Math.floor(Math.random() * dreams.length)];

window.addEventListener("mouseup", (event) => {
  if (event.button == 0) {
    exit(0)
  }
});

window.addEventListener("DOMContentLoaded", function () {
  getCurrentWindow().show();
  window.addEventListener("resize", resizeCanvas);
  canvas = document.getElementById("dreamCanvas") as HTMLCanvasElement;
  resizeCanvas();
  animate();
});

function resizeCanvas() {
  if (canvas) {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
  }
}

function animate() {
  if (!canvas) {
    return;
  }
  if (dream == "clock") {
    clock(canvas);
  }
  if (dream == "dendraclock") {
    dendraClock(canvas);
  }
  window.requestAnimationFrame(animate);
}
