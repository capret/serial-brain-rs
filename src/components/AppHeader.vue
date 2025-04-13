<template>
  <header class="header-container" id="titlebar">
    <div class="title-area">
      <h1 class="text-2xl font-bold">Signal Realtime Plot</h1>
      <p class="text-sm opacity-80">Monitor and analyze multi-channel signals in real-time</p>
    </div>
    <div class="window-controls">
      <button id="titlebar-minimize" class="control-btn minimize" title="Minimize">
        <span>—</span>
      </button>
      <button id="titlebar-maximize" class="control-btn maximize" title="Maximize">
        <span>▢</span>
      </button>
      <button id="titlebar-close" class="control-btn close" title="Close">
        <span>✕</span>
      </button>
    </div>
  </header>
</template>

<script setup>
// Import the Window API from Tauri
import { getCurrentWindow } from '@tauri-apps/api/window';
import { onMounted } from 'vue';
const appWindow = getCurrentWindow();
// Use onMounted to set up the event listeners once the DOM is ready
onMounted(() => {
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
    if (e.buttons === 1 && !e.target.closest('.window-controls')) {
      if (e.detail === 2) {
        appWindow.toggleMaximize();
      } else {
        appWindow.startDragging();
      }
    }
  });
});
</script>

<style scoped>
.header-container {
  top: 0;
  left: 0;
  right: 0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 0 1rem 1rem;
  background-image: linear-gradient(to right, rgb(99 27 255), rgb(70 7 208));
  color: white;
  position: relative;
  height: 150px;
  box-sizing: border-box;
  user-select: none;
  -webkit-app-region: drag;
}

.title-area {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  max-width: calc(100% - 140px); /* Leave space for window controls */
}

.text-2xl {
  font-size: 1.6rem;
  line-height: 2.1rem;
  font-weight: 700;
}

.text-sm {
  font-size: 0.95rem;
  line-height: 1.35rem;
}

.opacity-80 {
  opacity: 0.8;
}

.font-bold {
  font-weight: 700;
}

.window-controls {
  display: flex;
  position: absolute;
  top: 0;
  right: 0;
  -webkit-app-region: no-drag;
}

.control-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 46px;
  height: 46px;
  padding: 0;
  border: none;
  background-color: transparent;
  color: white;
  font-size: 20px;
  line-height: 1;
  cursor: pointer;
  -webkit-app-region: no-drag;
}

.minimize span {
  margin-top: -8px;
  font-weight: bold;
}

.maximize span {
  font-size: 16px;
}

.close span {
  font-size: 24px;
}

.minimize:hover {
  background-color: rgba(255, 255, 255, 0.15);
}

.maximize:hover {
  background-color: rgba(255, 255, 255, 0.15);
}

.close:hover {
  background-color: #E81123;
}
</style>
