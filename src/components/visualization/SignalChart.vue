<template>
  <div class="border rounded-md shadow-sm bg-white p-4 h-[500px] mb-4">
    <!-- Chart Controls -->
    <div class="chart-controls mb-2">
      <div class="slider-container">
        <label for="windowSize">
          Display Window Size:
          <span class="window-size-value">{{ windowSize }}</span>
        </label>
        <div class="slider-with-labels">
          <span class="slider-min-label">100</span>
          <input 
            id="windowSize" 
            type="range" 
            v-model.number="windowSize" 
            min="100" 
            max="20000" 
            step="100"
            @change="initPlot" 
            class="window-size-slider" 
          />
          <span class="slider-max-label">20000</span>
        </div>
      </div>
      <button class="btn refresh-btn bg-blue-500 text-white rounded" @click="clearPlot">
        Clear Plot
      </button>
    </div>
    <!-- Y-Axis Canvas -->
    <div class="chart-visualization-container">
      <canvas ref="yAxisCanvas" class="y-axis-canvas"></canvas>
      <!-- WebGL‑Plot Canvas -->
      <canvas ref="plotCanvas" class="chart-container h-[420px]"></canvas>
    </div>
    <!-- X-Axis Canvas -->
    <canvas ref="xAxisCanvas" class="x-axis-canvas"></canvas>
  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { WebglPlot, WebglLine, ColorRGBA } from "webgl-plot";

/* ====================================================
   1. State and Plot Settings
   ==================================================== */
const windowSize = ref(1000);
const dataBuffer = ref([]);
const MAX_BUFFER_SIZE = 50000;
const yMin = ref(0);
const yMax = ref(1000);
const dataScale = ref(1);
const dataOffset = ref(0);

const plotCanvas = ref(null);
const xAxisCanvas = ref(null);
const yAxisCanvas = ref(null);
let wglp = null;
const lines = [];

let fpsCounter = 0;
let fpsControl = 1;
let animationFrame = null;
let dataUpdatePending = false;
let updateScheduled = false;
let lastUpdateTime = 0;

/* --- Interactive state variables --- */
let Rect = null;       // Zoom selection rectangle
let zoomFlag = false;  // Indicates left-click zoom gesture active
let cursorDownX = 0;   // X coordinate at mouse down (in current data coordinates)
let initialX = 0;      // For touch events

/* --- Axis configuration --- */
const Y_AXIS_DIVISIONS = 8;  // Number of divisions for Y axis
const X_AXIS_DIVISIONS = 10; // Number of divisions for X axis

/* ====================================================
   2. Utility Functions
   ==================================================== */
/** Convert a hex color string (e.g. "#FF6384") to a normalized ColorRGBA. */
function hexToRGBA(hex) {
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
function initAxisCanvas(canvas) {
  if (!canvas) return null;
  const devicePixelRatio = window.devicePixelRatio || 1;
  canvas.width = canvas.clientWidth * devicePixelRatio;
  canvas.height = canvas.clientHeight * devicePixelRatio;
  
  const ctx2d = canvas.getContext("2d");
  if (ctx2d) {
    ctx2d.font = "12px Arial";
    ctx2d.fillStyle = "#4a5568"; // Dark gray text
    ctx2d.strokeStyle = "#cbd5e0"; // Light gray lines
    ctx2d.lineWidth = 1;
  }
  return ctx2d;
}

/**
 * Update Y-axis labels based on current scale and offset
 */
function updateYAxis() {
  const canvas = yAxisCanvas.value;
  if (!canvas) return;
  
  const ctx2d = initAxisCanvas(canvas);
  if (!ctx2d) return;
  
  const width = canvas.width;
  const height = canvas.height;
  const divisions = Y_AXIS_DIVISIONS;
  
  ctx2d.clearRect(0, 0, width, height);
  
  for (let i = 0; i <= divisions; i++) {
    // Calculate the data value at this position
    const value = yMax.value - (i / divisions) * (yMax.value - yMin.value);
    // Calculate position - leave room for x-axis at bottom
    const y = (i / divisions) * (height - 20);
    
    // Draw grid line
    ctx2d.beginPath();
    ctx2d.moveTo(width - 5, y);
    ctx2d.lineTo(width, y);
    ctx2d.stroke();
    
    // Draw text label
    const formattedValue = value.toFixed(1);
    ctx2d.fillText(formattedValue, 2, y + 4);
  }
}

/**
 * Update X-axis labels based on current scale and offset
 */
function updateXAxis() {
  const canvas = xAxisCanvas.value;
  if (!canvas) return;
  
  const ctx2d = initAxisCanvas(canvas);
  if (!ctx2d) return;
  
  const width = canvas.width;
  const height = canvas.height;
  const divisions = X_AXIS_DIVISIONS;
  
  ctx2d.clearRect(0, 0, width, height);
  
  // Calculate time values based on window size and sample rate
  // For simplicity, we'll use index values here
  // In a real app, you might want to use actual time or timestamps
  for (let i = 0; i <= divisions; i++) {
    const position = (i / divisions) * width;
    
    // Calculate the x value at this position
    // Convert from screen position to data coordinates
    const screenX = (2 * position / width) - 1; // Convert to [-1, 1] range
    const dataX = (screenX - wglp?.gOffsetX || 0) / (wglp?.gScaleX || 1);
    
    // Index in the data
    const index = Math.floor((dataX + 1) / 2 * windowSize.value);
    
    // Draw tick
    ctx2d.beginPath();
    ctx2d.moveTo(position, 0);
    ctx2d.lineTo(position, 5);
    ctx2d.stroke();
    
    // Draw label
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
function initPlot() {
  const canvas = plotCanvas.value;
  if (!canvas) return;
  
  const devicePixelRatio = window.devicePixelRatio || 1;
  canvas.width = canvas.clientWidth * devicePixelRatio;
  canvas.height = canvas.clientHeight * devicePixelRatio;
  
  // Create a new WebGL‑Plot instance and clear previous lines.
  wglp = new WebglPlot(canvas);
  lines.length = 0;
  
  // Create 8 data channels with preset colors.
  const numPoints = windowSize.value;
  const channelColors = [
    "#FF6384", "#36A2EB", "#FFCE56", "#4BC0C0",
    "#9966FF", "#FF9F40", "#E7E9ED", "#7CFFC4"
  ];
  for (let i = 0; i < 8; i++) {
    const color = hexToRGBA(channelColors[i]);
    const line = new WebglLine(color, numPoints);
    line.arrangeX();
    wglp.addLine(line);
    lines.push(line);
  }
  
  // If historical data exists, fill the plot.
  if (dataBuffer.value.length > 0) {
    fillLinesWithExistingData();
  }
  
  // ----- Add Interactive Overlay: Zoom Rectangle -----
  // Initially, the rectangle spans the default x-range and full y range.
  Rect = new WebglLine(new ColorRGBA(0.9, 0.9, 0.9, 1), 4);
  Rect.loop = true;
  // Using yMin and yMax for y coordinates; these values will be updated automatically.
  Rect.xy = new Float32Array([-0.5, yMin.value, -0.5, yMax.value, 0.5, yMax.value, 0.5, yMin.value]);
  Rect.visible = false;
  wglp.addLine(Rect);
  
  // ----- Register Event Listeners (Left-Click and Touch Only) -----
  const canvasEl = plotCanvas.value;
  // Removed wheel and right-click events; also no cursor style change.
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
  
  // Initialize and update axes
  updateYAxis();
  updateXAxis();
}

/**
 * Clear the data buffer and reset the plot visualization.
 */
function clearPlot() {
  // Clear the data buffer
  dataBuffer.value = [];
  
  // Re-initialize the plot with empty data
  initPlot();
  
  // Reset vertical scaling
  yMin.value = 0;
  yMax.value = 1000;
  updatePlotScale();
  
  console.log("Plot cleared");
}

/**
 * Populate plot lines with buffered data.
 */
function fillLinesWithExistingData() {
  if (!lines.length || !dataBuffer.value.length) return;
  const displayData = dataBuffer.value.slice(-windowSize.value);
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
async function refreshData() {
  if (dataUpdatePending) {
    console.log("Skipping data fetch – another fetch is in progress");
    return;
  }
  dataUpdatePending = true;
  try {
    const startTime = Date.now();
    const newData = await invoke("get_recent_data");
    const fetchTime = Date.now() - startTime;
    if (newData && newData.length > 0) {
      addToDataBuffer(newData);
      addNewDataPoints(newData);
      console.log(`Got ${newData.length} data points in ${fetchTime}ms`);
    } else {
      console.log(`No new data received (${fetchTime}ms)`);
    }
  } catch (error) {
    console.error("Error retrieving data:", error);
  } finally {
    dataUpdatePending = false;
  }
}

/**
 * Append new data to the persistent data buffer.
 */
function addToDataBuffer(newData) {
  if (!newData || newData.length === 0) return;
  dataBuffer.value = [...dataBuffer.value, ...newData];
  if (dataBuffer.value.length > MAX_BUFFER_SIZE) {
    dataBuffer.value = dataBuffer.value.slice(-MAX_BUFFER_SIZE);
  }
}

/**
 * Add new data points to the plot (via shiftAdd) and update vertical scaling.
 */
function addNewDataPoints(newData) {
  if (!newData || newData.length === 0 || !lines.length) return;
  if (Math.random() < 0.1) updateMinMax();
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
  updatePlotScale();
}

/**
 * Update the vertical scaling of the plot.
 */
function updatePlotScale() {
  if (!wglp) return;
  const range = Math.max(0.001, yMax.value - yMin.value);
  dataScale.value = 2 / range;
  dataOffset.value = -(yMin.value + yMax.value) / 2;
  wglp.gScaleY = dataScale.value;
  for (let i = 0; i < lines.length; i++) {
    lines[i].offsetY = dataOffset.value * dataScale.value;
  }
  
  // Update axis when scale changes
  updateYAxis();
}

/**
 * Compute yMin and yMax from the visible data.
 */
function updateMinMax() {
  if (!dataBuffer.value || dataBuffer.value.length === 0) return;
  const visibleData = dataBuffer.value.slice(-windowSize.value);
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
function animate() {
  if (fpsCounter === 0 && wglp) {
    wglp.update();
    // Update X-axis on regular intervals when plot is updated
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
function scheduleUpdate() {
  if (updateScheduled) return;
  const now = Date.now();
  const minTimeBetween = 30;
  if (now - lastUpdateTime < minTimeBetween) {
    const waitTime = minTimeBetween - (now - lastUpdateTime);
    updateScheduled = true;
    setTimeout(() => {
      refreshData();
      updateScheduled = false;
      lastUpdateTime = Date.now();
    }, waitTime);
  } else {
    updateScheduled = true;
    setTimeout(() => {
      refreshData();
      updateScheduled = false;
      lastUpdateTime = Date.now();
    }, 0);
  }
}

/* ====================================================
   4. Interactive Event Handlers (Left‑Click Zoom & Touch)
   ==================================================== */

/**
 * On double‑click, reset the x‑axis zoom and remove the selection rectangle.
 */
function dblClick(e) {
  e.preventDefault();
  if (wglp) {
    wglp.gScaleX = 1;
    wglp.gOffsetX = 0;
  }
  if (Rect) {
    Rect.visible = false;
  }
}

/**
 * Prevent the context menu from appearing.
 */
function contextMenu(e) {
  e.preventDefault();
}

/**
 * Mouse down (only left button) initiates a zoom selection.
 * The raw mouse coordinate is converted into the current data coordinate.
 */
function mouseDown(e) {
  e.preventDefault();
  const canvas = plotCanvas.value;
  const devicePixelRatio = window.devicePixelRatio || 1;
  const boundingRect = canvas.getBoundingClientRect();
  const relativeX = (e.clientX - boundingRect.left) * devicePixelRatio;
  if (e.button === 0) { // Left button only.
    zoomFlag = true;
    const rawX = 2 * (relativeX / canvas.width) - 1;
    // Convert raw coordinate to current data coordinate.
    cursorDownX = (rawX - wglp.gOffsetX) / wglp.gScaleX;
    if (Rect) Rect.visible = true;
  }
}

/**
 * Mouse move updates the zoom selection rectangle.
 * The x coordinate is converted to the current data coordinate.
 * The rectangle’s y coordinates are set to the current yMin and yMax.
 */
function mouseMove(e) {
  e.preventDefault();
  if (zoomFlag && Rect) {
    const canvas = plotCanvas.value;
    const devicePixelRatio = window.devicePixelRatio || 1;
    const boundingRect = canvas.getBoundingClientRect();
    const relativeX = (e.clientX - boundingRect.left) * devicePixelRatio;
    const rawX = 2 * (relativeX / canvas.width) - 1;
    const currentDataX = (rawX - wglp.gOffsetX) / wglp.gScaleX;
    // Update rectangle to cover from cursorDownX to currentDataX and full y range.
    Rect.xy = new Float32Array([
      cursorDownX, yMin.value,
      cursorDownX, yMax.value,
      currentDataX, yMax.value,
      currentDataX, yMin.value,
    ]);
    Rect.visible = true;
  }
}

/**
 * Mouse up completes the zoom selection.
 * The final x coordinate is converted to data coordinate, and then
 * the new x‑scale and x‑offset are calculated so that the selected
 * region spans the full x‑axis.
 */
function mouseUp(e) {
  e.preventDefault();
  if (zoomFlag) {
    const canvas = plotCanvas.value;
    const devicePixelRatio = window.devicePixelRatio || 1;
    const boundingRect = canvas.getBoundingClientRect();
    const relativeX = (e.clientX - boundingRect.left) * devicePixelRatio;
    const rawX = 2 * (relativeX / canvas.width) - 1;
    const cursorUpX = (rawX - wglp.gOffsetX) / wglp.gScaleX;
    const selectionWidth = Math.abs(cursorUpX - cursorDownX);
    if (selectionWidth > 0) {
      // Compute new x-scale so that the selected data range (width) maps to the full axis (width = 2).
      const newScale = 2 / selectionWidth;
      const mid = (cursorDownX + cursorUpX) / 2;
      wglp.gScaleX = newScale;
      wglp.gOffsetX = -mid * newScale;
      
      // Update X-axis after zoom
      updateXAxis();
    }
    zoomFlag = false;
    if (Rect) {
      Rect.visible = false;
    }
  }
}

/**
 * Touch handlers mimic the mouse zoom selection.
 */
function touchStart(e) {
  e.preventDefault();
  if (e.touches.length === 2) {
    // Optional: implement pinch zoom if needed.
  }
  if (e.touches.length === 1) {
    zoomFlag = true;
    initialX = e.touches[0].pageX;
    // For touch, set cursorDownX similar to mouseDown.
    const canvas = plotCanvas.value;
    const devicePixelRatio = window.devicePixelRatio || 1;
    const boundingRect = canvas.getBoundingClientRect();
    const relativeX = (e.touches[0].clientX - boundingRect.left) * devicePixelRatio;
    const rawX = 2 * (relativeX / canvas.width) - 1;
    cursorDownX = (rawX - wglp.gOffsetX) / wglp.gScaleX;
    if (Rect) Rect.visible = true;
  }
}

function touchMove(e) {
  e.preventDefault();
  if (zoomFlag && e.touches.length === 1 && Rect) {
    const canvas = plotCanvas.value;
    const devicePixelRatio = window.devicePixelRatio || 1;
    const boundingRect = canvas.getBoundingClientRect();
    const relativeX = (e.touches[0].clientX - boundingRect.left) * devicePixelRatio;
    const rawX = 2 * (relativeX / canvas.width) - 1;
    const currentDataX = (rawX - wglp.gOffsetX) / wglp.gScaleX;
    Rect.xy = new Float32Array([
      cursorDownX, yMin.value,
      cursorDownX, yMax.value,
      currentDataX, yMax.value,
      currentDataX, yMin.value,
    ]);
    Rect.visible = true;
  }
}

function touchEnd(e) {
  e.preventDefault();
  if (zoomFlag) {
    // Complete the zoom selection similar to mouseUp.
    // Note: For brevity, touch end processing is kept simple.
    zoomFlag = false;
    if (Rect) Rect.visible = false;
  }
}

/* ====================================================
   5. Lifecycle and Resize Handling
   ==================================================== */
function handleResize() {
  if (!plotCanvas.value) return;
  if (window.resizeTimeout) clearTimeout(window.resizeTimeout);
  window.resizeTimeout = setTimeout(() => {
    initPlot();
    updateYAxis();
    updateXAxis();
  }, 250);
}

onMounted(async () => {
  initPlot();
  await refreshData();
  lastUpdateTime = Date.now();
  listen("serial_data", () => {
    scheduleUpdate();
  });
  window.addEventListener("resize", handleResize);
});

watch(windowSize, () => {
  const currentBuffer = [...dataBuffer.value];
  initPlot();
  dataBuffer.value = currentBuffer;
});

onBeforeUnmount(() => {
  if (animationFrame) {
    cancelAnimationFrame(animationFrame);
    animationFrame = null;
  }
  window.removeEventListener("resize", handleResize);
});
</script>

<style scoped>
.chart-controls {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.slider-container {
  flex: 1;
  min-width: 300px;
}

.window-size-value {
  font-weight: bold;
  color: #4a5568;
  min-width: 3rem;
  display: inline-block;
}

.slider-with-labels {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-top: 0.25rem;
}

.window-size-slider {
  flex: 1;
  height: 6px;
  background: #e2e8f0;
  border-radius: 4px;
  outline: none;
  -webkit-appearance: none;
  appearance: none;
}

.window-size-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 16px;
  height: 16px;
  background: #3b82f6;
  border-radius: 50%;
  cursor: pointer;
  transition: background 0.2s;
}

.window-size-slider::-webkit-slider-thumb:hover {
  background: #2563eb;
}

.window-size-slider::-moz-range-thumb {
  width: 16px;
  height: 16px;
  background: #3b82f6;
  border-radius: 50%;
  cursor: pointer;
  border: none;
  transition: background 0.2s;
}

.window-size-slider::-moz-range-thumb:hover {
  background: #2563eb;
}

.slider-min-label,
.slider-max-label {
  font-size: 0.8rem;
  color: #718096;
  width: 3rem;
  text-align: center;
}

.refresh-btn {
  padding: 0.5rem 1rem;
  font-weight: 500;
  transition: background-color 0.2s;
}

.refresh-btn:hover {
  background-color: #2563eb;
}

/* The canvas fills the available space */
.chart-container {
  width: 100%;
  height: 400px;
  display: block;
}

.chart-visualization-container {
  position: relative;
}

.y-axis-canvas {
  position: absolute;
  left: 0;
  top: 0;
  width: 40px;
  height: 100%;
  background-color: #fff;
  border-right: 1px solid #ddd;
}

.x-axis-canvas {
  width: 100%;
  height: 40px;
  background-color: #fff;
  border-top: 1px solid #ddd;
}
</style>
