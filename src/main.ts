
import { clock } from "./dreams/clocks";
import { dendraClock } from "./dreams/dendraclock";

declare var pywebview: any;

let canvas: HTMLCanvasElement | null = null;
//let dream = "clock";
let dream = "dendraclock";

window.addEventListener('pywebviewready', function () {
  window.addEventListener("mouseup", (event) => {
    if (event.button == 0) {
      pywebview.api.quit();
    }
  });
})

window.addEventListener("DOMContentLoaded", function () {
  window.addEventListener("resize", resizeCanvas);
  canvas = document.getElementById("dreamCanvas") as HTMLCanvasElement;
  let dream: string = "dendraclock";
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
