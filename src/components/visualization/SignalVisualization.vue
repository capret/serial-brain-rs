<template>
  <div class="flex-1 p-6">
    <div class="mb-4 flex justify-between items-center">
      <div>
        <h2 class="text-xl font-semibold">Signal Visualization</h2>
      </div>
    </div>
    
    <SignalChart 
      ref="signalChart" 
      @crosshair-move="handleCrosshairMove"
    />
    
    <div class="flex gap-4 ">
      <ChannelStatCard 
        v-for="(channel, index) in channelStats" 
        :key="index"
        :channelTitle="`Channel ${index + 1}`" 
        :currentValue="formatValue(channel.current)" 
        :colorClass="channelColors[index]" 
      />
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { listen } from '@tauri-apps/api/event';
import SignalChart from './SignalChart.vue';
import ChannelStatCard from './ChannelStatCard.vue';

// Define state for channel statistics
const channelStats = ref([
  { current: 0 },
  { current: 0 },
  { current: 0 },
  { current: 0 },
  { current: 0 },
  { current: 0 },
  { current: 0 },
  { current: 0 }
]);

// Channel color classes for visual differentiation
const channelColors = [
  'bg-channel-1', // #FF6384
  'bg-channel-2', // #36A2EB
  'bg-channel-3', // #FFCE56
  'bg-channel-4', // #4BC0C0
  'bg-channel-5', // #9966FF
  'bg-channel-6', // #FF9F40
  'bg-channel-7', // #E7E9ED
  'bg-channel-8'  // #7CFFC4
];

// Format numeric values with appropriate precision
function formatValue(value) {
  return value.toFixed(1);
}

// Handle crosshair movement to update current values
function handleCrosshairMove(data) {
  if (data && data.dataValues) {
    // Update current values from crosshair position
    for (let i = 0; i < Math.min(data.dataValues.length, channelStats.value.length); i++) {
      channelStats.value[i].current = data.dataValues[i];
    }
  }
}

// Update statistics for all channels when new data is received
function updateChannelStats(data) {
  if (!data || !Array.isArray(data) || data.length === 0) return;

  // Update current value for each channel
  for (let i = 0; i < Math.min(data.length, channelStats.value.length); i++) {
    channelStats.value[i].current = data[i];
  }
}

// Listen for serial data events
let unlistenFn = null;

onMounted(async () => {
  unlistenFn = await listen('serial_data', (event) => {
    // Event data is assumed to be an array of 8 channel values
    if (event.payload) {
      updateChannelStats(event.payload);
    }
  });
});

onBeforeUnmount(() => {
  if (unlistenFn) {
    unlistenFn();
  }
});
</script>

<style scoped>
/* Channel background colors matching the chart */
.bg-channel-1 {
  background-color: #FF6384;
}

.bg-channel-2 {
  background-color: #36A2EB;
}

.bg-channel-3 {
  background-color: #FFCE56;
}

.bg-channel-4 {
  background-color: #4BC0C0;
}

.bg-channel-5 {
  background-color: #9966FF;
}

.bg-channel-6 {
  background-color: #FF9F40;
}

.bg-channel-7 {
  background-color: #E7E9ED;
}

.bg-channel-8 {
  background-color: #7CFFC4;
}
</style>
