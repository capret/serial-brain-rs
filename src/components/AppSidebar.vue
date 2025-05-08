<template>
  <div>
    <button @click="collapsed = !collapsed"
      :class="['mb-4 text-white focus:outline-none block max-[800px]:hidden', collapsed ? 'mx-auto' : 'mx-0']">
      <svg xmlns="http://www.w3.org/2000/svg" class="w-6 h-6" fill="none" stroke="currentColor" stroke-width="2"
        stroke-linecap="round" stroke-linejoin="round">
        <path d="M4 6h16M4 12h16M4 18h16" />
      </svg>
    </button>

    <div
      class="space-y-3 max-[800px]:space-y-0 max-[800px]:flex max-[800px]:flex-row max-[800px]:space-x-3 max-[800px]:justify-center">
      <div @click="setActiveView('visualization')" :class="[`p-4 rounded-lg cursor-pointer transition-all duration-300 transform hover:translate-x-1`,
        activeView === 'visualization'
          ? 'bg-blue-600 bg-opacity-20 border-l-4 border-blue-500 rounded-r-lg'
          : 'bg-gray-700 hover:bg-gray-600']">
        <div class="flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"></polyline>
          </svg>
          <h3 v-if="!collapsed" class="font-bold max-[800px]:hidden">Visualization</h3>
        </div>
      </div>

      <div @click="onActionClick('filters')" :class="[`p-4 rounded-lg cursor-pointer transition-all duration-300 transform hover:translate-x-1`,
        (activeView === 'filters' || (activeView === 'visualization' && additionalViews.includes('filters')))
          ? 'bg-blue-600 bg-opacity-20 border-l-4 border-blue-500 rounded-r-lg'
          : 'bg-gray-700 hover:bg-gray-600']">
        <div class="flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"></polygon>
          </svg>
          <h3 v-if="!collapsed" class="font-bold max-[800px]:hidden">Filters</h3>
        </div>
      </div>

      <div @click="onActionClick('folder')" :class="[`p-4 rounded-lg cursor-pointer transition-all duration-300 transform hover:translate-x-1`,
        (activeView === 'folder' || (activeView === 'visualization' && additionalViews.includes('folder')))
          ? 'bg-blue-600 bg-opacity-20 border-l-4 border-blue-500 rounded-r-lg'
          : 'bg-gray-700 hover:bg-gray-600']">
        <div class="flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
          </svg>
          <h3 v-if="!collapsed" class="font-bold max-[800px]:hidden">Recording</h3>
        </div>
      </div>

      <div @click="onActionClick('streaming')" :class="[`p-4 rounded-lg cursor-pointer transition-all duration-300 transform hover:translate-x-1`,
        (activeView === 'streaming' || (activeView === 'visualization' && additionalViews.includes('streaming')))
          ? 'bg-blue-600 bg-opacity-20 border-l-4 border-blue-500 rounded-r-lg'
          : 'bg-gray-700 hover:bg-gray-600']">
        <div class="flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polygon points="23 7 16 12 23 17 23 7"></polygon>
            <rect x="1" y="5" width="15" height="14" rx="2" ry="2"></rect>
          </svg>
          <h3 v-if="!collapsed" class="font-bold max-[800px]:hidden">Streaming</h3>
        </div>
      </div>
      
    </div>
    <div class="mt-auto max-[800px]:mt-0 max-[800px]:flex max-[800px]:space-x-3 max-[800px]:ml-auto">
      <div @click="setActiveView('signal')" :class="[`p-4 rounded-lg cursor-pointer transition-all duration-300 transform hover:translate-x-1`,
        activeView === 'signal'
          ? 'bg-blue-600 bg-opacity-20 border-l-4 border-blue-500 rounded-r-lg'
          : 'bg-gray-700 hover:bg-gray-600']">
        <div class="flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
            class="feather feather-settings">
            <circle cx="12" cy="12" r="3"></circle>
            <path
              d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z">
            </path>
          </svg>
          <h3 v-if="!collapsed" class="font-bold max-[800px]:hidden">Settings</h3>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const props = defineProps({
  activeView: {
    type: String,
    required: true
  },
  additionalViews: {
    type: Array,
    default: () => []
  }
});

const emit = defineEmits(['update:activeView', 'update:additionalViews']);
const collapsed = ref(false);

function setActiveView(view: string) {
  emit('update:activeView', view);
}

function setAdditionalViews(view: string) {
  const list = props.additionalViews;
  const next = list.includes(view)
    ? list.filter(v => v !== view)
    : [...list, view];
  emit('update:additionalViews', next);
}

// Unified single-click toggle or navigate
function onActionClick(view: string) {
  if (props.activeView === 'visualization') {
    setAdditionalViews(view);
  } else {
    setActiveView(view);
  }
}
</script>
