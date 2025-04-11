<template>
  <details class="panel" open>
    <summary class="panel-header">
      <span class="font-medium">Data Source</span> 
      <span class="material-symbols-outlined transition-transform group-open:rotate-180">expand_more</span>
    </summary>
    <div class="panel-body">
      <div class="space-y-2">
        <label class="flex items-center space-x-2 cursor-pointer p-2 hover:bg-gray-100 rounded">
          <input type="radio" name="dataSource" value="serial" v-model="selectedSource" class="w-4 h-4 text-primary-600 focus:ring-primary-500" />
          <span>Serial Connection</span>
        </label>
        <label class="flex items-center space-x-2 cursor-pointer p-2 hover:bg-gray-100 rounded">
          <input type="radio" name="dataSource" value="tcp" v-model="selectedSource" class="w-4 h-4 text-primary-600 focus:ring-primary-500" />
          <span>TCP Connection</span>
        </label>
        <label class="flex items-center space-x-2 cursor-pointer p-2 hover:bg-gray-100 rounded">
          <input type="radio" name="dataSource" value="fake" v-model="selectedSource" class="w-4 h-4 text-primary-600 focus:ring-primary-500" />
          <span>Fake Data Generator</span>
        </label>
      </div>
    </div>
  </details>
</template>

<script setup>
import { ref, defineExpose, watch } from 'vue';

const selectedSource = ref('serial');

// Expose the selected data source to parent components
defineExpose({
  getSelectedSource: () => selectedSource.value
});
</script>

<style scoped>
.space-x-2 > :not([hidden]) ~ :not([hidden]) {
  margin-left: 0.5rem;
}

.rounded {
  border-radius: 0.25rem;
}

.hover\:bg-gray-100:hover {
  background-color: rgb(243 244 246);
}

.w-4 {
  width: 1rem;
}

.h-4 {
  height: 1rem;
}

.cursor-pointer {
  cursor: pointer;
}

.p-2 {
  padding: 0.5rem;
}

.font-medium {
  font-weight: 500;
}

.text-primary-600 {
  color: rgb(99 27 255);
}

.focus\:ring-primary-500:focus {
  --tw-ring-color: rgb(115 65 255);
  box-shadow: 0 0 0 3px rgba(115, 65, 255, 0.5);
}

.material-symbols-outlined {
  font-variation-settings: 'FILL' 0, 'wght' 400, 'GRAD' 0, 'opsz' 24;
}

.transition-transform {
  transition-property: transform;
  transition-duration: 150ms;
}
</style>
