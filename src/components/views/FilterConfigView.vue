<template>
  <div class="bg-gray-800 bg-opacity-60 rounded-lg p-6">
    <div class="flex justify-between items-start mb-6">
      <div>
        <h2 class="text-3xl font-bold text-blue-400">Filter Configuration</h2>
      </div>
      <div class="flex gap-3">
        <button
          class="bg-blue-600 hover:bg-blue-700 px-6 py-3 rounded-md font-semibold flex items-center gap-2 transition-all duration-300 transform hover:scale-105 shadow-lg">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"></path>
            <polyline points="17 21 17 13 7 13 7 21"></polyline>
            <polyline points="7 3 7 8 15 8"></polyline>
          </svg>
          Save Config
        </button>
      </div>
    </div>
    <div class="mb-6">
      <div class="grid grid-cols-2 gap-4">
        <div
          class="bg-gray-700 hover:bg-gray-600 p-4 rounded-lg transition-all duration-300 transform hover:scale-[1.02] cursor-pointer">
          <div class="flex justify-between items-start mb-2">
            <h4 class="font-medium">Low Pass</h4>
            <span class="text-xs bg-blue-500 px-2 py-1 rounded">Active</span>
          </div>
          <p class="text-sm text-gray-400 mb-2">Value: 50 Hz</p>
          <div class="mt-3">
            <input type="range" min="1" max="200" value="50" class="w-full"/>
            <div class="flex justify-between text-xs text-gray-400 mt-1">
              <span>1 Hz</span>
              <span>200 Hz</span>
            </div>
          </div>
        </div>
        <div
          class="bg-gray-700 hover:bg-gray-600 p-4 rounded-lg transition-all duration-300 transform hover:scale-[1.02] cursor-pointer">
          <div class="flex justify-between items-start mb-2">
            <h4 class="font-medium">Band Pass</h4>
            <span class="text-xs bg-gray-500 px-2 py-1 rounded">Not Active</span>
          </div>
          <p class="text-sm text-gray-400 mb-2">Low: 20 Hz High: 100 Hz</p>
          <div class="mt-3 space-y-3">
            <div>
              <label class="text-xs text-gray-400">Low cutoff</label>
              <input type="range" min="1" max="50" value="20" class="w-full"/>
            </div>
            <div>
              <label class="text-xs text-gray-400">High cutoff</label>
              <input type="range" min="51" max="200" value="100" class="w-full"/>
            </div>
          </div>
        </div>
      </div>
      <div class="mt-6">
        <button class="bg-green-600 hover:bg-green-700 px-4 py-2 rounded-md font-medium mt-4">
          + Add New Filter
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
// Filter configuration logic can be added here
</script>
