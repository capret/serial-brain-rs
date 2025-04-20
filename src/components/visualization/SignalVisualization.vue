<template>
  <div class="flex-1 p-6">
    <SignalChart 
      ref="signalChart" 
      :running="isRunning"
      @crosshair-move="handleCrosshairMove"
    />
    <div class="flex flex-wrap gap-4 ">
      <ChannelStatCard
        v-for="(channel, index) in channelStats"
        :key="index"
        :channelTitle="`Channel ${index + 1}`"
        :currentValue="formatValue(channel.current)"
        :color="channelColors[index]"
        :visible="channelVisibility[index]"
        @color-change="onColorChange(index, $event)"
        @toggle-visibility="onToggleVisibility(index)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { listen } from '@tauri-apps/api/event';
import SignalChart from './SignalChart.vue';
import ChannelStatCard from './ChannelStatCard.vue';
import { channelColors, channelVisibility } from './channelSettings';
import { isRunning } from '../../store/appState';

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

const signalChart = ref<InstanceType<typeof SignalChart> | null>(null);

// Format numeric values with appropriate precision
function formatValue(value: number) {
  return value.toFixed(1);
}

// Handle crosshair movement to update current values
function handleCrosshairMove(data: any) {
  if (data && data.dataValues) {
    // Update current values from crosshair position
    for (let i = 0; i < Math.min(data.dataValues.length, channelStats.value.length); i++) {
      channelStats.value[i].current = data.dataValues[i];
    }
  }
}

// Update statistics for all channels when new data is received
function updateChannelStats(data: number[]) {
  if (!data || !Array.isArray(data) || data.length === 0) return;

  // Update current value for each channel
  for (let i = 0; i < Math.min(data.length, channelStats.value.length); i++) {
    channelStats.value[i].current = data[i];
  }
}

function onColorChange(index: number, newColor: string) {
  channelColors[index] = newColor;
  signalChart.value?.setChannelColor(index, newColor);
}

// Toggle channel line and card visibility
function onToggleVisibility(index: number) {
  channelVisibility[index] = !channelVisibility[index];
  signalChart.value?.setChannelVisibility(index, channelVisibility[index]);
}

// Listen for serial data events
let unlistenFn: any = null;

onMounted(async () => {
  unlistenFn = await listen('serial_data', (event: any) => {
    // Update stats only when running
    if (event.payload && isRunning.value) {
      updateChannelStats(event.payload);
    }
  });
});

onBeforeUnmount(() => {
  if (unlistenFn) {
    unlistenFn();
  }
});

/**
 * Clear the chart by delegating to SignalChart.clearPlot()
 */
function clearPlot(): void {
  signalChart.value?.clearPlot();
}

// expose signalChart and clearPlot for parent access
defineExpose({ signalChart, clearPlot });
</script>

<style scoped>

</style>
