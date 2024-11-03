
import { clock } from "./dreams/clocks";
import { dendraClock } from "./dreams/dendraclock";

declare var pywebview: any;

let pywebviewready = false;
let DOMReady = false;
let canvas: HTMLCanvasElement | null = null;
let dream = "dendraclock";

window.addEventListener('pywebviewready', function () {
  pywebviewready = true;
  big_init();
})

window.addEventListener("DOMContentLoaded", function () {
  DOMReady = true;
  big_init();
});

function big_init() {
  if (!pywebviewready || !DOMReady) {
    return;
  }
  window.addEventListener("mouseup", (event) => {
    if (event.button == 0) {
      pywebview.api.quit();
    }
  });
  window.addEventListener("resize", resizeCanvas);
  canvas = document.getElementById("dreamCanvas") as HTMLCanvasElement;
  let dream: string = "dendraclock";
  resizeCanvas();
  animate();
  
}


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
