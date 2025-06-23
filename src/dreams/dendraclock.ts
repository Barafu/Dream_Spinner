let performance_text: string = "Not set";
const SHOW_PERFORMANCE = false;

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
    this.calculateEndPoint();
  }

  calculateEndPoint() {
    let angle = this.angle;

    // Calculate the change in x and y
    const deltaX = this.length * Math.sin(angle);
    const deltaY = this.length * Math.cos(angle);

    // Calculate end point
    this.endX = this.startX + deltaX;
    this.endY = this.startY - deltaY;
  }
}

class DendraClockPersistentOptions {
  ZOOM = 0.25;
  START_LINE_WIDTH = 8;
  DEPTH = 8;
  LENGTH_FACTOR = 0.8;
  LUMINANCE_FACTOR = 0.8;
  WIDTH_FACTOR = 0.7;
  START_ARM_LENGTH = 250;
}

class ClockTask {
  x: number;
  y: number;
  rotation: number;
  constructor(x: number, y: number, rotation: number) {
    this.x = x;
    this.y = y;
    this.rotation = rotation;
  }
}

class ClockTaskResult {
  hands: Hand[];
  tasks: ClockTask[];
  constructor() {
    this.hands = [];
    this.tasks = [];
  }
}

class ClockTaskAngles {
  hour_angle: number;
  minute_angle: number;
  second_angle: number;

  constructor(hour_angle: number, minute_angle: number, second_angle: number) {
    this.hour_angle = hour_angle;
    this.minute_angle = minute_angle;
    this.second_angle = second_angle;
  }
}

function processClockTask(clock_task: ClockTask, angles: ClockTaskAngles, depth: number, settings: DendraClockPersistentOptions) {

  let ccma = angles.minute_angle + clock_task.rotation;
  let ccsa = angles.second_angle + clock_task.rotation;
  let hand_length = settings.START_ARM_LENGTH * Math.pow(settings.LENGTH_FACTOR, depth);
  let m_hand = new Hand(clock_task.x, clock_task.y, hand_length, ccma);
  let s_hand = new Hand(clock_task.x, clock_task.y, hand_length, ccsa);
  
  
  let result = new ClockTaskResult();
  
  
  result.hands.push(m_hand);
  result.hands.push(s_hand);
  
  if (depth == 0) {
    let ccha = angles.hour_angle + clock_task.rotation;
    let h_hand = new Hand(clock_task.x, clock_task.y, hand_length * 0.7, ccha);
    result.hands.push(h_hand);
  }

  if (depth < settings.DEPTH) {
    let mt = new ClockTask(m_hand.endX, m_hand.endY,  m_hand.angle - angles.hour_angle + Math.PI );
    let st = new ClockTask(s_hand.endX, s_hand.endY,  s_hand.angle - angles.hour_angle + Math.PI );
    result.tasks.push(mt);
    result.tasks.push(st);
  }

  return result;
}

export function dendraClock(canvas: HTMLCanvasElement) {
  //var startTime = performance.now();
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

  // Calculate angles of current time
  const now = new Date();
  const seconds = now.getSeconds() + now.getMilliseconds() / 1000.0;
  const minutes = now.getMinutes() + seconds / 60.0;
  const hours = now.getHours() + minutes / 60.0;
  const hour_angle = (hours * Math.PI) / 6 ;
  const minute_angle = (minutes * Math.PI) / 30 ;
  const second_angle = (seconds * Math.PI) / 30 ;
  let clock_tasks: ClockTask[] = [];
  let next_tasks: ClockTask[] = [];

  clock_tasks.push(new ClockTask(canvas.width / 2, canvas.height / 2, 0));

  let depth = 0;
  let hands: Hand[] = [];

  do {
    for (let current_task of clock_tasks) {
      let current_angles = new ClockTaskAngles(hour_angle, minute_angle, second_angle);
      let result = processClockTask(current_task, current_angles, depth, settings);
        next_tasks.push(...result.tasks);
        hands.push(...result.hands);
    }
    clock_tasks = next_tasks;
    next_tasks = [];
    hands_map.set(depth, hands);
    hands = [];
    depth++;
  } while (clock_tasks.length > 0);

  console.assert(hands_map.get(0)?.length == 3);
  //const calc_timestamp = performance.now();


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
  if (SHOW_PERFORMANCE) {
    ctx.fillStyle = "yellow";
    ctx.font = "20px serif";
    ctx.fillText(performance_text, 10, 20);
  }
  ctx.fillStyle = "black";
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  //const draw_timestamp = performance.now();

  //performanses(startTime, calc_timestamp, draw_timestamp);
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
// @ts-expect-error - This function will be used later
function performances(start: DOMHighResTimeStamp, calc: DOMHighResTimeStamp, draw: DOMHighResTimeStamp) {
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
   */
  function process_stage(stage: number[]) {
    let sum = stage.reduce((partialSum, a) => partialSum + a, 0);
    let avg = sum / stage.length;
    return avg;
  }
  const calc_perf = process_stage(stage_calc);
  const draw_perf = process_stage(stage_draw);

  performance_text = `Calculation: ${calc_perf.toFixed(3)} msec, Drawing: ${draw_perf.toFixed(3)} msec`;

  stage_calc.length = 0;
  stage_draw.length = 0;
}
