<template>
  <div class="bg-gray-800 bg-opacity-60 rounded-lg p-6">
    <div class="flex flex-wrap items-center mb-6 gap-3">
      <h2 class="text-3xl font-bold text-blue-400 flex-shrink-0">{{ $t('filter.title') }}</h2>
      <div class="flex items-center gap-3 flex-grow min-w-0">
        <select v-model="newFilterType" class="bg-gray-700 text-white p-2 rounded">
          <option v-for="opt in filterTypes" :key="opt.value" :value="opt.value">{{opt.label}}</option>
        </select>
        <button @click="addFilter" class="bg-green-600 hover:bg-green-700 px-4 py-2 rounded-md font-medium">
          + {{ $t('filter.addNew') }}
        </button>
      </div>
      <button @click="saveConfig" class="bg-blue-600 hover:bg-blue-700 px-6 py-3 rounded-md font-semibold flex items-center gap-2 transition-all duration-300 transform hover:scale-105 shadow-lg flex-grow min-w-0 justify-center">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
          stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"></path>
          <polyline points="17 21 17 13 7 13 7 21"></polyline>
          <polyline points="7 3 7 8 15 8"></polyline>
        </svg>
        {{ $t('filter.saveConfig') }}
      </button>
    </div>
    <div class="mb-6">
      <transition-group name="filter" tag="div" class="grid gap-4" style="grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));">
        <div v-for="filter in filters" :key="filter.id"
          class="bg-gray-700 hover:bg-gray-600 p-4 rounded-lg transition-all duration-300 transform hover:scale-[1.02]">
          <div class="flex justify-between items-start mb-2">
            <h4 class="font-medium">{{ displayName(filter.type) }}</h4>
            <div class="flex items-center gap-2">
              <button @click="toggleActive(filter)" :class="['text-xs px-2 py-1 rounded', filter.active ? 'bg-green-500' : 'bg-gray-500']">
                {{ filter.active ? $t('filter.active') : $t('filter.notActive') }}
              </button>
              <button @click="deleteFilter(filter)" class="text-xs p-1 rounded bg-gray-500 hover:bg-gray-400 text-white">
                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7"/>
                  <path stroke-linecap="round" stroke-linejoin="round" d="M10 11v6M14 11v6"/>
                  <path stroke-linecap="round" stroke-linejoin="round" d="M9 7h6"/>
                </svg>
              </button>
            </div>
          </div>
          <template v-if="filter.type==='lowpass'||filter.type==='highpass'">
            <div class="flex items-center gap-2 mb-2">
              <label class="text-sm text-gray-400">{{ $t('filter.value') }}:</label>
              <input type="number" min="1" max="200" v-model.number="filter.value" class="w-20  text-white p-1 rounded"/>
              <span class="text-sm text-gray-400">Hz</span>
            </div>
            <div class="mt-3">
              <input type="range" min="1" max="200" v-model.number="filter.value" class="w-full accent-blue-500"/>
              <div class="flex justify-between text-xs text-gray-400 mt-1">
                <span>1 Hz</span>
                <span>200 Hz</span>
              </div>
            </div>
          </template>
          <template v-else-if="filter.type==='bandpass'">
            <div class="flex items-center gap-2 mb-2">
              <label class="text-sm text-gray-400">{{ $t('filter.low') }}:</label>
              <input type="number" min="1" max="200" v-model.number="filter.lowValue" @input="ensureHigh(filter)" class="w-16 text-white p-1 rounded"/>
              <span class="text-sm text-gray-400">Hz</span>
              <label class="text-sm text-gray-400">{{ $t('filter.high') }}:</label>
              <input type="number" min="1" max="200" v-model.number="filter.highValue" @input="ensureLow(filter)" class="w-16 text-white p-1 rounded"/>
              <span class="text-sm text-gray-400">Hz</span>
            </div>
            <div class="mt-3 space-y-3">
              <div>
                <label class="text-xs text-gray-400">{{ $t('filter.lowCutoff') }}</label>
                <input type="range" min="1" max="200" v-model.number="filter.lowValue" @input="ensureHigh(filter)" class="w-full accent-blue-500"/>
              </div>
              <div>
                <label class="text-xs text-gray-400">{{ $t('filter.highCutoff') }}</label>
                <input type="range" min="1" max="200" v-model.number="filter.highValue" @input="ensureLow(filter)" class="w-full accent-blue-500"/>
              </div>
            </div>
          </template>
          <template v-else-if="filter.type==='notch'">
            <div class="flex items-center gap-2 mb-2">
              <label class="text-sm text-gray-400">{{ $t('filter.value') }}:</label>
              <input type="number" min="1" max="200" v-model.number="filter.value" class="w-20  text-white p-1 rounded"/>
              <span class="text-sm text-gray-400">Hz</span>
            </div>
            <div class="mt-3">
              <input type="range" min="1" max="200" v-model.number="filter.value" class="w-full accent-blue-500"/>
              <div class="flex justify-between text-xs text-gray-400 mt-1">
                <span>1 Hz</span>
                <span>200 Hz</span>
              </div>
            </div>
          </template>
        </div>
      </transition-group>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';

// Initialize i18n
const { t } = useI18n();

const filterTypes = [
  { value: 'lowpass', label: t('filter.lowPass') },
  { value: 'highpass', label: t('filter.highPass') },
  { value: 'bandpass', label: t('filter.bandPass') },
  { value: 'notch', label: t('filter.notchPass') }
];

function createFilter(type) {
  const id = Date.now() + Math.random();
  switch (type) {
    case 'lowpass':
    case 'highpass':
      return { id, type, active: false, value: 50 };
    case 'bandpass':
      return { id, type, active: false, lowValue: 20, highValue: 100 };
    case 'notch':
      return { id, type, active: false, value: 50 };
    default:
      return { id, type, active: false };
  }
}

const filters = ref([]);
const newFilterType = ref(filterTypes[0].value);

function addFilter() {
  filters.value.unshift(createFilter(newFilterType.value));
}

function toggleActive(filter) {
  filter.active = !filter.active;
}

function ensureHigh(filter) {
  if (filter.highValue <= filter.lowValue) {
    filter.highValue = filter.lowValue + 1;
  }
}

// Ensure low cutoff always below high cutoff
function ensureLow(filter) {
  if (filter.lowValue >= filter.highValue) {
    filter.lowValue = filter.highValue - 1;
  }
}

function displayName(type) {
  const f = filterTypes.find(f => f.value === type);
  return f ? f.label : type;
}

function saveConfig() {
  console.log('Filters:', filters.value);
}

// Remove a filter card
function deleteFilter(filter) {
  filters.value = filters.value.filter(f => f.id !== filter.id);
}
</script>

<style scoped>
/* Filter card add/delete animations */
.filter-enter-active, .filter-leave-active {
  transition: all 200ms ease-in-out;
}
.filter-enter-from, .filter-leave-to {
  opacity: 0;
  transform: scale(0.9);
}
.filter-enter-to, .filter-leave-from {
  opacity: 1;
  transform: scale(1);
}
</style>
