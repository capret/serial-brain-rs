<template>
  <details class="panel">
    <summary class="panel-header">
      <span class="font-medium">Signal Filters</span>
      <span class="material-symbols-outlined transition-transform group-open:rotate-180">expand_more</span>
    </summary>
    <div class="panel-body">
      <div class="space-y-3 mb-4">
        <div class="p-3 bg-gray-50 rounded-md relative">
          <div class="flex justify-between">
            <span class="font-medium">Low-pass Filter</span>
            <button class="text-red-500 hover:text-red-700">
              <span class="material-symbols-outlined">delete</span>
            </button>
          </div>
          <div class="mt-2">
            <label class="form-label">Cutoff Frequency (Hz)</label>
            <input type="number" class="form-input" value="10" />
          </div>
        </div>
      </div>
      <button class="btn btn-secondary w-full">
        <span class="material-symbols-outlined mr-1">add</span> Add Filter
      </button>
    </div>
  </details>
</template>

<script setup>
// Signal filters component logic
</script>
