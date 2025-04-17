<template>
  <div class="bg-gray-800 bg-opacity-60 rounded-lg p-6">
    <div class="flex justify-between items-start mb-6">
      <div>
        <h2 class="text-3xl font-bold text-blue-400">Visualization</h2>
      </div>
      <div class="flex gap-3">
        <button
          class="bg-blue-600 hover:bg-blue-700 px-6 py-3 rounded-md font-semibold flex items-center gap-2 transition-all duration-300 transform hover:scale-105 shadow-lg">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24"
            fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
            class="feather feather-play">
            <polygon points="5 3 19 12 5 21 5 3"></polygon>
          </svg>
          Launch
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
    <div class="mb-6">
      <h3 class="text-lg font-semibold mb-2">Display Window:</h3>
      <SignalVisualization />
    </div>
  </div>
</template>

<script setup>
import SignalVisualization from '../visualization/SignalVisualization.vue';
</script>
