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
    
    <div class="channel-card-grid">
      <ChannelStatCard 
        v-for="(channel, index) in channelStats" 
        :key="index"
        :title="`Channel ${index + 1}`" 
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
.flex-1 {
  flex: 1 1 0%;
}

.p-6 {
  padding: 1.5rem;
}
.grid {
  display: grid;
}
.grid-cols-3 {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}
.mb-4 {
  margin-bottom: 1rem;
}

.flex {
  display: flex;
}
.rounded-lg {
  border-radius: 24px;
}
.justify-between {
  justify-content: space-between;
}

.items-center {
  align-items: center;
}

.text-xl {
  font-size: 1.25rem;
  line-height: 1.75rem;
}

.font-semibold {
  font-weight: 600;
}

.text-sm {
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.text-gray-500 {
  color: rgb(107 114 128);
}

.space-x-2 > :not([hidden]) ~ :not([hidden]) {
  margin-left: 0.5rem;
}

.p-2 {
  padding: 0.5rem;
}

.rounded-md {
  border-radius: 0.375rem;
}

.border {
  border-width: 1px;
}

.border-gray-300 {
  border-color: rgb(209 213 219);
}

.hover\:bg-gray-100:hover {
  background-color: rgb(243 244 246);
}

.transition-colors {
  transition-property: color, background-color, border-color;
  transition-duration: 150ms;
}

.channel-card-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
}

.gap-4 {
  gap: 1rem;
}

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

.bg-blue-500 {
  background-color: rgb(59, 130, 246);
}

.bg-green-500 {
  background-color: rgb(34, 197, 94);
}

.bg-yellow-500 {
  background-color: rgb(234, 179, 8);
}

.bg-red-500 {
  background-color: rgb(239, 68, 68);
}

.text-white {
  color: white;
}

.py-2 {
  padding-top: 0.5rem;
  padding-bottom: 0.5rem;
}

.px-4 {
  padding-left: 1rem;
  padding-right: 1rem;
}

.rounded {
  border-radius: 0.25rem;
}

.text-lg {
  font-size: 1.125rem;
  line-height: 1.75rem;
}

.font-medium {
  font-weight: 500;
}

.mb-2 {
  margin-bottom: 0.5rem;
}

.block {
  display: block;
}

.mb-1 {
  margin-bottom: 0.25rem;
}

.w-full {
  width: 100%;
}
</style>
