<template>
  <div id="main" class="flex flex-col h-screen w-screen overflow-hidden">
    <div
      class="bg-gradient-to-br from-gray-900 to-gray-800 px-6 rounded-lg shadow-2xl max-[800px]:px-0 font-sans text-white flex flex-col flex-grow overflow-hidden" style="border-radius: 0.5rem;">
      <AppHeader />
      <div class="flex gap-8 flex-grow h-full overflow-hidden max-[800px]:flex-col">
        <!-- Sidebar Component -->
        <AppSidebar v-model:activeView="primaryView" v-model:additionalViews="additionalViews" v-model:collapsed="collapsed" :class="[
            collapsed ? 'w-16 px-2 py-5' : 'w-60 p-5',
            'mb-5 border border-gray-700 shadow-inner rounded-lg overflow-y-auto',
            'flex flex-col flex-shrink-0 bg-gray-800 bg-opacity-60',
            'max-[800px]:fixed max-[800px]:bottom-0 max-[800px]:left-0',
            'max-[800px]:flex-row max-[800px]:items-center max-[800px]:z-50',
            'max-[800px]:shadow-[0_-4px_6px_0_rgba(0,0,0,0.1)] max-[800px]:mb-0',
            'max-[800px]:w-full max-[800px]:px-8 max-[800px]:py-4'
          ]" />
        <div class="flex-grow overflow-y-auto space-y-6 h-full pb-4 max-[800px]:pb-16 max-[800px]:mx-2" style="min-height: 0;">
          <!-- Primary View -->
          <VisualizationView v-if="primaryView === 'visualization'" class="view"/>
          <SignalConfigView v-if="primaryView === 'signal'" class="view" :selected-data-source="selectedDataSource" :serial-settings="serialSettings" :tcp-settings="tcpSettings" :fake-data-settings="fakeDataSettings" @data-source-changed="onDataSourceChanged" />
          <FilterConfigView v-if="primaryView === 'filters'" class="view" />
          <RecordingView v-if="primaryView === 'folder'" class="view" />
          <StreamingView v-if="primaryView === 'streaming'" class="view" />
          <!-- Additional Views (only in visualization) -->
          <StreamingView v-if="primaryView === 'visualization' && additionalViews.includes('streaming')" class="view" />
          <FilterConfigView v-if="primaryView === 'visualization' && additionalViews.includes('filters')" class="view" />
          <RecordingView v-if="primaryView === 'visualization' && additionalViews.includes('folder')" class="view" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// Add style to document body directly
document.body.style.margin = '0';
document.body.style.padding = '0';
// document.body.style.overflow = 'hidden';
document.documentElement.style.margin = '0';
document.documentElement.style.padding = '0';

// Add meta viewport tag to disable zooming
const metaViewport = document.createElement('meta');
metaViewport.name = 'viewport';
metaViewport.content = 'width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no';
document.head.appendChild(metaViewport);

// Prevent pinch zoom
document.addEventListener('touchmove', function(event) {
  if (event.touches.length > 1) {
    event.preventDefault();
  }
}, { passive: false });
// Import the Window API from Tauri
import { onMounted, ref } from 'vue';
import { serialSettings, tcpSettings, fakeDataSettings } from './store/appState';

// Import components
import AppSidebar from './components/AppSidebar.vue';
import VisualizationView from './components/views/VisualizationView.vue';
import SignalConfigView from './components/views/SignalConfigView.vue';
import FilterConfigView from './components/views/FilterConfigView.vue';
import RecordingView from './components/views/RecordingView.vue';
import StreamingView from './components/views/StreamingView.vue';
import AppHeader from './components/AppHeader.vue';
const primaryView = ref('visualization'); // Options: 'visualization', 'signal', 'filters', 'folder', 'streaming'
const additionalViews = ref<string[]>([]);
const collapsed = ref(false);

// Signal source state
const selectedDataSource = ref('fake'); // Options: 'serial', 'tcp', 'fake'

// Serial settings now defined in central state (appState.ts)

// Function to handle data source change
function onDataSourceChanged(source: 'serial' | 'tcp' | 'fake') {
  selectedDataSource.value = source;
  console.log(`Data source changed to: ${source}`);
}

onMounted(() => {
  console.log("Mounted! Checking DOM...");
  // Apply styles to ensure no margins
  const mainContainer = document.getElementById('main');
  if (mainContainer) {
    Object.assign(mainContainer.style, {
      margin: '0',
      padding: '0',
      width: '100vw',
      height: '100vh',
      touchAction: 'pan-x pan-y' // Allows scrolling but prevents zooming
    });
  }
  
  // Additional touch event handler for the main container
  mainContainer?.addEventListener('touchstart', (e) => {
    if (e.touches.length > 1) {
      e.preventDefault();
    }
  }, { passive: false });
});
</script>

<style scoped>
#main {
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
  user-select: none;
}

</style>

<style>
/* Transparent scrollbar for right content */
#main .overflow-y-auto::-webkit-scrollbar { width: 5px; height: 5px; }
#main .overflow-y-auto::-webkit-scrollbar-track { background: transparent; }
#main .overflow-y-auto::-webkit-scrollbar-thumb { background-color: rgba(255,255,255,0.2); border-radius: 4px; }
#main .overflow-y-auto::-webkit-scrollbar-thumb:hover { background-color: rgba(255,255,255,0.3); }
</style>
