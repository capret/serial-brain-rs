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
        <input
          id="windowSize"
          type="range"
          v-model.number="windowSize"
          min="500"
          max="20000"
          step="500"
          @change="initPlot"
          class="w-full h-1 bg-gray-200 rounded-lg accent-blue-500"
        />
        <span class="text-xs w-12 text-center">20000</span>
      </div>
    </div>
  </div>
  <div class="rounded-md shadow-sm bg-gray-900 p-4 mb-4">
    <div
      ref="plotGrid"
      class="grid"
      :style="{
        gridTemplateColumns: '40px 1fr',
        gridTemplateRows: plotHeight + 'px 40px',
      }"
    >
      <canvas
        ref="yAxisCanvas"
        class="border-r border-gray-200"
        :style="{ width: '40px', height: plotHeight + 'px' }"
      ></canvas>
      <canvas
        ref="plotCanvas"
        class="w-full"
        :style="{ height: plotHeight + 'px' }"
      ></canvas>
      <div class="w-full"></div>
      <canvas
        ref="xAxisCanvas"
        class="border-t border-gray-200 w-full"
        style="height:40px"
      ></canvas>
    </div>
  </div>
</template>

<script setup lang="ts">
/* ====================================================
   0. Imports and Component Setup
   ==================================================== */
import {
  ref,
  onMounted,
  onBeforeUnmount,
  onActivated,
  onDeactivated,
  watch,
  nextTick,
} from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { WebglPlot, WebglLine, ColorRGBA } from 'webgl-plot';
import { chartDataBuffer } from '../../store/appState';
import { channelColors, channelVisibility } from './channelSettings';

const emit = defineEmits<{
  (event: 'crosshair-move', payload: { x: number; y: number; dataValues: number[] }): void;
}>();
const props = defineProps<{ running: boolean }>();

/* ====================================================
   1. Plot‑sizing — single source of truth
   ==================================================== */
const plotGrid = ref<HTMLDivElement | null>(null);
const plotHeight = ref<number>(300); // include axis + plot
function recalcPlotHeight(): void {
  const width = plotGrid.value?.clientWidth ?? 600;
  plotHeight.value = width / 2;
}

/* ====================================================
   2. Constants & State
   ==================================================== */
const Dpr = window.devicePixelRatio || 1;
const MAX_BUFFER_SIZE = 20000;
const Y_AXIS_DIVISIONS = 8;
const X_AXIS_DIVISIONS = 10;
const CROSSHAIR_THROTTLE_MS = 10;

const windowSize = ref<number>(1000);
const dataBuffer = chartDataBuffer;

const yMin = ref<number>(-1);
const yMax = ref<number>(1.05);
const dataScale = ref<number>(1);
const dataOffset = ref<number>(0);

/* Canvas Refs */
const plotCanvas = ref<HTMLCanvasElement | null>(null);
const xAxisCanvas = ref<HTMLCanvasElement | null>(null);
const yAxisCanvas = ref<HTMLCanvasElement | null>(null);

/* WebGL‑Plot objects */
let wglp: WebglPlot | null = null;
const lines: WebglLine[] = [];
let Rect: WebglLine | null = null;
let crossXLine: WebglLine | null = null;
let crossYLine: WebglLine | null = null;

/* Runtime control */
let animationFrame: number | null = null;
let fetchIntervalId: number | null = null;
let dataUpdatePending = false;
let fpsCounter = 0;
let fpsControl = 1;
let isActive = true;

/* Interaction helpers */
let zoomFlag = false;
let cursorDownX = 0;
let crossX = 0;
let crossY = 0;
let crosshairLastUpdateTime = 0;

/* ====================================================
   3. Helpers
   ==================================================== */
function setCanvasResolution(cv: HTMLCanvasElement | null) {
  if (!cv) return;
  cv.width = cv.clientWidth * Dpr;
  cv.height = cv.clientHeight * Dpr;
}
function hexToRGBA(hex: string): ColorRGBA {
  const v = parseInt(hex.replace('#', ''), 16);
  const r = (v >> 16) & 255;
  const g = (v >> 8) & 255;
  const b = v & 255;
  return new ColorRGBA(r / 255, g / 255, b / 255, 1);
}

/* ---------- Axis rendering ---------- */
function initAxisCanvas(canvas: HTMLCanvasElement | null): CanvasRenderingContext2D | null {
  if (!canvas) return null;
  setCanvasResolution(canvas);
  const ctx = canvas.getContext('2d');
  if (!ctx) return null;
  ctx.font = '12px Arial';
  ctx.textBaseline = 'middle';
  ctx.textAlign = 'left';
  ctx.fillStyle = '#cbd5e0';
  ctx.strokeStyle = '#cbd5e0';
  ctx.lineWidth = 2;
  return ctx;
}
function updateYAxis() {
  const ctx = initAxisCanvas(yAxisCanvas.value);
  if (!ctx) return;
  const { width, height } = yAxisCanvas.value!;
  ctx.clearRect(0, 0, width, height);
  for (let i = 0; i <= Y_AXIS_DIVISIONS; i++) {
    const value = yMax.value - (i / Y_AXIS_DIVISIONS) * (yMax.value - yMin.value);
    const y = (i / Y_AXIS_DIVISIONS) * (height - 1);
    ctx.beginPath();
    ctx.moveTo(width - 5, y);
    ctx.lineTo(width, y);
    ctx.stroke();
    ctx.fillText(value.toFixed(1), 4, y);
  }
}
function updateXAxis() {
  const ctx = initAxisCanvas(xAxisCanvas.value);
  if (!ctx || !wglp) return;
  const width = xAxisCanvas.value!.width;
  const height = xAxisCanvas.value!.height;
  ctx.clearRect(0, 0, width, height);
  for (let i = 0; i <= X_AXIS_DIVISIONS; i++) {
    const pos = (i / X_AXIS_DIVISIONS) * width;
    const screenX = (2 * pos) / width - 1;
    const dataX = (screenX - wglp.gOffsetX) / wglp.gScaleX;
    const index = Math.floor(((dataX + 1) / 2) * windowSize.value);
    ctx.beginPath();
    ctx.moveTo(pos, 0);
    ctx.lineTo(pos, 5);
    ctx.stroke();
    if (index >= 0 && index <= windowSize.value) ctx.fillText(index.toString(), pos - 10, 20);
  }
}

/* ---------- Data helpers ---------- */
function addToDataBuffer(newData: number[][]) {
  dataBuffer.push(...newData);
  if (dataBuffer.length > MAX_BUFFER_SIZE) dataBuffer.splice(0, dataBuffer.length - MAX_BUFFER_SIZE);
}
function updateMinMax() {
  if (!dataBuffer.length) return;
  const visible = dataBuffer.slice(-windowSize.value);
  let min = Infinity,
    max = -Infinity;
  visible.forEach(row => row.forEach(v => ((min = Math.min(min, v)), (max = Math.max(max, v)))));
  const range = Math.max(0.001, max - min);
  const margin = range * 0.05;
  yMin.value = min - margin;
  yMax.value = max + margin;
}
function updatePlotScale() {
  if (!wglp) return;
  const range = Math.max(0.001, yMax.value - yMin.value);
  dataScale.value = 2 / range;
  dataOffset.value = -(yMin.value + yMax.value) / 2;
  wglp.gScaleY = dataScale.value;
  wglp.gOffsetY = dataOffset.value * dataScale.value;
  updateYAxis();
  updateCrosshair(crossX, crossY);
}
function addNewDataPoints(newData: number[][]) {
  if (!lines.length) return;
  const perCh: Float32Array[] = Array.from({ length: 8 }, () => new Float32Array(newData.length));
  newData.forEach((row, i) => row.forEach((v, ch) => (perCh[ch][i] = v)));
  perCh.forEach((arr, ch) => lines[ch]?.shiftAdd(arr));
  updateMinMax();
  updatePlotScale();
}
function fillLinesWithExistingData() {
  const slice = dataBuffer.slice(-windowSize.value);
  if (!slice.length) return;
  updateMinMax();
  const offset = Math.max(0, windowSize.value - slice.length);
  for (let ch = 0; ch < 8 && ch < lines.length; ch++) {
    if (offset) {
      const first = slice[0][ch];
      for (let i = 0; i < offset; i++) lines[ch].setY(i, first);
    }
    slice.forEach((row, i) => lines[ch].setY(offset + i, row[ch]));
  }
}

/* ---------- Crosshair ---------- */
function getDataValuesAtPosition(x: number): number[] {
  const visible = dataBuffer.slice(-windowSize.value);
  if (!visible.length) return Array(8).fill(0);
  const idxCanvas = Math.round(((x + 1) / 2) * (windowSize.value - 1));
  const offset = Math.max(0, windowSize.value - visible.length);
  if (idxCanvas < offset) return Array(8).fill(0);
  const dataIdx = idxCanvas - offset;
  return visible[dataIdx] ?? Array(8).fill(0);
}
function updateCrosshair(x: number, y: number) {
  if (!crossXLine || !crossYLine) return;
  crossX = x;
  crossY = y;
  crossXLine.xy = new Float32Array([x, yMin.value, x, yMax.value]);
  crossYLine.xy = new Float32Array([-1, y, 1, y]);
  emit('crosshair-move', { x, y, dataValues: getDataValuesAtPosition(x) });
}

/* ---------- Pointer -> data coords ---------- */
function getPointerDataCoords(e: MouseEvent | TouchEvent): { dataX: number; dataY?: number } {
  const cv = plotCanvas.value!;
  const rect = cv.getBoundingClientRect();
  const clientX = 'touches' in e ? e.touches[0].clientX : e.clientX;
  const clientY = 'touches' in e ? e.touches[0].clientY : e.clientY;
  const relX = ((clientX - rect.left) * Dpr) / cv.width;
  const rawX = 2 * relX - 1;
  const dataX = (rawX - wglp!.gOffsetX) / wglp!.gScaleX;
  if ('touches' in e) return { dataX };
  const relY = ((clientY - rect.top) * Dpr) / cv.height;
  const rawY = 1 - 2 * relY;
  const dataY = (rawY - wglp!.gOffsetY) / wglp!.gScaleY;
  return { dataX, dataY };
}

/* ---------- Mouse & touch events ---------- */
function dblClick(e: MouseEvent) {
  e.preventDefault();
  if (!wglp) return;
  wglp.gScaleX = 1;
  wglp.gOffsetX = 0;
  if (Rect) Rect.visible = false;
  updateXAxis();
}
function mouseDown(e: MouseEvent) {
  if (e.button !== 0) return;
  e.preventDefault();
  zoomFlag = true;
  cursorDownX = getPointerDataCoords(e).dataX;
  if (Rect) Rect.visible = true;
}
function touchStart(e: TouchEvent) {
  e.preventDefault();
  if (e.touches.length !== 1) return;
  zoomFlag = true;
  cursorDownX = getPointerDataCoords(e).dataX;
  if (Rect) Rect.visible = true;
}
function mouseMove(e: MouseEvent) {
  e.preventDefault();
  const { dataX, dataY } = getPointerDataCoords(e);
  const now = Date.now();
  if (dataY !== undefined && now - crosshairLastUpdateTime >= CROSSHAIR_THROTTLE_MS) {
    updateCrosshair(dataX, dataY);
    crosshairLastUpdateTime = now;
  }
  if (zoomFlag && Rect) {
    Rect.xy = new Float32Array([
      cursorDownX,
      yMin.value,
      cursorDownX,
      yMax.value,
      dataX,
      yMax.value,
      dataX,
      yMin.value,
    ]);
  }
}
function touchMove(e: TouchEvent) {
  e.preventDefault();
  if (!zoomFlag || !Rect) return;
  const { dataX } = getPointerDataCoords(e);
  Rect.xy = new Float32Array([
    cursorDownX,
    yMin.value,
    cursorDownX,
    yMax.value,
    dataX,
    yMax.value,
    dataX,
    yMin.value,
  ]);
}
function mouseUp(e: MouseEvent) {
  e.preventDefault();
  if (!zoomFlag) return;
  const { dataX: upX } = getPointerDataCoords(e);
  const selWidth = Math.abs(upX - cursorDownX);
  if (selWidth > 0 && wglp) {
    wglp.gScaleX = 2 / selWidth;
    wglp.gOffsetX = -(cursorDownX + upX) / 2 * wglp.gScaleX;
    updateXAxis();
  }
  zoomFlag = false;
  if (Rect) Rect.visible = false;
}
function touchEnd(e: TouchEvent) {
  e.preventDefault();
  zoomFlag = false;
  if (Rect) Rect.visible = false;
}

/* ====================================================
   4. Plot Initialization & Animation
   ==================================================== */
function initPlot() {
  recalcPlotHeight();
  const cv = plotCanvas.value;
  if (!cv) return;
  setCanvasResolution(cv);
  wglp = new WebglPlot(cv);
  lines.length = 0;
  const numPts = windowSize.value;
  for (let i = 0; i < channelColors.length; i++) {
    const line = new WebglLine(hexToRGBA(channelColors[i]), numPts);
    line.arrangeX();
    line.visible = channelVisibility[i];
    wglp.addLine(line);
    lines.push(line);
  }
  if (dataBuffer.length) fillLinesWithExistingData();
  Rect = new WebglLine(new ColorRGBA(0.486, 1, 0.769, 0.1), 4);
  Rect.loop = true;
  Rect.xy = new Float32Array([-0.5, yMin.value, -0.5, yMax.value, 0.5, yMax.value, 0.5, yMin.value]);
  Rect.visible = false;
  wglp.addLine(Rect);
  const crossColor = new ColorRGBA(0.9, 0.1, 0.1, 1);
  crossXLine = new WebglLine(crossColor, 2);
  crossYLine = new WebglLine(crossColor, 2);
  wglp.addLine(crossXLine);
  wglp.addLine(crossYLine);
  updateCrosshair(0, (yMin.value + yMax.value) / 2);
  cv.addEventListener('touchstart', touchStart);
  cv.addEventListener('touchmove', touchMove);
  cv.addEventListener('touchend', touchEnd);
  cv.addEventListener('mousedown', mouseDown);
  cv.addEventListener('mousemove', mouseMove);
  cv.addEventListener('mouseup', mouseUp);
  cv.addEventListener('dblclick', dblClick);
  cv.addEventListener('contextmenu', e => e.preventDefault());
  if (!animationFrame) animationFrame = requestAnimationFrame(animate);
  updateYAxis();
  updateXAxis();
  updatePlotScale();
}
function animate() {
  if (!isActive) return;
  if (fpsCounter === 0 && wglp) {
    if (props.running) refreshData();
    wglp.update();
    if (Math.random() < 0.1) updateXAxis();
  }
  fpsCounter = (fpsCounter + 1) % fpsControl;
  animationFrame = requestAnimationFrame(animate);
}

/* ---------- Data refresh ---------- */
function refreshData() {
  if (!isActive || !wglp || !props.running || dataUpdatePending) return;
  dataUpdatePending = true;
  invoke<number[][]>('get_recent_data')
    .then(newData => {
      if (newData?.length) {
        addToDataBuffer(newData);
        addNewDataPoints(newData);
      }
    })
    .catch(err => console.error('Error retrieving data:', err))
    .finally(() => (dataUpdatePending = false));
}

/* ====================================================
   5. Channel helpers
   ==================================================== */
function setChannelColor(idx: number, hex: string) {
  lines[idx] && (lines[idx].color = hexToRGBA(hex));
}
function setChannelVisibility(idx: number, vis: boolean) {
  lines[idx] && (lines[idx].visible = vis);
}

function clearPlot() {
  dataBuffer.length = 0;
  yMin.value = -1;
  yMax.value = 1;
  initPlot();
  updatePlotScale();
}

/* ====================================================
   6. Lifecycle
   ==================================================== */
watch(
  () => props.running,
  val => {
    if (val) {
      refreshData();
      fetchIntervalId = setInterval(refreshData, 100);
    } else if (fetchIntervalId) {
      clearInterval(fetchIntervalId);
      fetchIntervalId = null;
    }
  },
  { immediate: true },
);
watch(windowSize, () => {
  const cache = [...dataBuffer];
  initPlot();
  dataBuffer.length = 0;
  dataBuffer.push(...cache);
  updateMinMax();
});

onMounted(async () => {
  await nextTick();
  recalcPlotHeight();
  initPlot();
  window.addEventListener('resize', handleResize);
});
function handleResize() {
  recalcPlotHeight();
  initPlot();
}

onBeforeUnmount(() => {
  isActive = false;
  if (animationFrame) cancelAnimationFrame(animationFrame);
  if (fetchIntervalId) clearInterval(fetchIntervalId);
  window.removeEventListener('resize', handleResize);
});
onActivated(() => {
  isActive = true;
  recalcPlotHeight();
  initPlot();
  window.addEventListener('resize', handleResize);
  if (props.running) fetchIntervalId = setInterval(refreshData, 100);
});
onDeactivated(() => {
  isActive = false;
  if (animationFrame) cancelAnimationFrame(animationFrame);
  if (fetchIntervalId) clearInterval(fetchIntervalId);
  window.removeEventListener('resize', handleResize);
});

defineExpose({ initPlot, setChannelColor, setChannelVisibility, clearPlot });
</script>
