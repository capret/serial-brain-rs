<template>
  <div class="bg-gray-700 p-3 rounded-lg w-30">
    <div>
      <div class="flex items-center gap-2">
        <span :class="['w-3 h-3 rounded-full', colorClass]"></span>
        <p class="text-sm text-gray-400 m-0">{{ title }}</p>
      </div>
      <div class="pl-5">
        <p class="font-semibold m-0">{{ currentValue }}</p>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  title: {
    type: String,
    required: true
  },
  currentValue: {
    type: [Number, String],
    required: true
  },
  colorClass: {
    type: String,
    default: 'bg-blue-500'
  }
});
</script>

<style scoped>
</style>
