<template>
  <!-- Chart Controls moved above plot area -->
  <div class="mb-2 flex flex-wrap items-center justify-between gap-4">
    <div class="flex-1 min-w-[300px]">
      <label for="windowSize">
        Display Window Size:
        <span class="font-bold  w-12 inline-block">{{ windowSize }}</span>
      </label>
      <div class="flex items-center gap-2 mt-1">
        <span class="text-xs w-12 text-center">500</span>
        <input id="windowSize" type="range" v-model.number="windowSize" min="500" max="20000" step="500"
          @change="initPlot" class="w-full h-1 bg-gray-200 rounded-lg accent-blue-500" />
        <span class="text-xs w-12 text-center">20000</span>
      </div>
    </div>
  </div>
  <div class="border rounded-md shadow-sm bg-gray-800 p-4 mb-4">
    <div class="grid" :style="{ gridTemplateColumns: '40px 1fr', gridTemplateRows: plotHeight + 'px 40px' }">
      <canvas ref="yAxisCanvas" class="border-r border-gray-200" :style="{ width: '40px', height: plotHeight + 'px' }"></canvas>
      <canvas ref="plotCanvas" class="w-full" :style="{ height: plotHeight + 'px' }"></canvas>
      <div class="w-full"></div>
      <canvas ref="xAxisCanvas" class="border-t border-gray-200 w-full" style="height:40px"></canvas>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, onActivated, onDeactivated, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { WebglPlot, WebglLine, ColorRGBA } from "webgl-plot";
import { chartDataBuffer } from '../../store/appState';
import { channelColors, channelVisibility } from './channelSettings';

const emit = defineEmits<{
  (event: 'crosshair-move', payload: { x: number; y: number; dataValues: number[] }): void;
}>();

const props = defineProps<{ running: boolean }>();

let fetchIntervalId: number | null = null;
let resizeTimeout: number | null = null;

/* ====================================================
   1. State and Plot Settings
   ==================================================== */
const windowSize = ref<number>(1000);
const dataBuffer = chartDataBuffer;
const MAX_BUFFER_SIZE = 20000;
const yMin = ref<number>(-1); // Default min value with small negative offset
const yMax = ref<number>(1.05);  // Default max value with small positive offset
const dataScale = ref<number>(1);
const dataOffset = ref<number>(0);

const plotCanvas = ref<HTMLCanvasElement | null>(null);
const xAxisCanvas = ref<HTMLCanvasElement | null>(null);
const yAxisCanvas = ref<HTMLCanvasElement | null>(null);
let wglp: WebglPlot | null = null;
const lines: WebglLine[] = [];

let fpsCounter: number = 0;
let fpsControl: number = 1;
let animationFrame: number | null = null;
let dataUpdatePending: boolean = false;
// let lastUpdateTime: number = 0;
let crosshairLastUpdateTime: number = 0; // Timestamp of last crosshair update
const CROSSHAIR_THROTTLE_MS = 10; // Throttle interval in ms

let Rect: WebglLine | null = null;       // Zoom selection rectangle
let zoomFlag: boolean = false;  // Indicates left-click zoom gesture active
let cursorDownX: number = 0;   // X coordinate at mouse down (in current data coordinates)

let crossXLine: WebglLine | null = null; // Horizontal crosshair line
let crossYLine: WebglLine | null = null; // Vertical crosshair line
let crossX: number = 0;        // Current X position of crosshair
let crossY: number = 0;        // Current Y position of crosshair

/* --- Axis configuration --- */
const Y_AXIS_DIVISIONS = 8;  // Number of divisions for Y axis
const X_AXIS_DIVISIONS = 10; // Number of divisions for X axis

const plotHeight = ref<number>(Math.max(300, window.innerHeight * 0.4));

/* ====================================================
   2. Utility Functions
   ==================================================== */
/** Convert a hex color string (e.g. "#FF6384") to a normalized ColorRGBA. */
function hexToRGBA(hex: string): ColorRGBA {
  hex = hex.replace("#", "");
  const bigint = parseInt(hex, 16);
  const r = (bigint >> 16) & 255;
  const g = (bigint >> 8) & 255;
  const b = bigint & 255;
  return new ColorRGBA(r / 255, g / 255, b / 255, 1);
}

/**
 * Initialize axis canvas with proper scaling and return the 2D context
 */
function initAxisCanvas(canvas: HTMLCanvasElement | null): CanvasRenderingContext2D | null {
  if (!canvas) return null;
  const devicePixelRatio = window.devicePixelRatio || 1;
  canvas.width = canvas.clientWidth * devicePixelRatio;
  canvas.height = canvas.clientHeight * devicePixelRatio;

  const ctx2d = canvas.getContext("2d");
  if (ctx2d) {
    ctx2d.font = "12px Arial";
    ctx2d.textBaseline = "middle";
    ctx2d.textAlign = "left";
    ctx2d.fillStyle = "#cbd5e0"; // Dark gray text
    ctx2d.strokeStyle = "#cbd5e0"; // Light gray lines
    ctx2d.lineWidth = 2;
  }
  return ctx2d;
}

/**
 * Update Y-axis labels based on current scale and offset
 */
function updateYAxis(): void {
  const canvas = yAxisCanvas.value;
  if (!canvas) return;

  const ctx2d = initAxisCanvas(canvas);
  if (!ctx2d) return;

  const width = canvas.width;
  const height = canvas.height;
  const divisions = Y_AXIS_DIVISIONS;

  ctx2d.clearRect(0, 0, width, height);

  for (let i = 0; i <= divisions; i++) {
    const value = yMax.value - (i / divisions) * (yMax.value - yMin.value);
    const y = (i / divisions) * (height - 1);

    ctx2d.beginPath();
    ctx2d.moveTo(width - 5, y);
    ctx2d.lineTo(width, y);
    ctx2d.stroke();

    const formattedValue = value.toFixed(1);
    ctx2d.fillText(formattedValue, 4, y);
  }
}

/**
 * Update X-axis labels based on current scale and offset
 */
function updateXAxis(): void {
  const canvas = xAxisCanvas.value;
  if (!canvas) return;

  const ctx2d = initAxisCanvas(canvas);
  if (!ctx2d) return;

  const width = canvas.width;
  const height = canvas.height;
  const divisions = X_AXIS_DIVISIONS;

  ctx2d.clearRect(0, 0, width, height);

  for (let i = 0; i <= divisions; i++) {
    const position = (i / divisions) * width;

    const screenX = (2 * position / width) - 1; // Convert to [-1, 1] range
    const dataX = (screenX - wglp!.gOffsetX || 0) / (wglp!.gScaleX || 1);

    const index = Math.floor((dataX + 1) / 2 * windowSize.value);

    ctx2d.beginPath();
    ctx2d.moveTo(position, 0);
    ctx2d.lineTo(position, 5);
    ctx2d.stroke();

    if (index >= 0 && index <= windowSize.value) {
      ctx2d.fillText(index.toString(), position - 10, 20);
    }
  }
}

/* ====================================================
   3. Plot Initialization & Data Handling
   ==================================================== */
/**
 * Initialize the WebGL‑Plot instance, set up plot lines,
 * add the zoom selection rectangle, and register event listeners.
 */
function initPlot(): void {
  const canvas = plotCanvas.value;
  if (!canvas) return;

  const devicePixelRatio = window.devicePixelRatio || 1;
  canvas.width = canvas.clientWidth * devicePixelRatio;
  canvas.height = canvas.clientHeight * devicePixelRatio;

  wglp = new WebglPlot(canvas);
  lines.length = 0;
  
  const numPoints = windowSize.value;
  for (let i = 0; i < channelColors.length; i++) {
    const color = hexToRGBA(channelColors[i]);
    const line = new WebglLine(color, numPoints);
    line.arrangeX();
    // apply saved visibility state
    line.visible = channelVisibility[i];
    
    wglp.addLine(line);
    lines.push(line);
  }

  if (dataBuffer.length > 0) {
    fillLinesWithExistingData();
  }

  Rect = new WebglLine(new ColorRGBA(0.486, 1, 0.769, 0.1), 4);
  Rect.loop = true;
  Rect.xy = new Float32Array([-0.5, yMin.value, -0.5, yMax.value, 0.5, yMax.value, 0.5, yMin.value]);
  Rect.visible = false;
  wglp.addLine(Rect);

  const crossColor = new ColorRGBA(0.9, 0.1, 0.1, 1); // Green color
  crossXLine = new WebglLine(crossColor, 2);
  crossYLine = new WebglLine(crossColor, 2);

  crossX = 0;
  crossY = (yMin.value + yMax.value) / 2;
  crossXLine.xy = new Float32Array([crossX, yMin.value, crossX, yMax.value]); // Vertical line spans full y-range
  crossYLine.xy = new Float32Array([-1, crossY, 1, crossY]); // Horizontal line

  wglp.addLine(crossXLine);
  wglp.addLine(crossYLine);

  const canvasEl = canvas;
  canvasEl.addEventListener("touchstart", touchStart);
  canvasEl.addEventListener("touchmove", touchMove);
  canvasEl.addEventListener("touchend", touchEnd);
  canvasEl.addEventListener("mousedown", mouseDown);
  canvasEl.addEventListener("mousemove", mouseMove);
  canvasEl.addEventListener("mouseup", mouseUp);
  canvasEl.addEventListener("dblclick", dblClick);
  canvasEl.addEventListener("contextmenu", contextMenu);

  if (!animationFrame) {
    animationFrame = requestAnimationFrame(animate);
  }

  updateYAxis();
  updateXAxis();
}

/**
 * Clear the data buffer and reset the plot visualization.
 */
function clearPlot(): void {
  dataBuffer.length = 0;

  initPlot();

  yMin.value = -1;
  yMax.value = 1;
  updatePlotScale();

  console.log("Plot cleared");
}

/**
 * Populate plot lines with buffered data.
 */
function fillLinesWithExistingData(): void {
  if (!lines.length || !dataBuffer.length) return;
  const displayData = dataBuffer.slice(-windowSize.value);
  updateMinMax();
  const offsetPoints = Math.max(0, windowSize.value - displayData.length);
  for (let ch = 0; ch < 8 && ch < lines.length; ch++) {
    if (offsetPoints > 0) {
      const firstPoint = displayData.length > 0 ? displayData[0][ch] : 0;
      for (let i = 0; i < offsetPoints; i++) {
        lines[ch].setY(i, firstPoint);
      }
    }
    for (let i = 0; i < displayData.length; i++) {
      lines[ch].setY(offsetPoints + i, displayData[i][ch]);
    }
  }
}

/**
 * Refresh data from the backend and update the plot.
 */
function refreshData(): void {
  // Bail if component inactive or not initialized
  if (!isActive || !wglp) return;
  if (!props.running) return;
  if (dataUpdatePending) {
    console.log("Skipping data fetch – another fetch is in progress");
    return;
  }
  dataUpdatePending = true;
  // const startTime = Date.now();
  invoke<number[][]>("get_recent_data")
    .then((newData) => {
      // const fetchTime = Date.now() - startTime;
      if (newData && newData.length > 0) {
        addToDataBuffer(newData);
        addNewDataPoints(newData);
      }
    })
    .catch((error) => {
      console.error("Error retrieving data:", error);
    })
    .finally(() => {
      dataUpdatePending = false;
    });
}

/**
 * Append new data to the persistent data buffer.
 */
function addToDataBuffer(newData: number[][]): void {
  if (!newData || newData.length === 0) return;
  dataBuffer.push(...newData);
  if (dataBuffer.length > MAX_BUFFER_SIZE) {
    // Trim oldest entries to keep buffer size
    dataBuffer.splice(0, dataBuffer.length - MAX_BUFFER_SIZE);
  }
}

/**
 * Add new data points to the plot (via shiftAdd) and update vertical scaling.
 */
function addNewDataPoints(newData: number[][]): void {
  if (!newData || newData.length === 0 || !lines.length) return;
  const rawData = [];
  for (let ch = 0; ch < 8; ch++) {
    rawData[ch] = new Float32Array(newData.length);
    for (let i = 0; i < newData.length; i++) {
      rawData[ch][i] = newData[i][ch];
    }
  }
  for (let ch = 0; ch < 8 && ch < lines.length; ch++) {
    lines[ch].shiftAdd(rawData[ch]);
  }
  updateMinMax();
  updatePlotScale();
  // updateCrosshair(crossX, crossY);
}

/**
 * Update the vertical scaling of the plot.
 */
function updatePlotScale(): void {
  if (!wglp) return;
  const range = Math.max(0.001, yMax.value - yMin.value);
  dataScale.value = 2 / range;
  dataOffset.value = -(yMin.value + yMax.value) / 2;
  // Apply global world-to-clip transform
  wglp.gScaleY = dataScale.value;
  wglp.gOffsetY = dataOffset.value * dataScale.value;
  updateYAxis();

  if (crossXLine && crossYLine) {
    updateCrosshair(crossX, crossY);
  }
}

/**
 * Compute yMin and yMax from the visible data.
 */
function updateMinMax(): void {
  if (!dataBuffer.length) return;
  const visibleData = dataBuffer.slice(-windowSize.value);
  let minVal = Infinity;
  let maxVal = -Infinity;
  for (let i = 0; i < visibleData.length; i++) {
    const point = visibleData[i];
    for (let ch = 0; ch < 8; ch++) {
      minVal = Math.min(minVal, point[ch]);
      maxVal = Math.max(maxVal, point[ch]);
    }
  }
  const range = Math.max(0.001, maxVal - minVal);
  const margin = range * 0.05;
  yMin.value = minVal - margin;
  yMax.value = maxVal + margin;
  updatePlotScale();
}

/**
 * Animation loop to continuously update the plot.
 */
function animate(): void {
  if (!isActive) return; // stop if component inactive
  if (fpsCounter === 0 && wglp) {
    if (props.running) {
      refreshData();
    }
    wglp.update();
    if (Math.random() < 0.1) {
      updateXAxis();
    }
  }
  fpsCounter = (fpsCounter + 1) % fpsControl;
  animationFrame = requestAnimationFrame(animate);
}

/**
 * Throttle data refresh to prevent rapid firing.
 */
// function scheduleUpdate(): void {
//   if (!props.running) return;
//   if (updateScheduled) return;
//   const now = Date.now();
//   const minTimeBetween = 30;
//   if (now - lastUpdateTime < minTimeBetween) {
//     const waitTime = minTimeBetween - (now - lastUpdateTime);
//     updateScheduled = true;
//     setTimeout(() => {
//       refreshData();
//       updateScheduled = false;
//       lastUpdateTime = Date.now();
//     }, waitTime);
//   } else {
//     updateScheduled = true;
//     setTimeout(() => {
//       refreshData();
//       updateScheduled = false;
//       lastUpdateTime = Date.now();
//     }, 0);
//   }
// }

/**
 * Get the data values for all channels at a specific x position
 */
function getDataValuesAtPosition(x: number): number[] {
  if (!dataBuffer.length || !lines.length) {
    return Array(8).fill(0);
  }

  const visibleData = dataBuffer.slice(-windowSize.value);
  if (visibleData.length === 0) {
    return Array(8).fill(0);
  }
  if (!lines.length || !wglp) {
    return Array(8).fill(0);
  }

  const line = lines[0];
  let canvasIndex = Math.round(((x + 1) / 2) * (line.numPoints - 1));

  const offset = Math.max(0, windowSize.value - visibleData.length);

  if (canvasIndex < 0 || canvasIndex >= line.numPoints) {
    return Array(8).fill(0); // Return zeros if out of range
  }

  let dataIndex;
  if (canvasIndex < offset) {
    return Array(8).fill(0);
  } else {
    dataIndex = canvasIndex - offset;
  }

  if (dataIndex < 0 || dataIndex >= visibleData.length) {
    return Array(8).fill(0);
  }

  const result = [];
  for (let ch = 0; ch < 8; ch++) {
    result.push(visibleData[dataIndex][ch]);
  }
  return result;
}

/**
 * Update the position of the crosshair lines
 */
function updateCrosshair(x: number, y: number): void {
  if (!crossXLine || !crossYLine) return;

  crossX = x;
  crossY = y;

  // Vertical line spans data y-range at world x
  crossXLine.xy = new Float32Array([x, yMin.value, x, yMax.value]);
  crossYLine.xy = new Float32Array([-1, y, 1, y]);

  const dataValues = getDataValuesAtPosition(x);
  // console.log(yMin.value, yMax.value, y)
  emit('crosshair-move', {
    x: x,
    y: y,
    dataValues: dataValues
  });
}

/**
 * Set color of a specific channel line without reinitializing the plot.
 */
function setChannelColor(index: number, colorHex: string): void {
  if (index < 0 || index >= lines.length || !lines[index]) return;
  lines[index].color = hexToRGBA(colorHex);
}

/**
 * Toggle visibility of a specific channel line.
 */
function setChannelVisibility(index: number, visible: boolean): void {
  if (index < 0 || index >= lines.length || !lines[index]) return;
  lines[index].visible = visible;
}

/* ====================================================
   4. Interactive Event Handlers (Left‑Click Zoom & Touch)
   ==================================================== */

/**
 * Convert mouse or touch event to data coordinates
 */
function getPointerDataCoords(e: MouseEvent | TouchEvent): { dataX: number; dataY?: number } {
  const canvas = plotCanvas.value!;
  const dpr = window.devicePixelRatio || 1;
  const rect = canvas.getBoundingClientRect();
  const clientX = 'touches' in e ? e.touches[0].clientX : (e as MouseEvent).clientX;
  const clientY = 'touches' in e ? e.touches[0].clientY : (e as MouseEvent).clientY;
  const relX = (clientX - rect.left) * dpr;
  const rawX = 2 * (relX / canvas.width) - 1;
  const dataX = (rawX - wglp!.gOffsetX || 0) / (wglp!.gScaleX || 1);
  let dataY;
  if (!('touches' in e)) {
    const relY = (clientY - rect.top) * dpr;
    const rawY = 1 - 2 * (relY / canvas.height);
    dataY = (rawY - wglp!.gOffsetY) / (wglp!.gScaleY || 1);
  }
  return { dataX, dataY };
}

/**
 * On double‑click, reset the x‑axis zoom and remove the selection rectangle.
 */
function dblClick(e: MouseEvent): void {
  e.preventDefault();
  if (wglp) { wglp.gScaleX = 1; wglp.gOffsetX = 0; }
  if (Rect) Rect.visible = false;
}

/**
 * Prevent the context menu from appearing.
 */
function contextMenu(e: MouseEvent): void {
  e.preventDefault();
}

/**
 * Mouse down (only left button) initiates a zoom selection.
 * The raw mouse coordinate is converted into the current data coordinate.
 */
function mouseDown(e: MouseEvent): void {
  e.preventDefault();
  if (e.button === 0) {
    zoomFlag = true;
    const { dataX } = getPointerDataCoords(e);
    cursorDownX = dataX;
    if (Rect) Rect.visible = true;
  }
}

/**
 * Touch start initiates a zoom selection.
 * The raw touch coordinate is converted into the current data coordinate.
 */
function touchStart(e: TouchEvent): void {
  e.preventDefault();
  if (e.touches.length === 1) {
    zoomFlag = true;
    const { dataX } = getPointerDataCoords(e);
    cursorDownX = dataX;
    if (Rect) Rect.visible = true;
  }
}

/**
 * Mouse move updates the zoom selection rectangle.
 * The x coordinate is converted to the current data coordinate.
 * The rectangle's y coordinates are set to the current yMin and yMax.
 */
function mouseMove(e: MouseEvent): void {
  e.preventDefault();
  const { dataX, dataY } = getPointerDataCoords(e);
  const now = Date.now();
  if (dataY !== undefined && now - crosshairLastUpdateTime >= CROSSHAIR_THROTTLE_MS) {
    updateCrosshair(dataX, dataY);
    crosshairLastUpdateTime = now;
  }
  if (zoomFlag && Rect) {
    Rect.xy = new Float32Array([
      cursorDownX, yMin.value,
      cursorDownX, yMax.value,
      dataX, yMax.value,
      dataX, yMin.value,
    ]);
    Rect.visible = true;
  }
}

/**
 * Touch move updates the zoom selection rectangle.
 * The x coordinate is converted to the current data coordinate.
 * The rectangle's y coordinates are set to the current yMin and yMax.
 */
function touchMove(e: TouchEvent): void {
  e.preventDefault();
  if (zoomFlag && Rect) {
    const { dataX } = getPointerDataCoords(e);
    Rect.xy = new Float32Array([
      cursorDownX, yMin.value,
      cursorDownX, yMax.value,
      dataX, yMax.value,
      dataX, yMin.value,
    ]);
    Rect.visible = true;
    // (Rect as WebglLine).color = new ColorRGBA(255, 0, 0, 0.5);
  }
}

/**
 * Mouse up completes the zoom selection.
 * The final x coordinate is converted to data coordinate, and then
 * the new x‑scale and x‑offset are calculated so that the selected
 * region spans the full x‑axis.
 */
function mouseUp(e: MouseEvent): void {
  e.preventDefault();
  if (zoomFlag) {
    const { dataX: cursorUpX } = getPointerDataCoords(e);
    const selectionWidth = Math.abs(cursorUpX - cursorDownX);
    if (selectionWidth > 0) {
      const newScale = 2 / selectionWidth;
      const mid = (cursorDownX + cursorUpX) / 2;
      wglp!.gScaleX = newScale;
      wglp!.gOffsetX = -mid * newScale;
      updateXAxis();
    }
    zoomFlag = false;
    if (Rect) Rect.visible = false;
  }
}

/**
 * Touch end completes the zoom selection.
 * The final x coordinate is converted to data coordinate, and then
 * the new x‑scale and x‑offset are calculated so that the selected
 * region spans the full x‑axis.
 */
function touchEnd(e: TouchEvent): void {
  e.preventDefault();
  if (zoomFlag) {
    zoomFlag = false;
    if (Rect) Rect.visible = false;
  }
}

/* ====================================================
   5. Lifecycle and Resize Handling
   ==================================================== */
function updatePlotHeight(): void {
  plotHeight.value = Math.max(300, window.innerHeight * 0.4);
}

function handleResize(): void {
  updatePlotHeight();
  if (!plotCanvas.value) return;
  if (resizeTimeout) clearTimeout(resizeTimeout);
  resizeTimeout = setTimeout(() => {
    initPlot();
    updateYAxis();
    updateXAxis();
  }, 50);
}

watch(() => props.running, (newVal) => {
  if (newVal) {
    // scheduleUpdate();
    fetchIntervalId = setInterval(() => refreshData(), 100);
  } else if (fetchIntervalId) {
    clearInterval(fetchIntervalId);
    fetchIntervalId = null;
  }
}, { immediate: true });

onMounted(() => {
  initPlot();
  // lastUpdateTime = Date.now();
  window.addEventListener("resize", handleResize);
});

watch(windowSize, () => {
  const currentBuffer = [...dataBuffer];
  initPlot();
  dataBuffer.length = 0;
  dataBuffer.push(...currentBuffer);
  updateMinMax();
});

let isActive = true;
onBeforeUnmount(() => {
  isActive = false;
  if (animationFrame) {
    cancelAnimationFrame(animationFrame);
    animationFrame = null;
  }
  window.removeEventListener("resize", handleResize);
  if (fetchIntervalId) {
    clearInterval(fetchIntervalId);
    fetchIntervalId = null;
  }
});

onActivated(() => {
  initPlot();
  // lastUpdateTime = Date.now();
  window.addEventListener("resize", handleResize);
  if (props.running) {
    fetchIntervalId = setInterval(() => refreshData(), 100);
  }
});

onDeactivated(() => {
  isActive = false;
  if (fetchIntervalId) {
    clearInterval(fetchIntervalId);
    fetchIntervalId = null;
  }
  if (animationFrame) {
    cancelAnimationFrame(animationFrame);
    animationFrame = null;
  }
  window.removeEventListener("resize", handleResize);
});

// Expose initPlot and setChannelColor for parent ref
defineExpose({ initPlot, setChannelColor, setChannelVisibility, clearPlot });
</script>
