import { exit } from '@tauri-apps/plugin-process';
//import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { getCurrentWindow } from "@tauri-apps/api/window";

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

  const appWindow = getCurrentWindow();
  appWindow.show();
});


