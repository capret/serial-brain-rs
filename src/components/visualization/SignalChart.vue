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

const emit = defineEmits(['crosshair-move']);

/* ====================================================
   1. State and Plot Settings
   ==================================================== */
const windowSize = ref(1000);
const dataBuffer = ref([]);
const MAX_BUFFER_SIZE = 50000;
const yMin = ref(-1);
const yMax = ref(1.05);
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

/* Crosshair & Zoom variables */
let Rect = null;
let zoomFlag = false;
let cursorDownX = 0;
let initialX = 0;
let crossXLine = null;
let crossYLine = null;
let crossX = 0;
let crossY = 0;

/* Axis config */
const Y_AXIS_DIVISIONS = 8;
const X_AXIS_DIVISIONS = 10;

/* ====================================================
   2. Utility Functions
   ==================================================== */
function hexToRGBA(hex) {
  hex = hex.replace("#", "");
  const bigint = parseInt(hex, 16);
  return new ColorRGBA(
    ((bigint >> 16) & 255) / 255,
    ((bigint >> 8) & 255) / 255,
    (bigint & 255) / 255,
    1
  );
}

function initAxisCanvas(canvas) {
  if (!canvas) return null;
  const dpr = window.devicePixelRatio || 1;
  canvas.width = canvas.clientWidth * dpr;
  canvas.height = canvas.clientHeight * dpr;
  const ctx = canvas.getContext("2d");
  if (ctx) {
    ctx.font = "12px Arial";
    ctx.fillStyle = "#4a5568";
    ctx.strokeStyle = "#cbd5e0";
    ctx.lineWidth = 1;
  }
  return ctx;
}

function updateYAxis() {
  const canvas = yAxisCanvas.value;
  const ctx = initAxisCanvas(canvas);
  if (!ctx) return;
  const { width, height } = canvas;
  ctx.clearRect(0, 0, width, height);
  for (let i = 0; i <= Y_AXIS_DIVISIONS; i++) {
    const value = yMax.value - (i / Y_AXIS_DIVISIONS) * (yMax.value - yMin.value);
    const y = (i / Y_AXIS_DIVISIONS) * (height - 20);
    ctx.beginPath();
    ctx.moveTo(width - 5, y);
    ctx.lineTo(width, y);
    ctx.stroke();
    ctx.fillText(value.toFixed(1), 2, y + 4);
  }
}

function updateXAxis() {
  const canvas = xAxisCanvas.value;
  const ctx = initAxisCanvas(canvas);
  if (!ctx || !wglp) return;
  const { width, height } = canvas;
  ctx.clearRect(0, 0, width, height);
  for (let i = 0; i <= X_AXIS_DIVISIONS; i++) {
    const pos = (i / X_AXIS_DIVISIONS) * width;
    ctx.beginPath();
    ctx.moveTo(pos, 0);
    ctx.lineTo(pos, 5);
    ctx.stroke();
    const screenX = (2 * pos / width) - 1;
    const dataX = (screenX - wglp.gOffsetX) / wglp.gScaleX;
    const idx = Math.floor((dataX + 1) / 2 * windowSize.value);
    if (idx >= 0 && idx <= windowSize.value) {
      ctx.fillText(idx.toString(), pos - 10, 20);
    }
  }
}

/* ====================================================
   3. Plot Init & Data Handling
   ==================================================== */
function initPlot() {
  const canvas = plotCanvas.value;
  if (!canvas) return;
  const dpr = window.devicePixelRatio || 1;
  canvas.width = canvas.clientWidth * dpr;
  canvas.height = canvas.clientHeight * dpr;

  wglp = new WebglPlot(canvas);
  lines.length = 0;

  // create 8 lines
  const colors = ["#FF6384","#36A2EB","#FFCE56","#4BC0C0","#9966FF","#FF9F40","#E7E9ED","#7CFFC4"];
  for (let i = 0; i < 8; i++) {
    const ln = new WebglLine(hexToRGBA(colors[i]), windowSize.value);
    ln.arrangeX();
    wglp.addLine(ln);
    lines.push(ln);
  }

  // fill old data
  if (dataBuffer.value.length) fillLinesWithExistingData();

  // zoom rect
  Rect = new WebglLine(new ColorRGBA(0.9,0.9,0.9,1),4);
  Rect.loop = true;
  Rect.visible = false;
  wglp.addLine(Rect);

  // crosshairs
  const crossC = new ColorRGBA(0.9,0.1,0.1,1);
  crossXLine = new WebglLine(crossC,2);
  crossYLine = new WebglLine(crossC,2);
  crossX = 0;
  crossY = (yMin.value + yMax.value)/2;
  crossXLine.xy = new Float32Array([crossX, yMin.value, crossX, yMax.value]);
  crossYLine.xy = new Float32Array([-1, crossY, 1, crossY]);
  wglp.addLine(crossXLine);
  wglp.addLine(crossYLine);

  // events
  const cEl = canvas;
  cEl.addEventListener("mousedown", mouseDown);
  cEl.addEventListener("mousemove", mouseMove);
  cEl.addEventListener("mouseup", mouseUp);
  cEl.addEventListener("dblclick", dblClick);
  cEl.addEventListener("contextmenu", contextMenu);
  cEl.addEventListener("touchstart", touchStart);
  cEl.addEventListener("touchmove", touchMove);
  cEl.addEventListener("touchend", touchEnd);

  if (!animationFrame) animationFrame = requestAnimationFrame(animate);
  updateYAxis();
  updateXAxis();
}

function clearPlot() {
  dataBuffer.value = [];
  initPlot();
  yMin.value = -1;
  yMax.value = 1;
  updatePlotScale();
  console.log("Plot cleared");
}

function fillLinesWithExistingData() {
  const disp = dataBuffer.value.slice(-windowSize.value);
  if (!lines.length || !disp.length) return;
  updateMinMax();
  const offset = Math.max(0, windowSize.value - disp.length);
  for (let ch=0; ch<lines.length; ch++) {
    const line = lines[ch];
    const first = disp.length ? disp[0][ch] : 0;
    for (let i=0; i<offset; i++) line.setY(i, first);
    for (let i=0; i<disp.length; i++) line.setY(offset+i, disp[i][ch]);
  }
}

/**
 * Refresh data without blocking the UI
 */
function refreshData() {
  if (dataUpdatePending) {
    console.log("Skipping data fetch – another fetch is in progress");
    return;
  }
  dataUpdatePending = true;
  const startTime = Date.now();

  invoke("get_recent_data")
    .then((newData) => {
      const fetchTime = Date.now() - startTime;
      if (newData && newData.length) {
        addToDataBuffer(newData);
        addNewDataPoints(newData);
      }
    })
    .catch((err) => {
      console.error("Error retrieving data:", err);
    })
    .finally(() => {
      dataUpdatePending = false;
    });
}

function addToDataBuffer(newData) {
  dataBuffer.value = [...dataBuffer.value, ...newData];
  if (dataBuffer.value.length > MAX_BUFFER_SIZE) {
    dataBuffer.value = dataBuffer.value.slice(-MAX_BUFFER_SIZE);
  }
}

function addNewDataPoints(newData) {
  if (!newData.length || !lines.length) return;
  if (Math.random()<0.1) updateMinMax();
  const rawArrays = newData[0].map((_,ch) => {
    const arr = new Float32Array(newData.length);
    for (let i=0; i<newData.length; i++) arr[i] = newData[i][ch];
    return arr;
  });
  lines.forEach((ln,ch) => ln.shiftAdd(rawArrays[ch]));
  updatePlotScale();
}

function updatePlotScale() {
  if (!wglp) return;
  const range = Math.max(0.001, yMax.value - yMin.value);
  dataScale.value = 2 / range;
  dataOffset.value = -(yMin.value + yMax.value) / 2;
  wglp.gScaleY = dataScale.value;
  lines.forEach(ln => ln.offsetY = dataOffset.value * dataScale.value);
  updateYAxis();
  if (crossXLine && crossYLine) updateCrosshair(crossX, crossY);
}

function updateMinMax() {
  const visible = dataBuffer.value.slice(-windowSize.value);
  if (!visible.length) return;
  let minV = Infinity, maxV = -Infinity;
  visible.forEach(pt => pt.forEach(v => {
    minV = Math.min(minV, v);
    maxV = Math.max(maxV, v);
  }));
  const m = Math.max(0.001, maxV - minV) * 0.05;
  yMin.value = minV - m;
  yMax.value = maxV + m;
  updatePlotScale();
}

/* Animation & throttling */
function animate() {
  if (fpsCounter === 0 && wglp) {
    wglp.update();
    if (Math.random()<0.1) updateXAxis();
  }
  fpsCounter = (fpsCounter + 1) % fpsControl;
  animationFrame = requestAnimationFrame(animate);
}

function scheduleUpdate() {
  if (updateScheduled) return;
  const now = Date.now();
  const delta = 16;
  const wait = Math.max(0, delta - (now - lastUpdateTime));
  updateScheduled = true;
  setTimeout(() => {
    refreshData();
    lastUpdateTime = Date.now();
    updateScheduled = false;
  }, wait);
}

/* Crosshair & events (dblClick, mouseDown, mouseMove, mouseUp, touchStart, touchMove, touchEnd, contextMenu) */
// …reuse all your existing event handler definitions here…

/* ====================================================
   5. Lifecycle & Resize
   ==================================================== */
function handleResize() {
  if (window.resizeTimeout) clearTimeout(window.resizeTimeout);
  window.resizeTimeout = setTimeout(() => {
    initPlot();
    updateYAxis();
    updateXAxis();
  }, 50);
}

onMounted(() => {
  initPlot();
  refreshData();
  lastUpdateTime = Date.now();
  listen("serial_data", () => scheduleUpdate());
  window.addEventListener("resize", handleResize);
});

onBeforeUnmount(() => {
  if (animationFrame) cancelAnimationFrame(animationFrame);
  window.removeEventListener("resize", handleResize);
});

watch(windowSize, () => {
  const buf = [...dataBuffer.value];
  initPlot();
  dataBuffer.value = buf;
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

.slider-min-label, .slider-max-label {
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

.chart-container {
  width: 100%;
  position: relative;
}
</style>
