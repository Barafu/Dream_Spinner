let _performance_text: string = "Not set";

class Hand {
  startX: number;
  startY: number;
  length: number;
  angle: number;
  endX: number = -1000;
  endY: number = -1000;

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
    this.endX = this.startX + deltaX;
    this.endY = this.startY + deltaY; // Subtract because y-axis is inverted in most computer graphics systems
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
  constructor(
    time: Date,
    centerX: number,
    centerY: number,
    current_depth: number,
    settings: any,
  ) {
    this.time = time;
    this.centerX = centerX;
    this.centerY = centerY;
    this.current_depth = current_depth;
    this.settings = settings;

    const seconds = time.getSeconds() + time.getMilliseconds() / 1000.0;
    const minutes = time.getMinutes() + seconds / 60.0;
    const hours = time.getHours() + minutes / 60.0;

    const arm_length =
      settings.START_ARM_LENGTH *
      Math.pow(settings.LENGTH_FACTOR, current_depth);

    this.hourHand = new Hand(
      centerX,
      centerY,
      arm_length * 0.7,
      (hours * Math.PI) / 6,
    );
    this.minuteHand = new Hand(
      centerX,
      centerY,
      arm_length,
      (minutes * Math.PI) / 30,
    );
    this.secondHand = new Hand(
      centerX,
      centerY,
      arm_length,
      (seconds * Math.PI) / 30,
    );
  }

  calculateEndPoints() {
    this.hourHand.calculateEndPoint();
    this.minuteHand.calculateEndPoint();
    this.secondHand.calculateEndPoint();
  }

  rotateClockwise(angleInRadians: number) {
    this.hourHand.rotateClockwise(angleInRadians);
    this.minuteHand.rotateClockwise(angleInRadians);
    this.secondHand.rotateClockwise(angleInRadians);

    this.calculateEndPoints();
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
  DEPTH = 8;
  LENGTH_FACTOR = 0.9;
  LUMINANCE_FACTOR = 0.9;
  WIDTH_FACTOR = 0.7;
  START_ARM_LENGTH = 150;
}

class ClockTask {
  x: number;
  y: number;
  depth: number;
  rotation: number;
  constructor(x: number, y: number, depth: number, rotation: number) {
    this.x = x;
    this.y = y;
    this.depth = depth;
    this.rotation = rotation;
  }
}

export function dendraClock(canvas: HTMLCanvasElement) {
  var startTime = performance.now();
  const settings = new DendraClockPersistentOptions();

  // Prepare hands storage
  let hands_map: Map<number, Hand[]> = new Map();
  for (let i = 0; i <= settings.DEPTH; i++) {
    hands_map.set(i, []);
  }
  let ctx = canvas.getContext("2d")!;
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  // ===== Calculate stage =====
  // Calculate all arms positions and save them to hands_map
  const now = new Date();
  let clock_tasks: ClockTask[] = []; 

  clock_tasks.push(new ClockTask(canvas.width / 2, canvas.height / 2, 0, 0));

  while (clock_tasks.length > 0) {
    const clock_task = clock_tasks.pop()!;
    if (clock_task.depth > settings.DEPTH) continue;
    const clock = new AnalogClock(now, clock_task.x, clock_task.y, clock_task.depth, settings);

    // Store hands in hands array for current depth
    let hands_array = hands_map.get(clock_task.depth);
    if (hands_array == undefined) {
      console.assert(hands_array != undefined);
      hands_array = [];   // To please TS checker only
    }

    if (clock_task.depth != 0) {
      clock.rotateToHour(clock_task.rotation);
    }
    else {
      clock.calculateEndPoints();
      hands_array.push(clock.hourHand);
    }
    hands_array.push(clock.minuteHand);
    hands_array.push(clock.secondHand);

    let mt = new ClockTask(clock.minuteHand.endX, clock.minuteHand.endY, clock_task.depth + 1, clock.minuteHand.angle);
    let st = new ClockTask(clock.secondHand.endX, clock.secondHand.endY, clock_task.depth + 1, clock.secondHand.angle);
    clock_tasks.push(mt);
    clock_tasks.push(st);
  }


  console.assert(hands_map.get(0)?.length == 3);
  const calc_timestamp = performance.now();

  // ===== Draw stage =====
  ctx.globalCompositeOperation = "destination-over";
  ctx.lineCap = "round";

  // Draw hands, seeting style according to depth
  for (let [current_depth, hands_array] of hands_map) {
    const arm_width =
      settings.START_LINE_WIDTH *
      Math.pow(settings.WIDTH_FACTOR, current_depth);
    const transparency_factor = Math.pow(
      settings.LUMINANCE_FACTOR,
      current_depth - 1,
    );
    const color = `rgba(255, 255, 255, ${transparency_factor})`;
    ctx.lineWidth = arm_width;
    ctx.strokeStyle = color;

    ctx.beginPath();
    for (let hand of hands_array) {
      ctx.moveTo(hand.startX, hand.startY);
      ctx.lineTo(hand.endX, hand.endY);
    }
    ctx.stroke();
  }
  // ctx.fillStyle = "yellow";
  // ctx.font = "20px serif";
  // ctx.fillText(performance_text, 10, 20);
  ctx.fillStyle = "black";
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  const draw_timestamp = performance.now();

  performanses(startTime, calc_timestamp, draw_timestamp);
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

let stage_calc: number[] = [];
let stage_draw: number[] = [];

/**
 * Measures and logs the performance of calculation and drawing stages.
 * 
 * This function calculates the time taken for calculation and drawing stages
 * in a rendering process. It stores these times in separate arrays for
 * calculation and drawing. Once enough data points are collected, it computes
 * the average performance for each stage and logs it to the console.
 * 
 * @param {DOMHighResTimeStamp} start - The timestamp marking the start of the calculation stage.
 * @param {DOMHighResTimeStamp} calc - The timestamp marking the end of the calculation stage and start of the drawing stage.
 * @param {DOMHighResTimeStamp} draw - The timestamp marking the end of the drawing stage.
 */
function performanses(start: DOMHighResTimeStamp, calc: DOMHighResTimeStamp, draw: DOMHighResTimeStamp) {
  const calc_time: number = calc - start;
  const draw_time: number = draw - calc;

  stage_calc.push(calc_time);
  stage_draw.push(draw_time);

  if (stage_calc.length < 300) {
    return;
  }

  /**
   * Process a stage of timings, log the average to the console and
   * clear the array.
   * @param {number[]} stage - array of timings
   * @param {string} text - string to prefix the log message with
   */
  function process_stage(stage: number[]) {
    let sum = stage.reduce((partialSum, a) => partialSum + a, 0);
    let avg = sum / stage.length;
    return 1000 / avg;
  }
  const calc_perf = process_stage(stage_calc);
  const draw_perf = process_stage(stage_draw);

  _performance_text = `Calculation: ${calc_perf.toFixed(2)}hz, Drawing: ${draw_perf.toFixed(2)}hz`;

  stage_calc.length = 0;
  stage_draw.length = 0;
}
