<template>
  <div class="bg-gray-800 bg-opacity-60 rounded-lg p-6">
    <h2 class="text-3xl font-bold text-blue-400 mb-4">Streaming</h2>
    <div class="flex flex-wrap gap-6">
      <div class="flex flex-col items-center flex-1 min-w-[320px]">
        <canvas ref="canvas" :width="w" :height="h" class="border rounded-md"></canvas>
        <p class="mt-2 text-sm">Streaming random RGB @30 fps from Rust</p>
      </div>
      <div class="flex flex-col flex-1 min-w-[220px]">
        <label class="text-sm text-gray-300 mb-1">Stream Path</label>
        <input v-model="streamUrl" type="text" placeholder="MJPEG URL" class="bg-gray-700 text-sm p-2 rounded-md w-full" />
        <div class="mt-2 flex gap-2">
          <button @click="toggleStreaming" class="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-md">
            {{ isStreaming ? 'Pause' : 'Connect' }}
          </button>
          <button @click="toggleFake" class="bg-green-600 hover:bg-green-700 px-4 py-2 rounded-md">
            {{ fakeEnabled ? 'Disable Fake' : 'Enable Fake' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

const w = 320, h = 240;
const canvas = ref<HTMLCanvasElement>();
const ctx = ref<CanvasRenderingContext2D>();
let unlistenFn: any = null;
let errorUnlisten: () => void;
const streamUrl = ref('http://192.168.1.123:81/stream');
const isStreaming = ref(false);
const fakeEnabled = ref(false);

const toggleStreaming = () => {
  if (isStreaming.value) {
    invoke('stop_streaming')
      .then(() => { isStreaming.value = false; })
      .catch(console.error);
  } else {
    invoke('start_streaming', { path: streamUrl.value, fake: fakeEnabled.value })
      .then(() => { isStreaming.value = true; })
      .catch(console.error);
  }
};

const toggleFake = () => {
  if (isStreaming.value) {
    invoke('stop_streaming')
      .then(() => {
        fakeEnabled.value = !fakeEnabled.value;
        return invoke('start_streaming', { path: streamUrl.value, fake: fakeEnabled.value });
      })
      .catch(console.error);
  } else {
    fakeEnabled.value = !fakeEnabled.value;
  }
};

onMounted(async () => {
  ctx.value = canvas.value!.getContext('2d')!;
  unlistenFn = await listen<string>('frame', ({ payload }) => {
    const img = new Image();
    img.onload = () => ctx.value!.drawImage(img, 0, 0, w, h);
    img.src = 'data:image/png;base64,' + payload;
  });
  // handle backend stream errors
  errorUnlisten = await listen<string>('stream_error', ({ payload }) => {
    console.error('Stream error:', payload);
    isStreaming.value = false;
  });
});

onBeforeUnmount(() => {
  if (unlistenFn) {
    unlistenFn();
  }
  if (errorUnlisten) {
    errorUnlisten();
  }
});
</script>

<style scoped>
/* optional styling for streaming view */
</style>
