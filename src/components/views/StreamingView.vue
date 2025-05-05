<template>
  <div class="bg-gray-800 bg-opacity-60 rounded-lg p-6">
    <h2 class="text-3xl font-bold text-blue-400 mb-4">Camera Streaming</h2>

    <div class="flex flex-wrap gap-6">
      <!-- ■■■ Preview panel ■■■ -->
      <div class="flex flex-col items-center flex-1 min-w-[320px]">
        <div class="relative">
          <!-- full‑res canvas shrunk with pure CSS -->
          <canvas
            ref="canvas"
            :width="SRC_W"
            :height="SRC_H"
            class="border rounded-md"
            style="width: 320px; height: 240px"
          ></canvas>

          <!-- brightness indicator -->
          <div
            class="absolute top-2 left-2 w-6 h-6 rounded-full border-2 border-white shadow-sm flex items-center justify-center"
            :class="isBrightEnough ? 'bg-green-500' : 'bg-red-500'"
            :title="isBrightEnough ? 'Average brightness ≥ 80' : 'Average brightness < 80'"
          >
            <span class="text-xs text-white font-bold">
              {{ isBrightEnough ? 'OK' : 'LO' }}
            </span>
          </div>
        </div>
        <p class="mt-2 text-sm">Streaming RGB @ 30 fps from Rust</p>
      </div>

      <!-- ■■■ Controls ■■■ -->
      <div class="flex flex-col flex-1 min-w-[220px]">
        <label class="text-sm text-gray-300 mb-1">Stream Path</label>
        <input
          v-model="streamUrl"
          type="text"
          placeholder="MJPEG URL"
          class="bg-gray-700 text-sm p-2 rounded-md w-full"
        />

        <div class="mt-2 flex flex-wrap gap-2">
          <button
            @click="toggleStreaming"
            class="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-md"
          >
            {{ isStreaming ? 'Pause' : 'Connect' }}
          </button>

          <button
            @click="toggleFake"
            class="bg-green-600 hover:bg-green-700 px-4 py-2 rounded-md"
          >
            {{ fakeEnabled ? 'Disable Fake' : 'Enable Fake' }}
          </button>

          <button
            @click="toggleRecording"
            :class="isRecording
              ? 'bg-red-600 hover:bg-red-700'
              : 'bg-yellow-600 hover:bg-yellow-700'"
            class="px-4 py-2 rounded-md"
          >
            {{ isRecording ? 'Stop Recording' : 'Record Video Only' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/* -------------------------------------------------------------------
   imports
------------------------------------------------------------------- */
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { writeFile, BaseDirectory } from '@tauri-apps/plugin-fs';

/* -------------------------------------------------------------------
   constants
------------------------------------------------------------------- */
const SRC_W = 640;           // incoming frame width
const SRC_H = 480;           // incoming frame height
const FPS   = 30;

/* -------------------------------------------------------------------
   reactive state
------------------------------------------------------------------- */
const streamUrl      = ref('http://192.168.1.123:81/stream');
const isStreaming    = ref(false);
const fakeEnabled    = ref(false);
const isBrightEnough = ref(true);

const isRecording    = ref(false);

/* -------------------------------------------------------------------
   canvas / contexts
------------------------------------------------------------------- */
// preview canvas (visible)
const canvas = ref<HTMLCanvasElement | null>(null);
let   prevCtx: CanvasRenderingContext2D;

// off‑screen canvas for high‑res recording
const recCanvas = document.createElement('canvas');
recCanvas.width  = SRC_W;
recCanvas.height = SRC_H;
const recCtx = recCanvas.getContext('2d')!;

/* -------------------------------------------------------------------
   MediaRecorder
------------------------------------------------------------------- */
let mediaRecorder: MediaRecorder | null = null;
let recordedChunks: Blob[] = [];

/* -------------------------------------------------------------------
   event unlisten handles
------------------------------------------------------------------- */
let frameUnlisten: () => void;
let errorUnlisten: () => void;
let analysisUnlisten: () => void;

/* ===================================================================
   streaming controls
=================================================================== */
function toggleStreaming() {
  if (isStreaming.value) {
    invoke('stop_streaming')
      .then(() => {
        isStreaming.value = false;
        if (isRecording.value) stopRecording();   // safety
      })
      .catch(console.error);
  } else {
    invoke('start_streaming', { path: streamUrl.value, fake: fakeEnabled.value })
      .then(() => { isStreaming.value = true; })
      .catch(console.error);
  }
}

function toggleFake() {
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
}

/* ===================================================================
   recording controls
=================================================================== */
function toggleRecording() {
  isRecording.value ? stopRecording() : startRecording();
}

function startRecording() {
  // prepare stream
  const stream = recCanvas.captureStream(FPS);

  // choose MIME
  let mime = 'video/mp4';
  if (!MediaRecorder.isTypeSupported(mime)) mime = 'video/webm';

  mediaRecorder  = new MediaRecorder(stream, { mimeType: mime });
  recordedChunks = [];
  isRecording.value = true;

  mediaRecorder.ondataavailable = (e: BlobEvent) => {
    if (e.data.size > 0) recordedChunks.push(e.data);
  };

  mediaRecorder.onstop = async () => {
    if (!recordedChunks.length) return;
    const blob   = new Blob(recordedChunks, { type: mime });
    const buffer = new Uint8Array(await blob.arrayBuffer());
    const ext    = mime.includes('mp4') ? 'mp4' : 'webm';
    const file   = `camera_recording_${Date.now()}.${ext}`;

    try {
      await writeFile(file, buffer, { baseDir: BaseDirectory.Document });
      console.log(`📁 Saved to Documents/${file}`);
    } catch (err) {
      console.error('Failed to save recording:', err);
    }
  };

  mediaRecorder.start();
}

function stopRecording() {
  if (mediaRecorder && mediaRecorder.state !== 'inactive') mediaRecorder.stop();
  isRecording.value = false;
}

/* ===================================================================
   lifecycle
=================================================================== */
onMounted(async () => {
  // preview context
  prevCtx = canvas.value!.getContext('2d')!;

  /* ---------- frame listener ---------- */
  frameUnlisten = await listen<string>('frame', ({ payload }) => {
    const img = new Image();
    img.onload = () => {
      /* draw full‑res frame to recorder canvas */
      recCtx.drawImage(img, 0, 0, SRC_W, SRC_H);

      /* draw the same frame to preview canvas (also full res; CSS shrinks) */
      prevCtx.drawImage(img, 0, 0, SRC_W, SRC_H);
    };
    img.src = `data:image/png;base64,${payload}`;
  });

  /* ---------- brightness listener ---------- */
  analysisUnlisten = await listen<boolean>('frame_analysis', ({ payload }) => {
    isBrightEnough.value = payload;
  });

  /* ---------- backend error listener ---------- */
  errorUnlisten = await listen<string>('stream_error', ({ payload }) => {
    console.error('Stream error:', payload);
    isStreaming.value = false;
    if (isRecording.value) stopRecording();
  });
});

onBeforeUnmount(() => {
  frameUnlisten?.();
  analysisUnlisten?.();
  errorUnlisten?.();
  if (isRecording.value) stopRecording();
});
</script>

<style scoped>
/* optional extra styling */
</style>
