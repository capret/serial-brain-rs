<template>
  <div id="main" class="flex flex-col h-screen w-screen overflow-hidden">
    <div
      class="bg-gradient-to-br from-gray-900 to-gray-800 rounded-lg shadow-2xl p-8 font-sans text-white flex flex-col flex-grow overflow-hidden" style="border-radius: 0.5rem;">
      <header class="flex justify-between items-center mb-8" id="titlebar">
        <h1 class="text-3xl font-bold tracking-tight">
          Serial<span class="text-blue-400">Brain</span>
        </h1>
        <div class="flex items-center gap-3 z-10">
          <button class="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-md 
                flex items-center gap-2 transition-all duration-300 transform hover:scale-105">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
              stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="feather feather-refresh-cw">
              <polyline points="23 4 23 10 17 10"></polyline>
              <polyline points="1 20 1 14 7 14"></polyline>
              <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
            </svg>
            Refresh
          </button>
          <button
            class="bg-gray-700 hover:bg-gray-600 px-4 py-2 rounded-md flex items-center gap-2 transition-all duration-300 transform hover:scale-105">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
              stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="feather feather-settings">
              <circle cx="12" cy="12" r="3"></circle>
              <path
                d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z">
              </path>
            </svg>
            Settings
          </button>
        </div>
        <div>
          <div class="flex items-center gap-2 ml-2">
            <button id="titlebar-minimize"
              class="titlebar-button text-gray-400 hover:text-white p-1 transition-colors duration-200">
              <svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 24 24" fill="none"
                stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="5" y1="12" x2="19" y2="12"></line>
              </svg>
            </button>
            <button id="titlebar-maximize"
              class="titlebar-button text-gray-400 hover:text-white p-1 transition-colors duration-200">
              <svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 24 24" fill="none"
                stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
              </svg>
            </button>
            <button id="titlebar-close"
              class="titlebar-button text-gray-400 hover:text-white p-1 transition-colors duration-200">
              <svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 24 24" fill="none"
                stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </button>
          </div>
        </div>
      </header>
      <div class="flex gap-8 flex-grow h-full overflow-hidden">
        <!-- Sidebar Component -->
        <AppSidebar v-model:activeView="activeView" />
        <div class="flex-grow space-y-6 overflow-y-auto h-full pr-2 max-h-full pb-4" style="min-height: 0;">
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
import AppSidebar from './components/sidebar/AppSidebar.vue';
import VisualizationView from './components/views/VisualizationView.vue';
import SignalConfigView from './components/views/SignalConfigView.vue';
import FilterConfigView from './components/views/FilterConfigView.vue';
import RecordingView from './components/views/RecordingView.vue';

const appWindow = getCurrentWindow();

// State for tracking which content is shown in the right panel
const activeView = ref('visualization'); // Options: 'visualization', 'signal', 'filters', 'folder'

// Signal source state
const selectedDataSource = ref('fake'); // Options: 'serial', 'tcp', 'fake'

// Settings for different signal sources
const serialSettings = ref({
  port: '',
  baudRate: 115200,
  stopBits: 1,
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

  // Set up the window controls
  document.getElementById('titlebar-minimize')?.addEventListener('click', () => {
    console.log('Minimize clicked');
    appWindow.minimize();
  });

  document.getElementById('titlebar-maximize')?.addEventListener('click', () => {
    console.log('Maximize clicked');
    appWindow.toggleMaximize();
  });

  document.getElementById('titlebar-close')?.addEventListener('click', () => {
    console.log('Close clicked');
    appWindow.close();
  });

  document.getElementById('titlebar')?.addEventListener('mousedown', (e) => {
    // If the target is a button, exit without dragging.
    if (e.target.closest('button')) return;

    if (e.buttons === 1) {
      if (e.detail === 2) {
        console.log('Toggle maximize clicked');
        appWindow.toggleMaximize();
      } else {
        console.log('Dragging');
        appWindow.startDragging();
      }
    }
  });
});
</script>

<style scoped>
#main {
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
  user-select: none;
}

/* Custom select styling */
.custom-select {
  color: white;
}

/* Cross-browser styling for the select element */
.custom-select {
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  text-indent: 0.01px;
  text-overflow: '';
}

/* Custom select hover effects */
.custom-select:hover {
  background-color: rgb(45, 55, 72) !important;
}
</style>

