<template>
  <div id="main" class="flex flex-col h-screen w-screen overflow-hidden">
    <div
      class="bg-gradient-to-br from-gray-900 to-gray-800 rounded-lg shadow-2xl p-8 max-[800px]:px-0 font-sans text-white flex flex-col flex-grow overflow-hidden" style="border-radius: 0.5rem;">
      <AppHeader />
      <div class="flex gap-8 flex-grow h-full overflow-hidden max-[800px]:flex-col">
        <!-- Sidebar Component -->
        <AppSidebar v-model:activeView="activeView" />
        <div class="flex-grow space-y-6 overflow-y-auto h-full  max-h-full pb-4" style="min-height: 0;">
          <!-- View Components -->
          <VisualizationView v-if="activeView === 'visualization'" />
          <SignalConfigView v-if="activeView === 'signal'" :selected-data-source="selectedDataSource"
            :serial-settings="serialSettings" :tcp-settings="tcpSettings" :fake-data-settings="fakeDataSettings"
            @data-source-changed="onDataSourceChanged" />
          <FilterConfigView v-if="activeView === 'filters'" />
          <RecordingView v-if="activeView === 'folder'" />
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
import AppHeader from './components/AppHeader.vue';


// State for tracking which content is shown in the right panel
const activeView = ref('visualization'); // Options: 'visualization', 'signal', 'filters', 'folder'

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
#main .overflow-y-auto::-webkit-scrollbar { width: 8px; height: 8px; }
#main .overflow-y-auto::-webkit-scrollbar-track { background: transparent; }
#main .overflow-y-auto::-webkit-scrollbar-thumb { background-color: rgba(255,255,255,0.2); border-radius: 4px; }
#main .overflow-y-auto::-webkit-scrollbar-thumb:hover { background-color: rgba(255,255,255,0.3); }
</style>
