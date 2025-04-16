<template>
  <div id="main" class="flex flex-col h-screen w-screen">
    <div
      class="bg-gradient-to-br from-gray-900 to-gray-800 rounded-lg shadow-2xl p-8 font-sans text-white flex flex-col flex-grow">
      <header class="flex justify-between items-center mb-8" id="titlebar">
        <h1 class="text-3xl font-bold tracking-tight">
          Serial<span class="text-blue-400">Brain</span>
        </h1>
        <div class="flex items-center gap-3 z-10">
          <button
            class="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-md 
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
      <div class="grid grid-cols-3 gap-8 flex-grow">
        <div class="col-span-1 bg-gray-800 bg-opacity-60 rounded-lg p-5 overflow-y-auto">
          <h2 class="text-xl font-semibold mb-4">Available Signal</h2>
          <div class="space-y-3">
            <div
              class="bg-blue-600 bg-opacity-20 border-l-4 border-blue-500 p-4 rounded-r-lg cursor-pointer hover:bg-blue-600 hover:bg-opacity-30 transition-all duration-300 transform hover:translate-x-1">
              <h3 class="font-bold">Fake Reader</h3>
              <p class="text-sm text-gray-300">Signal Simulation</p>
            </div>
            <div
              class="bg-gray-700 p-4 rounded-lg cursor-pointer hover:bg-gray-600 transition-all duration-300 transform hover:translate-x-1">
              <h3 class="font-bold">Serial Reader</h3>
              <p class="text-sm text-gray-300">USB Connection</p>
            </div>
            <div
              class="bg-gray-700 p-4 rounded-lg cursor-pointer hover:bg-gray-600 transition-all duration-300 transform hover:translate-x-1">
              <h3 class="font-bold">Socket Reader</h3>
              <p class="text-sm text-gray-300">TCP Connection</p>
            </div>
          </div>
        </div>
        <div class="col-span-2 space-y-6">
          <div class="bg-gray-800 bg-opacity-60 rounded-lg p-6">
            <div class="flex justify-between items-start mb-6">
              <div>
                <h2 class="text-3xl font-bold text-blue-400">Visualization</h2>
              </div>
              <div class="flex gap-3">
                <button
                  class="bg-blue-600 hover:bg-blue-700 px-6 py-3 rounded-md font-semibold flex items-center gap-2 transition-all duration-300 transform hover:scale-105 shadow-lg">
                  <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24"
                    fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                    class="feather feather-play">
                    <polygon points="5 3 19 12 5 21 5 3"></polygon>
                  </svg>
                  Launch
                </button>
                <details class="relative inline-block">
                  <summary
                    class="bg-gray-700 hover:bg-gray-600 px-3 py-3 rounded-md cursor-pointer transition-all duration-300 list-none flex items-center justify-center">
                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24"
                      fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                      class="feather feather-more-vertical">
                      <circle cx="12" cy="12" r="1"></circle>
                      <circle cx="12" cy="5" r="1"></circle>
                      <circle cx="12" cy="19" r="1"></circle>
                    </svg>
                  </summary>
                  <div class="absolute right-0 mt-2 w-48 bg-gray-700 rounded-md shadow-lg z-20 py-1">
                    <a href="#" class="block px-4 py-2 hover:bg-gray-600 transition-colors duration-200">
                      Update
                    </a>
                    <a href="#" class="block px-4 py-2 hover:bg-gray-600 transition-colors duration-200">
                      Uninstall
                    </a>
                    <a href="#" class="block px-4 py-2 hover:bg-gray-600 transition-colors duration-200">
                      Report Issue
                    </a>
                  </div>
                </details>
              </div>
            </div>
            <div class="mb-6">
              <h3 class="text-lg font-semibold mb-2">Display Window:</h3>
              <!-- <p class="text-gray-300">Plotting Area</p> -->
              <SignalVisualization />
            </div>
          </div>
          <div class="bg-gray-800 bg-opacity-60 rounded-lg p-6">
            <div class="flex justify-between items-center mb-4">
              <h3 class="text-xl font-semibold">Filter Configuration</h3>
              <button
                class="bg-gray-700 hover:bg-gray-600 px-3 py-2 rounded-md text-sm flex items-center gap-2 transition-all duration-300 transform hover:scale-105">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24"
                  fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                  class="feather feather-plus">
                  <line x1="12" y1="5" x2="12" y2="19"></line>
                  <line x1="5" y1="12" x2="19" y2="12"></line>
                </svg>
                Add Filter
              </button>
            </div>
            <div class="grid grid-cols-2 gap-4">
              <div
                class="bg-gray-700 hover:bg-gray-600 p-4 rounded-lg transition-all duration-300 transform hover:scale-[1.02] cursor-pointer">
                <div class="flex justify-between items-start mb-2">
                  <h4 class="font-medium">Low Pass</h4>
                  <span class="text-xs bg-blue-500 px-2 py-1 rounded">Active</span>
                </div>
                <p class="text-sm text-gray-400">Value: 50 Hz</p>
              </div>
              <div
                class="bg-gray-700 hover:bg-gray-600 p-4 rounded-lg transition-all duration-300 transform hover:scale-[1.02] cursor-pointer">
                <div class="flex justify-between items-start mb-2">
                  <h4 class="font-medium">Band Pass</h4>
                  <span class="text-xs bg-gray-500 px-2 py-1 rounded">Not Active</span>
                </div>
                <p class="text-sm text-gray-400">Low: 20 Hz High: 100 Hz</p>
              </div>
            </div>
          </div>
          <div class="flex flex-col gap-4">
            <div class="bg-gray-800 bg-opacity-60 rounded-lg p-6">
              <div class="space-y-3">
                <div class="mt-6">
                  <div class="flex items-center justify-between mb-2">
                    <p class="text-sm">Select folder to record data</p>
                    <span class="text-xs text-gray-400">Required</span>
                  </div>
                  <div class="flex">
                    <input type="text" placeholder="No file selected" readonly=""
                      class="bg-gray-700 px-3 py-2 rounded-l-md flex-grow text-gray-300" />
                    <button
                      class="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-r-md transition-colors duration-300">
                      Browse
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
// Add style to document body directly
document.body.style.margin = '0';
document.body.style.padding = '0';
document.body.style.overflow = 'hidden';
document.documentElement.style.margin = '0';
document.documentElement.style.padding = '0';
// Import the Window API from Tauri
import { getCurrentWindow } from '@tauri-apps/api/window';
import { onMounted } from 'vue';
// import AppHeader from './components/AppHeader.vue';
// import Sidebar from './components/sidebar/Sidebar.vue';
import SignalVisualization from './components/visualization/SignalVisualization.vue';
const appWindow = getCurrentWindow();

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
.main {
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
  user-select: none;
}
</style>
