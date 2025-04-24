<template>
  <div id="main" class="flex flex-col h-screen w-screen overflow-hidden">
    <div
      class="bg-gradient-to-br from-gray-900 to-gray-800 px-6 rounded-lg shadow-2xl max-[800px]:px-0 font-sans text-white flex flex-col flex-grow overflow-hidden" style="border-radius: 0.5rem;">
      <AppHeader />
      <div class="flex gap-8 flex-grow h-full overflow-hidden max-[800px]:flex-col">
        <!-- Sidebar Component -->
        <AppSidebar v-model:activeView="primaryView" v-model:additionalViews="additionalViews" :class="[
            collapsed ? 'w-16 px-2 py-5' : 'w-60 p-5',
            'mb-5 max-[800px]:mb-0',
            'border border-gray-700',
            'shadow-inner',
            'flex flex-col flex-shrink-0 bg-gray-800 bg-opacity-60 rounded-lg overflow-y-auto',
            'max-[800px]:fixed max-[800px]:bottom-0 max-[800px]:left-0',
            'max-[800px]:flex-row max-[800px]:items-center max-[800px]:z-50',
            'max-[800px]:shadow-[0_-4px_6px_0_rgba(0,0,0,0.1)]',
            'max-[800px]:w-full max-[800px]:px-8 max-[800px]:py-4'
          ]" />
        <div class="flex-grow overflow-y-auto space-y-6 h-full mb-5 pb-4 max-[800px]:pb-16" style="min-height: 0;">
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

<script setup>
// Add style to document body directly
document.body.style.margin = '0';
document.body.style.padding = '0';
// document.body.style.overflow = 'hidden';
document.documentElement.style.margin = '0';
document.documentElement.style.padding = '0';
// Import the Window API from Tauri
import { getCurrentWindow } from '@tauri-apps/api/window';
import { onMounted, ref } from 'vue';
// Import components
import AppSidebar from './components/AppSidebar.vue';
import VisualizationView from './components/views/VisualizationView.vue';
import SignalConfigView from './components/views/SignalConfigView.vue';
import FilterConfigView from './components/views/FilterConfigView.vue';
import RecordingView from './components/views/RecordingView.vue';
import StreamingView from './components/views/StreamingView.vue';
import AppHeader from './components/AppHeader.vue';


// State for primary view and appended additional views (in visualization)
const primaryView = ref('visualization'); // Options: 'visualization', 'signal', 'filters', 'folder', 'streaming'
const additionalViews = ref([]); // list of appended views when in visualization

// Signal source state
const selectedDataSource = ref('fake'); // Options: 'serial', 'tcp', 'fake'

// Settings for different signal sources
const serialSettings = ref({
  port: '',
  baudRate: 460800,
  stopBits: 1,
  parity: 'none',
  dataBits: 8,
  serialInfo: '',      // store serial text info
  serialInfoBuffer: [], // buffer for recent serial info lines
  availablePorts: [],
  sendData: '',
  sendEndFlag: '\n'
});

const tcpSettings = ref({
  host: 'localhost',
  port: 1234,
  protocol: 'tcp'
});

const fakeDataSettings = ref({
  minValue: -100,
  maxValue: 100,
  frequency: 500,
  channelCount: 8,
  waveform: 'sine'
});

// Function to handle data source change
function onDataSourceChanged(source) {
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
      height: '100vh'
    });
  }
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
