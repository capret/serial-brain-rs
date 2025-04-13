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
      <button class="btn follow-btn bg-green-500 text-white rounded" @click="followStream">Follow Stream</button>
      <button class="btn refresh-btn bg-blue-500 text-white rounded" @click="refreshData">Refresh Data</button>
    </div>
    <div ref="chartDiv" class="chart-container"></div>
  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
// If you have a module for TimeChart, import it here; otherwise assume it is globally available.
import TimeChart from 'timechart';

const windowSize = ref(1000);
// const displayedData = ref([]);
let global_index = 0;
// References for the container and the TimeChart instance.
const chartDiv = ref(null);
let timeChartInstance = null;

// Define colors for the eight channels.
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

// Create persistent data arrays that will be maintained throughout the component lifecycle
const channelDataArrays = Array.from({ length: 8 }, () => []);

// Define the initial series configuration for TimeChart.
const initialSeries = channelColors.map((color, i) => ({
  name: `Channel ${i + 1}`,
  data: channelDataArrays[i], // Reference to persistent data array
  lineWidth: 1,
  color: color
}));

// TimeChart options with an x-range that uses the slider value for its maximum (using index values).
const chartOptions = {
  series: initialSeries,
  // xRange: { min: 0, max: windowSize.value },
  realTime: true,
  xRange: { min: 0, max: 1000 },
  tooltip: { enabled: false },
  zoom: {
    x: {
      autoRange: true,
    },
    y: {
      autoRange: true,
    },
  },
};

// Initialize the TimeChart.
function initChart() {
  if (chartDiv.value) {
    // Get the parent container's dimensions for responsive sizing
    const parentHeight = chartDiv.value.parentElement.clientHeight || 500;
    // Calculate 60% of available height, with a minimum of 300px
    const containerHeight = Math.max(parentHeight * 0.6, 300);
    
    console.log("Initializing chart with responsive dimensions, height:", containerHeight);

    chartOptions.height = containerHeight;
    chartDiv.value.style.height = containerHeight + 'px';
    // Let width be controlled by CSS (100%)
    // chartOptions.realTime = true;
    // Create the TimeChart instance
    timeChartInstance = new TimeChart(chartDiv.value, chartOptions);
    // timeChartInstance.options.realTime = true;
    // timeChartInstance.options.xRange = { min: 0, max: 2000 };
    // timeChartInstance.update();
    console.log("X Range:", timeChartInstance.options.xRange);
    // Handle window resize to make chart responsive
    // window.addEventListener('resize', resizeChart);
  } else {
    console.error("Chart container element not found");
  }
}


function updateChartData(newData) {
  const n = newData.length;
  console.log("Updating chart data with", n, "points");
  // channelDataArrays.forEach(array => {
  //   while (array.length > 0) {
  //     array.pop();
  //   }
  // });
  // Add new data points
  for (let i = 0; i < n; i++) {
    const point = newData[i];
    const index = global_index;
    for (let ch = 0; ch < Math.min(8, point.length); ch++) {
      channelDataArrays[ch].push({ x: index, y: point[ch] });
    }
    global_index++;
  }
  


  const maxSize = 25000;

  // channelDataArrays.forEach(array => {
  //   if (array.length > maxSize) {
  //     console.log("Trimming array to", maxSize, "points");
  //     const excessPoints = array.length - maxSize;
  //     array.splice(0, excessPoints);
  //     array.shift(excessPoints);

  //   }
  // });
  // timeChartInstance.xRange = { min: 0, max: 1000 };
  timeChartInstance.update();

  return true; // Indicate successful update
}
async function followStream() {
  if (timeChartInstance) {
    timeChartInstance.options.realTime = true;
    timeChartInstance.update();
  }
}

// Refresh data from the backend and update the chart.
async function refreshData() {
  try {
    const newData = await invoke('get_recent_data', { n: 1000 });
    // displayedData.value = newData;
    if (newData && newData.length > 0 && timeChartInstance) {
      updateChartData(newData);
    }
  } catch (error) {
    console.error("Error retrieving data:", error);
  }
}

// Optionally throttle rapid data updates.
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
  if (chartDiv.value && timeChartInstance) {
    const parentHeight = chartDiv.value.parentElement.clientHeight || 500;
    const containerHeight = Math.max(parentHeight * 0.6, 300);
    chartDiv.value.style.height = containerHeight + 'px';
    if (typeof timeChartInstance.resize === 'function') {
      timeChartInstance.resize();
    } else {
      timeChartInstance.update();
    }
  }
}

onMounted(async () => {
  initChart();
  listen("serial_data", () => {
    scheduleUpdate();
  });
  refreshData();
});

onBeforeUnmount(() => {
  if (timeChartInstance && typeof timeChartInstance.destroy === 'function') {
    timeChartInstance.destroy();
  }
  // window.removeEventListener("resize", resizeChart);
});

// Expose refreshData to the parent component if needed.
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
  width: 100%; /* Full width */
  height: 60vh; /* Default to 60% of viewport height if JavaScript fails */
  position: relative;
  display: block;
  min-height: 300px;
  background-color: #FFFFFF;
  box-sizing: border-box;
}
</style>
