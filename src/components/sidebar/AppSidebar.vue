<template>
  <div class="w-72 flex-shrink-0 bg-gray-800 bg-opacity-60 rounded-lg p-5 overflow-y-auto">
    <!-- <h2 class="text-xl font-semibold mb-4">Available Signal</h2>
    <div class="space-y-3">
      <div
        class="bg-blue-600 bg-opacity-20 border-l-4 border-blue-500 p-4 rounded-r-lg cursor-pointer hover:bg-blue-600 hover:bg-opacity-30 transition-all duration-300 transform hover:translate-x-1">
        <h3 class="font-bold">Fake Reader</h3>
        <p class="text-sm text-gray-300">Signal Simulation</p>
      </div>
      <div
        class="bg-gray-700 p-4 rounded-lg cursor-pointer hover:bg-gray-600 transition-all duration-300 transform hover:translate-x-1">
        <h3 class="font-bold">Serial Reader</h3>
        <p class="text-sm text-gray-300">USB Connection</p>
      </div>
      <div
        class="bg-gray-700 p-4 rounded-lg cursor-pointer hover:bg-gray-600 transition-all duration-300 transform hover:translate-x-1">
        <h3 class="font-bold">Socket Reader</h3>
        <p class="text-sm text-gray-300">TCP Connection</p>
      </div>
    </div> -->

    <!-- Toggle Controls -->
    <!-- <div class="mt-8"> -->
      <h2 class="text-xl font-semibold mb-4">View Controls</h2>
      <div class="space-y-3">
        <div @click="setActiveView('visualization')" :class="[`p-4 rounded-lg cursor-pointer transition-all duration-300 transform hover:translate-x-1`,
          activeView === 'visualization'
            ? 'bg-blue-600 bg-opacity-20 border-l-4 border-blue-500 rounded-r-lg'
            : 'bg-gray-700 hover:bg-gray-600']">
          <div class="flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
              stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"></polyline>
            </svg>
            <h3 class="font-bold">Visualization</h3>
          </div>
        </div>

        <div @click="setActiveView('signal')" :class="[`p-4 rounded-lg cursor-pointer transition-all duration-300 transform hover:translate-x-1`,
          activeView === 'signal'
            ? 'bg-blue-600 bg-opacity-20 border-l-4 border-blue-500 rounded-r-lg'
            : 'bg-gray-700 hover:bg-gray-600']">
          <div class="flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
              stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M22 12h-4l-3 9L9 3l-3 9H2"></path>
            </svg>
            <h3 class="font-bold">Signal</h3>
          </div>
        </div>

        <div @click="setActiveView('filters')" :class="[`p-4 rounded-lg cursor-pointer transition-all duration-300 transform hover:translate-x-1`,
          activeView === 'filters'
            ? 'bg-blue-600 bg-opacity-20 border-l-4 border-blue-500 rounded-r-lg'
            : 'bg-gray-700 hover:bg-gray-600']">
          <div class="flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
              stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"></polygon>
            </svg>
            <h3 class="font-bold">Filters</h3>
          </div>
        </div>

        <div @click="setActiveView('folder')" :class="[`p-4 rounded-lg cursor-pointer transition-all duration-300 transform hover:translate-x-1`,
          activeView === 'folder'
            ? 'bg-blue-600 bg-opacity-20 border-l-4 border-blue-500 rounded-r-lg'
            : 'bg-gray-700 hover:bg-gray-600']">
          <div class="flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
              stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
            </svg>
            <h3 class="font-bold">Recording</h3>
          </div>
        </div>
      </div>
    <!-- </div> -->
  </div>
</template>

<script setup>


const props = defineProps({
  activeView: {
    type: String,
    required: true
  }
});

const emit = defineEmits(['update:activeView']);

function setActiveView(view) {
  emit('update:activeView', view);
}
</script>
