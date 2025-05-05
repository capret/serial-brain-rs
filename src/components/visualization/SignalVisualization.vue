<template>
  <div class="flex-1">
    <SignalChart 
      ref="signalChart" 
      :running="isRunning"
      @crosshair-move="handleCrosshairMove"
      @quality-update="handleQualityUpdate"
    />
    <div class="grid gap-4" style="grid-template-columns: repeat(auto-fill, minmax(7rem, 1fr));">
      <ChannelStatCard
        v-for="(channel, index) in channelStats"
        :key="index"
        :channelTitle="`Channel ${index + 1}`"
        :currentValue="formatValue(channel.current)"
        :color="channelColors[index]"
        :visible="channelVisibility[index]"
        :signalQuality="channelQuality[index]"
        @color-change="onColorChange(index, $event)"
        @toggle-visibility="onToggleVisibility(index)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
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

// Define state for signal quality (true = good, false = bad)
const channelQuality = ref([
  true, true, true, true, true, true, true, true
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

// Update statistics for all channels when new data is received (commented out as not currently used)

// Handle signal quality updates from the chart component
function handleQualityUpdate(qualityData: boolean[]) {
  if (!qualityData || !Array.isArray(qualityData) || qualityData.length === 0) return;
  
  // Update signal quality for each channel
  for (let i = 0; i < Math.min(qualityData.length, channelQuality.value.length); i++) {
    channelQuality.value[i] = qualityData[i];
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
  // Serial data event listening is currently disabled
  // If needed, uncomment and import listen from '@tauri-apps/api/event'
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
