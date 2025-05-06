<template>
  <div>
    <h3 class="text-lg font-semibold mb-4">Recording Options</h3>
    <div class="space-y-4">
      <div class="flex items-center">
        <input 
          type="checkbox" 
          id="autostart" 
          v-model="autostartEnabled"
          :disabled="disabled"
          class="mr-3 w-5 h-5" />
        <label for="autostart" class="text-gray-300">
          Auto-start recording when connection is established
        </label>
      </div>
      <div class="flex items-center gap-3">
        <label for="maxDuration" class="text-gray-300">Maximum recording duration (min):</label>
        <input 
          type="number" 
          id="maxDuration" 
          v-model="maxDurationValue"
          :disabled="disabled"
          min="1" 
          class="bg-gray-700 px-3 py-2 rounded-md w-32" />
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed } from 'vue';

const props = defineProps({
  autostart: {
    type: Boolean,
    required: true,
  },
  maxRecordingDuration: {
    type: Number,
    required: true,
  },
  disabled: {
    type: Boolean,
    default: false,
  },
});

const emit = defineEmits(['update:autostart', 'update:maxRecordingDuration']);

// Create computed properties for v-model
const autostartEnabled = computed({
  get() {
    return props.autostart;
  },
  set(value: boolean) {
    emit('update:autostart', value);
  }
});

const maxDurationValue = computed({
  get: () => props.maxRecordingDuration,
  set: (value) => emit('update:maxRecordingDuration', value)
});
</script>
