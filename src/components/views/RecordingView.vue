<template>
  <div class="bg-gray-800 bg-opacity-60 rounded-lg p-6">
    <div class="flex flex-wrap justify-between items-start mb-6">
      <div class="max-[800px]:w-full">
        <h2 class="text-3xl font-bold text-blue-400">Recording Setup</h2>
      </div>
      <div class="flex gap-3 max-[800px]:w-full max-[800px]:mt-4">
        <button
          class="bg-blue-600 hover:bg-blue-700 px-6 py-3 rounded-md font-semibold flex items-center justify-center gap-2 transition-all duration-300 transform hover:scale-105 shadow-lg max-[800px]:w-full">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polygon points="5 3 19 12 5 21 5 3"></polygon>
          </svg>
          Start Recording
        </button>
      </div>
    </div>
    <div class="space-y-6">
      <div>
        <h3 class="text-lg font-semibold mb-4">Storage Location</h3>
        <div class="mt-2">
          <div class="flex items-center justify-between mb-2">
            <p class="text-sm">Select folder to record data</p>
            <span class="text-xs text-gray-400">Required</span>
          </div>
          <div class="flex">
            <input type="text" placeholder="No folder selected" readonly=""
              class="bg-gray-700 px-3 py-2 rounded-l-md flex-grow text-gray-300" />
            <button
              class="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-r-md transition-colors duration-300">
              Browse
            </button>
          </div>
        </div>
      </div>
      
      <div>
        <h3 class="text-lg font-semibold mb-4">Recording Format</h3>
        <div class="grid grid-cols-3 gap-4">
          <div class="bg-gray-700 p-4 rounded-lg border-2 border-blue-500">
            <div class="flex items-center gap-2">
              <input type="radio" name="format" checked id="csv" />
              <label for="csv" class="font-medium">CSV</label>
            </div>
            <p class="text-xs text-gray-400 mt-1">Standard format compatible with most analysis tools</p>
          </div>
          <div class="bg-gray-700 p-4 rounded-lg">
            <div class="flex items-center gap-2">
              <input type="radio" name="format" id="binary" />
              <label for="binary" class="font-medium">Binary</label>
            </div>
            <p class="text-xs text-gray-400 mt-1">Compact storage for large datasets</p>
          </div>
          <div class="bg-gray-700 p-4 rounded-lg">
            <div class="flex items-center gap-2">
              <input type="radio" name="format" id="json" />
              <label for="json" class="font-medium">JSON</label>
            </div>
            <p class="text-xs text-gray-400 mt-1">Structured format with metadata</p>
          </div>
        </div>
      </div>
      
      <div>
        <h3 class="text-lg font-semibold mb-4">Recording Options</h3>
        <div class="space-y-4">
          <div class="flex items-center">
            <input type="checkbox" id="autostart" class="mr-2" />
            <label for="autostart">Auto-start recording when signal is detected</label>
          </div>
          <div>
            <label class="block mb-2 text-sm">Maximum recording duration (minutes)</label>
            <input type="number" value="30" min="1" class="bg-gray-700 px-3 py-2 rounded-md w-32" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
// Recording view logic can be added here
</script>
