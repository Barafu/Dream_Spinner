import { exit } from '@tauri-apps/plugin-process';

let canvas: HTMLCanvasElement = document.getElementById("dreamCanvas") as HTMLCanvasElement;

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

window.addEventListener("DOMContentLoaded", () => {
  resizeCanvas();
  window.requestAnimationFrame(dendraClock);
});

/*
function clock() {
  const now = new Date();
  const ctx = canvas.getContext("2d");
  if (ctx === null) {
    console.error("Failed to get 2D drawing context");
    return;
  }
  ctx.save();
  const canvas_dim = Math.min(canvas.width, canvas.height);
  const canv_scale = canvas_dim / 330.0;
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.translate(canvas.width / 2, canvas.height / 2);
  ctx.scale(canv_scale, canv_scale);
  ctx.rotate(-Math.PI / 2);
  ctx.strokeStyle = "white";
  ctx.fillStyle = "white";
  ctx.lineWidth = 8;
  ctx.lineCap = "round";

  // Hour marks
  ctx.save();
  for (let i = 0; i < 12; i++) {
    ctx.beginPath();
    ctx.rotate(Math.PI / 6);
    ctx.moveTo(100, 0);
    ctx.lineTo(120, 0);
    ctx.stroke();
  }
  ctx.restore();

  // Minute marks
  ctx.save();
  ctx.lineWidth = 5;
  for (let i = 0; i < 60; i++) {
    if (i % 5 !== 0) {
      ctx.beginPath();
      ctx.moveTo(117, 0);
      ctx.lineTo(120, 0);
      ctx.stroke();
    }
    ctx.rotate(Math.PI / 30);
  }
  ctx.restore();

  const sec = now.getSeconds() + now.getMilliseconds() / 1000;
  const min = now.getMinutes();
  const hr = now.getHours() % 12;

  ctx.fillStyle = "white";

  // Write image description
  canvas.innerText = `The time is: ${hr}:${min}`;

  // Write Hours
  ctx.save();
  ctx.rotate(
    (Math.PI / 6) * hr + (Math.PI / 360) * min + (Math.PI / 21600) * sec,
  );
  ctx.lineWidth = 14;
  ctx.beginPath();
  ctx.moveTo(-20, 0);
  ctx.lineTo(80, 0);
  ctx.stroke();
  ctx.restore();

  // Write Minutes
  ctx.save();
  ctx.rotate((Math.PI / 30) * min + (Math.PI / 1800) * sec);
  ctx.lineWidth = 10;
  ctx.beginPath();
  ctx.moveTo(-28, 0);
  ctx.lineTo(112, 0);
  ctx.stroke();
  ctx.restore();

  // Write seconds
  ctx.save();
  ctx.rotate((sec * Math.PI) / 30);
  ctx.strokeStyle = "#D40000";
  ctx.fillStyle = "#D40000";
  ctx.lineWidth = 6;
  ctx.beginPath();
  ctx.moveTo(-30, 0);
  ctx.lineTo(83, 0);
  ctx.stroke();
  ctx.beginPath();
  ctx.arc(0, 0, 10, 0, Math.PI * 2, true);
  ctx.fill();
  ctx.beginPath();
  ctx.arc(95, 0, 10, 0, Math.PI * 2, true);
  ctx.stroke();
  ctx.fillStyle = "rgb(0 0 0 / 0%)";
  ctx.arc(0, 0, 3, 0, Math.PI * 2, true);
  ctx.fill();
  ctx.restore();

  ctx.beginPath();
  ctx.lineWidth = 14;
  ctx.strokeStyle = "#325FA2";
  ctx.arc(0, 0, 142, 0, Math.PI * 2, true);
  ctx.stroke();

  ctx.restore();

  window.requestAnimationFrame(clock);
}
*/

class Hand {
  startX: number;
  startY: number;
  length: number;
  angle: number;

  constructor(startX: number, startY: number, length: number, angle: number) {
    this.startX = startX;
    this.startY = startY;
    this.length = length;
    this.angle = angle; // in radians, 0.0 is straight up
  }

  calculateEndPoint() {
    let angle = this.angle - Math.PI / 2.0;

    // Calculate the change in x and y
    const deltaX = this.length * Math.cos(angle);
    const deltaY = this.length * Math.sin(angle);

    // Calculate end point
    const endX = this.startX + deltaX;
    const endY = this.startY + deltaY; // Subtract because y-axis is inverted in most computer graphics systems

    return { x: endX, y: endY };
  }

  rotateClockwise(angleInRadians: number) {
    this.angle = sum_rotations(this.angle, angleInRadians);
  }
}
class AnalogClock {
  time: Date;
  centerX: number;
  centerY: number;
  current_depth: number;
  settings: any;
  hourHand: Hand;
  minuteHand: Hand;
  secondHand: Hand;
  constructor(time: Date, centerX: number, centerY: number, current_depth: number, settings: any) {
    this.time = time;
    this.centerX = centerX;
    this.centerY = centerY;
    this.current_depth = current_depth;
    this.settings = settings;

    const hours = time.getHours() + time.getMinutes() / 60.0 + time.getSeconds() / 3600.0;
    const minutes = time.getMinutes() + time.getSeconds() / 60.0;
    const seconds = time.getSeconds() + time.getMilliseconds() / 1000.0;

    const arm_length = 100 * Math.pow(settings.LENGTH_FACTOR, current_depth);

    this.hourHand = new Hand(centerX, centerY, 50, hours * Math.PI / 6);
    this.minuteHand = new Hand(centerX, centerY, arm_length, minutes * Math.PI / 30);
    this.secondHand = new Hand(centerX, centerY, arm_length, seconds * Math.PI / 30);
  }

  draw(ctx: CanvasRenderingContext2D) {
    const arm_width = this.settings.START_LINE_WIDTH * Math.pow(this.settings.WIDTH_FACTOR, this.current_depth);
    ctx.lineWidth = arm_width;
    if (this.current_depth == 1) {
      const hourEndPoint = this.hourHand.calculateEndPoint();
      ctx.beginPath();
      ctx.moveTo(this.centerX, this.centerY);
      ctx.lineTo(hourEndPoint.x, hourEndPoint.y);
      ctx.strokeStyle = "green";
      ctx.stroke();
    }

    const minuteEndPoint = this.minuteHand.calculateEndPoint();
    ctx.beginPath();
    ctx.moveTo(this.centerX, this.centerY);
    ctx.lineTo(minuteEndPoint.x, minuteEndPoint.y);
    ctx.strokeStyle = "white";
    ctx.stroke();

    const secondEndPoint = this.secondHand.calculateEndPoint();
    ctx.beginPath();
    ctx.moveTo(this.centerX, this.centerY);
    ctx.lineTo(secondEndPoint.x, secondEndPoint.y);
    ctx.strokeStyle = "yellow";
    ctx.stroke();
  }

  rotateClockwise(angleInRadians: number) {
    this.hourHand.rotateClockwise(angleInRadians);
    this.minuteHand.rotateClockwise(angleInRadians);
    this.secondHand.rotateClockwise(angleInRadians);
  }

  rotateToHour(hour_angle: number) {
    const current_hour_angle = this.hourHand.angle;
    const rotation = hour_angle - current_hour_angle;
    this.rotateClockwise(rotation);
  }
}

class DendraClockPersistentOptions {
  ZOOM = 0.25;
  START_LINE_WIDTH = 8;
  DEPTH = 9;
  LENGTH_FACTOR = 0.95;
  LUMINANCE_FACTOR = 0.8;
  WIDTH_FACTOR = 0.7;
}

function dendraClock() {
  const settings = new DendraClockPersistentOptions();
  const ctx = canvas.getContext("2d")!;
  const now = new Date();
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  dendra_clock_recursive(settings, now, ctx,0, canvas.width / 2, canvas.height / 2, 0.0);
  window.requestAnimationFrame(dendraClock);
}

function dendra_clock_recursive(settings: DendraClockPersistentOptions, now: Date, ctx: CanvasRenderingContext2D, current_depth: number, x: number, y: number, extra_rotation: number) {
  current_depth++;
  if (current_depth == settings.DEPTH) return;
  const clock = new AnalogClock(now, x, y, current_depth, settings);
  clock.rotateToHour(extra_rotation);
  clock.draw(ctx);
  //const hour_pos = clock.hourHand.calculateEndPoint();
  const minute_pos = clock.minuteHand.calculateEndPoint();
  const seconds_pos = clock.secondHand.calculateEndPoint();
  const minutes_rotation = clock.minuteHand.angle;
  const seconds_rotation = clock.secondHand.angle;
  //dendra_clock_recursive(settings, current_depth, hour_pos.x, hour_pos.y);
  dendra_clock_recursive(settings, now, ctx, current_depth, minute_pos.x, minute_pos.y, minutes_rotation);
  dendra_clock_recursive(settings, now, ctx,current_depth, seconds_pos.x, seconds_pos.y, seconds_rotation);
}

function sum_rotations(rotation1: number, rotation2: number) {
  // Add the two rotation values
  let sum = rotation1 + rotation2;
  // Normalize the result to be within the range [0, 2π)
  sum = sum % (2 * Math.PI);
  // If the result is negative, add 2π to make it positive
  while (sum < 0) {
    sum += 2 * Math.PI;
  }
  return sum;
}
