<template>
  <div class="bg-gray-800 bg-opacity-60 rounded-lg p-6">
    <div class="flex flex-wrap justify-between items-start mb-6">
      <div>
        <h2 class="text-3xl font-bold text-blue-400">Signal Visualization</h2>
      </div>
      <div class="flex flex-wrap items-center gap-3 max-[800px]:mt-4">
        <button @click="toggleRunning"
          :disabled="isLaunchDisabled"
          :class="[
            'px-6 py-3 rounded-md font-semibold flex items-center gap-2 transition-all duration-300 transform hover:scale-105 shadow-lg',
            isLaunchDisabled ? 'bg-gray-500 opacity-50 cursor-not-allowed' : 'bg-blue-600 hover:bg-blue-700'
          ]"
         >
          <svg v-if="!isRunning" xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24"
            fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
            class="feather feather-play">
            <polygon points="5 3 19 12 5 21 5 3"></polygon>
          </svg>
          <svg v-else xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24"
            fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="6" y1="4" x2="6" y2="20"></line>
            <line x1="18" y1="4" x2="18" y2="20"></line>
          </svg>
          {{ isRunning ? 'Pause' : 'Launch' }}
        </button>
        <button @click="clearPlot" class="px-6 py-3 rounded-md font-semibold flex items-center gap-2 transition-all duration-300 transform hover:scale-105 shadow-lg bg-gray-600 hover:bg-gray-700 text-white">
          Clear Plot
        </button>
        <details class="relative inline-block">
          <summary
            class="bg-gray-700 hover:bg-gray-600 px-3 py-3 rounded-md cursor-pointer transition-all duration-300 list-none flex items-center justify-center">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
              stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="1"></circle>
              <circle cx="19" cy="12" r="1"></circle>
              <circle cx="5" cy="12" r="1"></circle>
            </svg>
          </summary>
          <div
            class="absolute right-0 mt-2 w-48 bg-gray-700 rounded-md shadow-xl z-20 overflow-hidden text-sm border border-gray-600">
            <a href="#" class="block px-4 py-2 hover:bg-gray-600 transition-colors duration-200">
              Export Data
            </a>
            <a href="#" class="block px-4 py-2 hover:bg-gray-600 transition-colors duration-200">
              Report Issue
            </a>
          </div>
        </details>
      </div>
    </div>
    <div class="mb-2">
      <SignalVisualization ref="viz" :running="isRunning" />
    </div>
  </div>
</template>

<script setup>
import SignalVisualization from '../visualization/SignalVisualization.vue';
import { ref, computed } from 'vue';
import { isRunning, connectionStatus } from '../../store/appState';

// ref to child component for clearing
const viz = ref(null);
/**
 * Invoke clearPlot exposed by SignalVisualization
 */
function clearPlot() {
  viz.value?.clearPlot();
}

function toggleRunning() {
  isRunning.value = !isRunning.value;
}

const isLaunchDisabled = computed(() => connectionStatus.value !== 'connected');
</script>
