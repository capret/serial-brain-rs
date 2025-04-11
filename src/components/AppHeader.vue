<template>
  <header class="header-container">
    <div class="title-area">
      <h1 class="text-2xl font-bold">Signal Realtime Plot</h1>
      <p class="text-sm opacity-80">Monitor and analyze multi-channel signals in real-time</p>
    </div>
    
    <div class="window-controls">
      <button @click="minimizeWindow" class="control-btn minimize" title="Minimize">
        <span>—</span>
      </button>
      <button @click="maximizeWindow" class="control-btn maximize" title="Maximize">
        <span>□</span>
      </button>
      <button @click="closeWindow" class="control-btn close" title="Close">
        <span>×</span>
      </button>
    </div>
  </header>
</template>

<script setup>
// Import the Window API from Tauri
import { getCurrentWindow } from '@tauri-apps/api/window';
const appWindow = getCurrentWindow();
const minimizeWindow = async () => {
  try {
    await appWindow.minimize();
  } catch (e) {
    console.error('Failed to minimize window:', e);
  }
};

const maximizeWindow = async () => {
  try {
    const isMaximized = await appWindow.isMaximized();
    if (isMaximized) {
      await appWindow.unmaximize();
    } else {
      await appWindow.maximize();
    }
  } catch (e) {
    console.error('Failed to toggle maximize window:', e);
  }
};

const closeWindow = async () => {
  try {
    await appWindow.close();
  } catch (e) {
    console.error('Failed to close window:', e);
  }
};
</script>

<style scoped>

.header-container {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem;
  background-image: linear-gradient(to right, rgb(99 27 255), rgb(70 7 208));
  color: white;
  position: relative;
  height: 150px;
  box-sizing: border-box;
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
  -webkit-app-region: no-drag; /* Make buttons clickable */
  z-index: 10;
}

.control-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 46px;
  height: 46px;
  border: none;
  cursor: pointer;
  font-size: 16px;
  transition: background-color 0.15s;
  color: white;
  background-color: transparent;
}

.minimize span {
  margin-top: -8px;
  font-weight: bold;
}

.maximize span {
  font-size: 18px;
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

.close span {
  font-size: 20px;
}
</style>
