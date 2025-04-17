<template>
  <div>
    <h3 class="text-lg font-semibold">Fake Data Generator Settings</h3>
    
    <div class="grid grid-cols-2 gap-4">
      <div>
        <label class="block text-sm mb-1">Min Value</label>
        <input type="number" class="w-full bg-gray-800 p-2 rounded" v-model.number="settings.minValue" placeholder="-100" />
      </div>
      
      <div>
        <label class="block text-sm mb-1">Max Value</label>
        <input type="number" class="w-full bg-gray-800 p-2 rounded" v-model.number="settings.maxValue" placeholder="100" />
      </div>
    </div>
    
    <div>
      <label class="block text-sm mb-1">Frequency (Hz)</label>
      <input type="number" class="w-full bg-gray-800 p-2 rounded" v-model.number="settings.frequency" 
        placeholder="500" min="0.1" max="1000" step="0.1" />
      <div class="mt-1">
        <input type="range" class="w-full" min="0.1" max="1000" step="0.1" v-model.number="settings.frequency" />
      </div>
    </div>
    
    <div class="grid grid-cols-2 gap-4">
      <div>
        <label class="block text-sm mb-1">Number of Channels</label>
        <input type="number" class="w-full bg-gray-800 p-2 rounded" v-model.number="settings.channelCount" 
          placeholder="8" min="1" max="8" />
      </div>
      
      <div>
        <label class="block text-sm mb-1">Waveform</label>
        <div class="relative">
          <select 
            v-model="settings.waveform"
            class="w-full bg-gray-800 p-2 rounded appearance-none cursor-pointer pr-8 custom-select"
            style="background-color: rgb(31, 41, 55);"
          >
            <option value="sine">Sine</option>
            <option value="square">Square</option>
            <option value="triangle">Triangle</option>
            <option value="sawtooth">Sawtooth</option>
            <option value="random">Random</option>
          </select>
          <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-white">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
              stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="6 9 12 15 18 9"></polyline>
            </svg>
          </div>
        </div>
      </div>
    </div>
    
    <div class="mt-2">
      <div class="p-3 bg-gray-600 rounded">
        <div class="flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" 
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="8" x2="12" y2="12"></line>
            <line x1="12" y1="16" x2="12.01" y2="16"></line>
          </svg>
          <span class="text-sm">Fake data is purely for testing and development purposes.</span>
          <div>
            <canvas ref="previewCanvas" class="h-80 bg-amber-400"></canvas>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
const props = defineProps({
  settings: {
    type: Object,
    required: true
  }
});
</script>
