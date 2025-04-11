<template>
  <div class="card">
    <div class="flex items-center mb-2">
      <span :class="['w-3 h-3 rounded-full mr-2', colorClass]"></span>
      <h3 class="font-medium">{{ title }}</h3>
    </div>
    <div class="text-xl font-semibold">{{ currentValue }}</div>
    <div class="text-sm text-gray-500 mt-1 flex justify-between">
      <span>Min: <span class="font-medium text-gray-800">{{ minValue }}</span></span>
      <span>Max: <span class="font-medium text-gray-800">{{ maxValue }}</span></span>
      <span>Avg: <span class="font-medium text-gray-800">{{ avgValue }}</span></span>
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
  minValue: {
    type: [Number, String],
    default: '0'
  },
  maxValue: {
    type: [Number, String],
    default: '0'
  },
  avgValue: {
    type: [Number, String],
    default: '0'
  },
  colorClass: {
    type: String,
    default: 'bg-blue-500'
  }
});
</script>

<style scoped>
.flex {
  display: flex;
}

.items-center {
  align-items: center;
}

.justify-between {
  justify-content: space-between;
}

.mb-2 {
  margin-bottom: 0.5rem;
}

.mt-1 {
  margin-top: 0.25rem;
}

.mr-2 {
  margin-right: 0.5rem;
}

.w-3 {
  width: 0.75rem;
}

.h-3 {
  height: 0.75rem;
}

.rounded-full {
  border-radius: 9999px;
}

.text-xl {
  font-size: 1.25rem;
  line-height: 1.75rem;
}

.text-sm {
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.font-medium {
  font-weight: 500;
}

.font-semibold {
  font-weight: 600;
}

.text-gray-500 {
  color: rgb(107 114 128);
}

.text-gray-800 {
  color: rgb(31 41 55);
}

.bg-blue-500 {
  background-color: rgb(59 130 246);
}

.bg-green-500 {
  background-color: rgb(34 197 94);
}

.bg-yellow-500 {
  background-color: rgb(234 179 8);
}

.bg-red-500 {
  background-color: rgb(239 68 68);
}
</style>
