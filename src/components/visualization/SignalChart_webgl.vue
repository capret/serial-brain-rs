<template>
  <div class="border rounded-md shadow-sm bg-white p-4 h-[500px] mb-4">
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
            @change="refreshData"   
            class="window-size-slider" 
          />
          <span class="slider-max-label">20000</span>
        </div>
      </div>
      <button class="btn refresh-btn bg-blue-500 text-white rounded" @click="refreshData">
        Refresh Data
      </button>
    </div>
    <!-- Use a canvas element for WebGL‑Plot -->
    <canvas ref="plotCanvas" class="chart-container h-[420px]"></canvas>
  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import {WebglPlot, WebglLine, ColorRGBA } from 'webgl-plot';

// Reactive variables for window size and the latest data received
const windowSize = ref(1000);
const displayedData = ref([]); // Expected data shape: an array with length = windowSize, where each element is an array of 8 channel values

// Reference to the canvas element
const plotCanvas = ref(null);

// WebglPlot instance and a list to hold each channel’s line instance
let wglp = null;
const lines = [];

// Predefined channel colors (in hex) for 8 channels.
const channelColors = [
  "#FF6384",
  "#36A2EB",
  "#FFCE56",
  "#4BC0C0",
  "#9966FF",
  "#FF9F40",
  "#E7E9ED",
  "#7CFFC4",
];

// Helper: Convert a hex color to a normalized ColorRGBA object.
function hexToRGBA(hex) {
  hex = hex.replace('#','');
  const bigint = parseInt(hex, 16);
  const r = (bigint >> 16) & 255;
  const g = (bigint >> 8) & 255;
  const b = bigint & 255;
  return new ColorRGBA(r / 255, g / 255, b / 255, 1);
}

// Initialize (or reinitialize) the WebGL‑Plot canvas and lines.
function initPlot() {
  const canvas = plotCanvas.value;
  if (!canvas) return;
  
  // Adjust canvas dimensions to account for the devicePixelRatio
  const devicePixelRatio = window.devicePixelRatio || 1;
  canvas.width = canvas.clientWidth * devicePixelRatio;
  canvas.height = canvas.clientHeight * devicePixelRatio;
  
  // Create a new WebglPlot instance
  wglp = new WebglPlot(canvas);
  
  // Clear any previous lines if reinitializing
  lines.length = 0;
  
  // Create one line per channel using the current window size for the number of points.
  const numPoints = windowSize.value;
  for (let i = 0; i < 8; i++) {
    const color = hexToRGBA(channelColors[i]);
    // Create a WebglLine where the second parameter is the number of points.
    const line = new WebglLine(color, numPoints);
    // Generate equally spaced x-values from -1 to 1.
    line.lineSpaceX(-1, 2 / numPoints);
    wglp.addLine(line);
    lines.push(line);
  }
}

// Retrieve data from the backend. The backend should return an array of length "n"
// where each element is an array of 8 channel values.
async function refreshData() {
  try {
    const newData = await invoke('get_recent_data', { n: windowSize.value });
    displayedData.value = newData;
  } catch (error) {
    console.error("Error retrieving data:", error);
  }
}

// Update each line’s y-values with the latest data.
// Assumes displayedData.value is an array of [n, 8]
function updatePlot() {
  if (!displayedData.value || displayedData.value.length === 0) return;
  const n = displayedData.value.length;
  for (let i = 0; i < n; i++) {
    const point = displayedData.value[i];
    // Loop over each channel (we expect 8 values per data point)
    for (let ch = 0; ch < 8; ch++) {
      lines[ch].setY(i, point[ch]);
    }
  }
}

// Animation loop: update the plot with new data then schedule the next frame.
function animate() {
  updatePlot();
  if (wglp) wglp.update();
  requestAnimationFrame(animate);
}

// Optionally throttle backend data updates
let updateScheduled = false;
function scheduleUpdate() {
  if (!updateScheduled) {
    updateScheduled = true;
    setTimeout(() => {
      refreshData();
      updateScheduled = false;
    }, 50);
  }
}

// On window resize, reinitialize the plot to fit the new dimensions.
function handleResize() {
  if (plotCanvas.value) {
    initPlot();
  }
}

onMounted(async () => {
  // Initialize the plot canvas and create the lines
  initPlot();
  
  // Load the initial data
  await refreshData();
  
  // Start the animation loop (dynamic updates)
  requestAnimationFrame(animate);
  
  // Listen for Tauri events (for example "serial_data") to trigger data refreshes.
  listen("serial_data", () => {
    scheduleUpdate();
  });
  
  // Update the plot on resize
  window.addEventListener("resize", handleResize);
});

// When windowSize changes (via slider), reinitialize the plot and refresh data.
watch(windowSize, () => {
  initPlot();
  refreshData();
});

// Clean up the resize listener
onBeforeUnmount(() => {
  window.removeEventListener("resize", handleResize);
});

// Expose refreshData so that the parent component can trigger a refresh if needed.
defineExpose({ refreshData });
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

/* The canvas fills the available space */
.chart-container {
  width: 100%;
  height: 100%;
  display: block;
}
</style>
