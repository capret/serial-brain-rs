<template>
  <div class="flex-1 p-6">
    <div class="mb-4 flex justify-between items-center">
      <div>
        <h2 class="text-xl font-semibold">Signal Visualization</h2>
        <p class="text-sm text-gray-500">Real-time multi-channel data plot</p>
      </div>
      <div class="flex space-x-2">
        <button class="p-2 rounded-md border border-gray-300 hover:bg-gray-100 transition-colors">
          <span class="material-symbols-outlined">fullscreen</span>
        </button>
        <button class="p-2 rounded-md border border-gray-300 hover:bg-gray-100 transition-colors">
          <span class="material-symbols-outlined">file_download</span>
        </button>
        <button class="p-2 rounded-md border border-gray-300 hover:bg-gray-100 transition-colors">
          <span class="material-symbols-outlined">settings</span>
        </button>
      </div>
    </div>
    
    <SignalChart ref="signalChart" />
    
    <div class="grid grid-cols-4 gap-4">
      <ChannelStatCard 
        title="Channel 1" 
        currentValue="78.6" 
        minValue="32.4" 
        maxValue="96.3" 
        avgValue="56.2"
        colorClass="bg-blue-500" 
      />
      <ChannelStatCard 
        title="Channel 2" 
        currentValue="45.3" 
        minValue="21.7" 
        maxValue="65.7" 
        avgValue="61.2"
        colorClass="bg-green-500" 
      />
      <ChannelStatCard 
        title="Channel 3" 
        currentValue="82.1" 
        minValue="45.6" 
        maxValue="94.2" 
        avgValue="70.3"
        colorClass="bg-yellow-500" 
      />
      <ChannelStatCard 
        title="Channel 4" 
        currentValue="67.8" 
        minValue="30.2" 
        maxValue="75.8" 
        avgValue="68.5"
        colorClass="bg-red-500" 
      />
    </div>
  </div>
</template>

<script setup>
import SignalChart from './SignalChart.vue';
import ChannelStatCard from './ChannelStatCard.vue';

</script>

<style scoped>
.flex-1 {
  flex: 1 1 0%;
}

.p-6 {
  padding: 1.5rem;
}

.mb-4 {
  margin-bottom: 1rem;
}

.flex {
  display: flex;
}

.justify-between {
  justify-content: space-between;
}

.items-center {
  align-items: center;
}

.text-xl {
  font-size: 1.25rem;
  line-height: 1.75rem;
}

.font-semibold {
  font-weight: 600;
}

.text-sm {
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.text-gray-500 {
  color: rgb(107 114 128);
}

.space-x-2 > :not([hidden]) ~ :not([hidden]) {
  margin-left: 0.5rem;
}

.p-2 {
  padding: 0.5rem;
}

.rounded-md {
  border-radius: 0.375rem;
}

.border {
  border-width: 1px;
}

.border-gray-300 {
  border-color: rgb(209 213 219);
}

.hover\:bg-gray-100:hover {
  background-color: rgb(243 244 246);
}

.transition-colors {
  transition-property: color, background-color, border-color;
  transition-duration: 150ms;
}

.grid {
  display: grid;
}

.grid-cols-4 {
  grid-template-columns: repeat(4, minmax(0, 1fr));
}

.gap-4 {
  gap: 1rem;
}

.bg-blue-500 {
  background-color: rgb(59, 130, 246);
}

.bg-green-500 {
  background-color: rgb(34, 197, 94);
}

.bg-yellow-500 {
  background-color: rgb(234, 179, 8);
}

.bg-red-500 {
  background-color: rgb(239, 68, 68);
}

.text-white {
  color: white;
}

.py-2 {
  padding-top: 0.5rem;
  padding-bottom: 0.5rem;
}

.px-4 {
  padding-left: 1rem;
  padding-right: 1rem;
}

.rounded {
  border-radius: 0.25rem;
}

.text-lg {
  font-size: 1.125rem;
  line-height: 1.75rem;
}

.font-medium {
  font-weight: 500;
}

.mb-2 {
  margin-bottom: 0.5rem;
}

.block {
  display: block;
}

.mb-1 {
  margin-bottom: 0.25rem;
}

.w-full {
  width: 100%;
}
</style>
