<template>
  <header class="flex justify-between items-center py-8 px-6 max-[800px]:px-4" id="titlebar">
    <h1 class="text-3xl font-bold tracking-tight">
      Serial<span class="text-blue-400">Brain</span>
    </h1>
    <div class="flex items-center gap-4">
      <!-- User info and logout button -->
      <div v-if="user" class="flex items-center gap-3 mr-2">
        <div class="flex items-center gap-2">
          <div class="w-8 h-8 rounded-full bg-blue-500 flex items-center justify-center text-white font-semibold">
            {{ user.email.charAt(0).toUpperCase() }}
          </div>
          <span class="text-sm text-gray-300 max-[800px]:hidden">{{ user.email }}</span>
        </div>
        <button 
          @click="$emit('logout')" 
          class="text-gray-400 hover:text-white p-1 transition-colors duration-200 flex items-center gap-1"
          title="Logout"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"></path>
            <polyline points="16 17 21 12 16 7"></polyline>
            <line x1="21" y1="12" x2="9" y2="12"></line>
          </svg>
          <span class="max-[800px]:hidden">Logout</span>
        </button>
      </div>
      
      <!-- Window controls -->
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

<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window';
import { onMounted, defineProps, defineEmits } from 'vue';

// Define props and emits
defineProps({
  user: {
    type: Object as () => { email: string } | null,
    default: null
  }
});

defineEmits(['logout']);


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
    // Type assertion for TypeScript
    const target = e.target as HTMLElement;
    if (target.closest('button')) return;
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
