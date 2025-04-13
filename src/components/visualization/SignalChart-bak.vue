<template>
  <div class="border rounded-md shadow-sm bg-white p-4 h-[500px] mb-4">
    <div class="chart-controls mb-2">
      <div class="slider-container">
        <label for="windowSize">Display Window Size: <span class="window-size-value">{{ windowSize }}</span></label>
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
      <button class="btn refresh-btn bg-blue-500 text-white rounded" @click="refreshData">Refresh Data</button>
    </div>
    <div ref="chartDiv" class="chart-container h-[420px]"></div>
  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import uPlot from 'uplot';
import 'uplot/dist/uPlot.min.css';


const windowSize = ref(1000);
const displayedData = ref([]);

// Reference for the chart container and uPlot instance.
const chartDiv = ref(null);
let uplotInstance = null;

// Define colors for eight channels.
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

// uPlot requires data as an array of arrays. The first array is the x-axis.
const initialData = [[], [], [], [], [], [], [], [], []];

// Define uPlot options and series definitions.
const series = [
  { label: "Channel 1", stroke: channelColors[0] },
  { label: "Channel 2", stroke: channelColors[1] },
  { label: "Channel 3", stroke: channelColors[2] },
  { label: "Channel 4", stroke: channelColors[3] },
  { label: "Channel 5", stroke: channelColors[4] },
  { label: "Channel 6", stroke: channelColors[5] },
  { label: "Channel 7", stroke: channelColors[6] },
  { label: "Channel 8", stroke: channelColors[7] },
];

let uplotOptions = {
  width: 800,
  height: 420,
  scales: { x: { time: false }, y: {} },
  series,
  axes: [
    { label: "Index" },
    { label: "Value" }
  ],
};

// Create or update the uPlot chart.
function initChart() {
  if (chartDiv.value) {
    uplotOptions.width = chartDiv.value.clientWidth;
    uplotInstance = new uPlot(uplotOptions, initialData, chartDiv.value);
  }
}

// Transform raw data into the format uPlot expects.
function formatData(newData) {
  const n = newData.length;
  const xValues = new Array(n);
  const channels = Array.from({ length: 8 }, () => new Array(n));
  for (let i = 0; i < n; i++) {
    xValues[i] = i + 1;  // Sequential x-values starting from 1
    const point = newData[i];
    for (let c = 0; c < 8; c++) {
      channels[c][i] = point[c];
    }
  }
  return [xValues, ...channels];
}

// Refresh data from the backend and update the chart.
async function refreshData() {
  try {
    // Remove the n parameter as backend now returns all data since last call
    const newData = await invoke('get_recent_data');
    displayedData.value = newData;
    if (newData && newData.length > 0 && uplotInstance) {
      // Process the data for uPlot and maintain a sliding window
      const currentData = uplotInstance.data;
      const formattedNewData = formatData(newData);
      
      // Only keep the latest windowSize.value data points
      let combinedData;
      if (currentData[0].length > 0) {
        // If we have existing data in the chart
        const existingDataLength = currentData[0].length;
        const newDataLength = formattedNewData[0].length;
        const totalLength = existingDataLength + newDataLength;
        
        // If combined data exceeds window size, keep only the most recent points
        if (totalLength > windowSize.value) {
          // Calculate how many points to keep from existing data
          const keepFromExisting = Math.max(0, windowSize.value - newDataLength);
          
          // For each data series (x-axis + 8 channels)
          combinedData = currentData.map((series, i) => {
            // Take the most recent points from existing data
            const keptExisting = keepFromExisting > 0 ? series.slice(-keepFromExisting) : [];
            // Append all new data
            return [...keptExisting, ...formattedNewData[i]];
          });
        } else {
          // If combined data fits within window size, keep all
          combinedData = currentData.map((series, i) => {
            return [...series, ...formattedNewData[i]];
          });
        }
      } else {
        // If there's no existing data, just use the new data
        combinedData = formattedNewData;
      }
      
      // Update x-values to be sequential
      if (combinedData[0].length > 0) {
        for (let i = 0; i < combinedData[0].length; i++) {
          combinedData[0][i] = i + 1;
        }
      }
      
      // Update the chart with the combined data
      uplotInstance.setData(combinedData);
    }
  } catch (error) {
    console.error("Error retrieving data:", error);
  }
}

// Optionally throttle data updates.
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

// Handle window resize to update the chart size.
function resizeChart() {
  if (chartDiv.value && uplotInstance) {
    const currentData = uplotInstance.data;
    uplotInstance.destroy();
    uplotOptions.width = chartDiv.value.clientWidth;
    uplotInstance = new uPlot(uplotOptions, currentData, chartDiv.value);
  }
}

// Set up listeners and initialize the chart.
onMounted(async () => {
  initChart();
  listen("serial_data", () => {
    scheduleUpdate();
  });
  refreshData();

  window.addEventListener("resize", resizeChart);
});

// Clean up listeners and chart instance.
onBeforeUnmount(() => {
  if (uplotInstance) {
    uplotInstance.destroy();
  }
  window.removeEventListener("resize", resizeChart);
});

// Expose refreshData to parent component
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

.chart-container {
  width: 100%;
  position: relative;
}
</style>
