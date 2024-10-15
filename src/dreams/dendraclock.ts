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
  settings: DendraClockPersistentOptions;
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

    const arm_length = settings.START_ARM_LENGTH * Math.pow(settings.LENGTH_FACTOR, current_depth);

    this.hourHand = new Hand(centerX, centerY, arm_length * 0.7, hours * Math.PI / 6);
    this.minuteHand = new Hand(centerX, centerY, arm_length, minutes * Math.PI / 30);
    this.secondHand = new Hand(centerX, centerY, arm_length, seconds * Math.PI / 30);
  }

  draw(ctx: CanvasRenderingContext2D) {
    const arm_width = this.settings.START_LINE_WIDTH * Math.pow(this.settings.WIDTH_FACTOR, this.current_depth);
    const transparency_factor = Math.pow(this.settings.LUMINANCE_FACTOR, this.current_depth-1);
    const color = `rgba(255, 255, 255, ${transparency_factor})`;
    ctx.lineWidth = arm_width;
    ctx.lineCap = "round";
    ctx.strokeStyle = color;

    if (this.current_depth == 1) {
      const hourEndPoint = this.hourHand.calculateEndPoint();
      ctx.beginPath();
      ctx.moveTo(this.centerX, this.centerY);
      ctx.lineTo(hourEndPoint.x, hourEndPoint.y);
      ctx.stroke();
    }

    const minuteEndPoint = this.minuteHand.calculateEndPoint();
    ctx.beginPath();
    ctx.moveTo(this.centerX, this.centerY);
    ctx.lineTo(minuteEndPoint.x, minuteEndPoint.y);
    ctx.stroke();

    const secondEndPoint = this.secondHand.calculateEndPoint();
    ctx.beginPath();
    ctx.moveTo(this.centerX, this.centerY);
    ctx.lineTo(secondEndPoint.x, secondEndPoint.y);
    ctx.stroke();
  }

  rotateClockwise(angleInRadians: number) {
    this.hourHand.rotateClockwise(angleInRadians);
    this.minuteHand.rotateClockwise(angleInRadians);
    this.secondHand.rotateClockwise(angleInRadians);
  }

  rotateToHour(hour_angle: number) {
    const current_hour_angle = this.hourHand.angle + Math.PI;
    const rotation = hour_angle - current_hour_angle;
    this.rotateClockwise(rotation);
  }
}

class DendraClockPersistentOptions {
  ZOOM = 0.25;
  START_LINE_WIDTH = 10;
  DEPTH = 9;
  LENGTH_FACTOR = 0.9;
  LUMINANCE_FACTOR = 0.9;
  WIDTH_FACTOR = 0.7;
  START_ARM_LENGTH = 150;
}

export function dendraClock(canvas: HTMLCanvasElement) {
  const settings = new DendraClockPersistentOptions();
  const ctx = canvas.getContext("2d")!;
  ctx.globalCompositeOperation = "destination-over";
  const now = new Date();
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  dendra_clock_recursive(settings, now, ctx, 0, canvas.width / 2, canvas.height / 2, 0.0);
}

function dendra_clock_recursive(settings: DendraClockPersistentOptions, now: Date, ctx: CanvasRenderingContext2D, current_depth: number, x: number, y: number, extra_rotation: number) {
  if (current_depth == settings.DEPTH) return;
  current_depth++;
  const clock = new AnalogClock(now, x, y, current_depth, settings);
  if (current_depth != 1) {
    clock.rotateToHour(extra_rotation);
  }
  clock.draw(ctx);
  //const hour_pos = clock.hourHand.calculateEndPoint();
  const minute_pos = clock.minuteHand.calculateEndPoint();
  const seconds_pos = clock.secondHand.calculateEndPoint();
  const minutes_rotation = clock.minuteHand.angle;
  const seconds_rotation = clock.secondHand.angle;
  dendra_clock_recursive(settings, now, ctx, current_depth, minute_pos.x, minute_pos.y, minutes_rotation);
  dendra_clock_recursive(settings, now, ctx, current_depth, seconds_pos.x, seconds_pos.y, seconds_rotation);
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
