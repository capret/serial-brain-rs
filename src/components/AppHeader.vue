<template>
  <header class="flex justify-between items-center py-8 px-6 max-[800px]:px-4" id="titlebar">
    <h1 class="text-3xl font-bold tracking-tight">
      Serial<span class="text-blue-400">Brain</span>
    </h1>
    <div>
      <div class="flex items-center gap-2 ml-2 max-[800px]:hidden">
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
</template>

<script setup>
import { getCurrentWindow } from '@tauri-apps/api/window';
import { onMounted } from 'vue';


onMounted(() => {
  const appWindow = getCurrentWindow();
  console.log("AppHeader mounted")
  document.getElementById('titlebar-minimize')?.addEventListener('click', () => {
    console.log("Minimize")
    appWindow.minimize();
  });
  document.getElementById('titlebar-maximize')?.addEventListener('click', () => {
    console.log("Maximize")
    appWindow.toggleMaximize();
  });
  document.getElementById('titlebar-close')?.addEventListener('click', () => {
    console.log("Close")
    appWindow.close();
  });
  document.getElementById('titlebar')?.addEventListener('mousedown', (e) => {
    if (e.target.closest('button')) return;
    if (e.buttons === 1) {
      if (e.detail === 2) {
        appWindow.toggleMaximize();
      } else {
        appWindow.startDragging();
      }
    }
  });
});
</script>
