<template>
  <div class="border rounded-md shadow-sm bg-white p-4 h-[500px] mb-4">
    <div class="form-group horizontal mb-2">
      <label for="windowSize">Display Window Size:</label>
      <input id="windowSize" type="number" v-model.number="windowSize" min="1" max="5000" @change="refreshData" class="px-2 py-1 rounded border" />
      <button class="btn ml-2 px-3 py-1 bg-blue-500 text-white rounded" @click="refreshData">Refresh Data</button>
    </div>
    <!-- Responsive chart container -->
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
    xValues[i] = i + 1;
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
    const newData = await invoke('get_recent_data', { n: windowSize.value });
    displayedData.value = newData;
    if (newData && newData.length > 0 && uplotInstance) {
      const formatted = formatData(newData);
      uplotInstance.setData(formatted);
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
