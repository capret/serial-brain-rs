<template>
  <div>
    <h3 class="text-lg font-semibold mb-4">{{ $t('recording.format') }}</h3>
    <div class="grid grid-cols-3 gap-4">
      <div 
        class="bg-gray-700 p-4 rounded-lg cursor-pointer" 
        :class="{'border-2 border-blue-500': modelValue === 'csv'}"
        @click="updateFormat('csv')">
        <div class="flex items-center gap-2">
          <input type="radio" name="format" :checked="modelValue === 'csv'" id="csv" />
          <label for="csv" class="font-medium">{{ $t('recording.csv') }}</label>
        </div>
        <p class="text-xs text-gray-400 mt-1">{{ $t('recording.csvDescription') }}</p>
      </div>
      <div 
        class="bg-gray-700 p-4 rounded-lg cursor-pointer" 
        :class="{'border-2 border-blue-500': modelValue === 'binary'}"
        @click="updateFormat('binary')">
        <div class="flex items-center gap-2">
          <input type="radio" name="format" :checked="modelValue === 'binary'" id="binary" />
          <label for="binary" class="font-medium">{{ $t('recording.binary') }}</label>
        </div>
        <p class="text-xs text-gray-400 mt-1">{{ $t('recording.binaryDescription') }}</p>
      </div>
      <div 
        class="bg-gray-700 p-4 rounded-lg cursor-pointer" 
        :class="{'border-2 border-blue-500': modelValue === 'json'}"
        @click="updateFormat('json')">
        <div class="flex items-center gap-2">
          <input type="radio" name="format" :checked="modelValue === 'json'" id="json" />
          <label for="json" class="font-medium">{{ $t('recording.json') }}</label>
        </div>
        <p class="text-xs text-gray-400 mt-1">{{ $t('recording.jsonDescription') }}</p>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
// Define props
const props = defineProps({
  modelValue: {
    type: String,
    required: true
  },
  disabled: {
    type: Boolean,
    default: false
  }
});

// Define emit
const emit = defineEmits(['update:modelValue']);

// Method to update the format
const updateFormat = (format: string) => {
  if (!props.disabled) {
    emit('update:modelValue', format);
  }
};
</script>
