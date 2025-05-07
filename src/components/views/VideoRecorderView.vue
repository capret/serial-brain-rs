<template>
  <div class="flex flex-col h-full">
    <div class="p-4 bg-zinc-900">
      <h1 class="text-xl font-bold text-white">Video Recorder</h1>
      <p class="text-sm text-gray-400">Record video using the Tauri backend</p>
    </div>

    <div class="flex-grow p-4 flex flex-col">
      <!-- Video preview -->
      <div class="relative bg-black rounded-lg overflow-hidden mb-4 flex-grow">
        <canvas 
          ref="videoCanvas" 
          width="320" 
          height="240" 
          class="w-full h-full object-contain"
        ></canvas>
        
        <!-- Recording indicator -->
        <div v-if="isRecording" class="absolute top-4 right-4 flex items-center">
          <div class="h-3 w-3 rounded-full bg-red-600 animate-pulse mr-2"></div>
          <span class="text-white text-sm font-medium">Recording</span>
        </div>
      </div>

      <!-- Controls -->
      <div class="flex justify-between items-center">
        <div class="flex space-x-2">
          <button
            @click="toggleCamera"
            class="px-4 py-2 rounded-md bg-blue-600 hover:bg-blue-700"
          >
            {{ cameraActive ? 'Stop Camera' : 'Start Camera' }}
          </button>
          
          <button
            v-if="cameraActive"
            @click="toggleRecording"
            :class="isRecording
              ? 'bg-red-600 hover:bg-red-700'
              : 'bg-emerald-600 hover:bg-emerald-700'"
            class="px-4 py-2 rounded-md"
          >
            {{ isRecording ? 'Stop Recording' : 'Start Recording' }}
          </button>
        </div>
        
        <div class="text-gray-300 text-sm">
          <div v-if="recordingStats.frames > 0">
            Frames: {{ recordingStats.frames }}
          </div>
          <div v-if="recordingStats.duration > 0">
            Duration: {{ (recordingStats.duration / 1000).toFixed(1) }}s
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';

// Constants
const VIDEO_WIDTH = 320;
const VIDEO_HEIGHT = 240;
const FRAME_RATE = 30; // frames per second

// Refs
const videoCanvas = ref<HTMLCanvasElement | null>(null);
const videoStream = ref<MediaStream | null>(null);
const videoContext = ref<CanvasRenderingContext2D | null>(null);

// State
const cameraActive = ref(false);
const isRecording = ref(false);
const recordingStats = ref({
  frames: 0,
  startTime: 0,
  duration: 0
});

// Animation frame ID for cleanup
let animationFrameId: number | null = null;
// Interval ID for sending frames
let frameInterval: number | null = null;

// Camera functions
async function startCamera() {
  try {
    const stream = await navigator.mediaDevices.getUserMedia({
      video: {
        width: { ideal: VIDEO_WIDTH },
        height: { ideal: VIDEO_HEIGHT }
      },
      audio: false
    });
    
    videoStream.value = stream;
    cameraActive.value = true;
    
    // Start rendering the camera feed on canvas
    startCanvasRender();
  } catch (err) {
    console.error('Error accessing camera:', err);
    alert('Failed to access camera. Please check permissions.');
  }
}

function stopCamera() {
  if (videoStream.value) {
    videoStream.value.getTracks().forEach(track => track.stop());
    videoStream.value = null;
    cameraActive.value = false;
    
    // Stop rendering
    if (animationFrameId !== null) {
      cancelAnimationFrame(animationFrameId);
      animationFrameId = null;
    }
    
    // Clear canvas
    if (videoCanvas.value && videoContext.value) {
      videoContext.value.clearRect(0, 0, videoCanvas.value.width, videoCanvas.value.height);
    }
  }
}

function toggleCamera() {
  if (cameraActive.value) {
    stopCamera();
  } else {
    startCamera();
  }
}

// Rendering functions
function startCanvasRender() {
  if (!videoCanvas.value) return;
  
  // Get the context
  videoContext.value = videoCanvas.value.getContext('2d');
  if (!videoContext.value) return;
  
  // Function to render video frame on canvas
  const renderFrame = () => {
    if (videoStream.value && videoContext.value && videoCanvas.value) {
      // Find the first video track
      const videoTrack = videoStream.value.getVideoTracks()[0];
      if (!videoTrack) return;
      
      // Create video element dynamically
      const videoEl = document.createElement('video');
      videoEl.srcObject = videoStream.value;
      videoEl.play().catch(err => console.error('Error playing video:', err));
      
      videoEl.onloadedmetadata = () => {
        // Draw video frame on canvas
        videoContext.value!.drawImage(videoEl, 0, 0, videoCanvas.value!.width, videoCanvas.value!.height);
        
        // Request next frame
        animationFrameId = requestAnimationFrame(renderFrame);
      };
    }
  };
  
  // Start the render loop
  renderFrame();
}

// Recording functions
async function startRecording() {
  try {
    // Ask user where to save the file
    const savePath = await save({
      filters: [{
        name: 'Video',
        extensions: ['mp4']
      }]
    });
    
    if (!savePath) return; // User cancelled
    
    // Reset stats
    recordingStats.value = {
      frames: 0,
      startTime: Date.now(),
      duration: 0
    };
    
    // Start recording in the backend
    const result = await invoke('record_video_stream', { filePath: savePath });
    if (!result) {
      throw new Error('Failed to start recording');
    }
    
    isRecording.value = true;
    
    // Start sending frames
    frameInterval = window.setInterval(captureAndSendFrame, 1000 / FRAME_RATE);
    
  } catch (err) {
    console.error('Error starting recording:', err);
    alert(`Failed to start recording: ${err}`);
  }
}

async function stopRecording() {
  try {
    if (frameInterval !== null) {
      clearInterval(frameInterval);
      frameInterval = null;
    }
    
    const result = await invoke('stop_video_recording');
    console.log('Recording stopped:', result);
    
    isRecording.value = false;
    recordingStats.value.duration = Date.now() - recordingStats.value.startTime;
    
  } catch (err) {
    console.error('Error stopping recording:', err);
    alert(`Failed to stop recording: ${err}`);
  }
}

function toggleRecording() {
  if (isRecording.value) {
    stopRecording();
  } else {
    startRecording();
  }
}

// Function to capture and send a video frame
async function captureAndSendFrame() {
  if (!videoCanvas.value || !isRecording.value) return;
  
  try {
    // Convert canvas to base64 PNG
    const frameData = videoCanvas.value.toDataURL('image/png').split(',')[1];
    
    // Send frame to backend
    await invoke('push_video_frame', { frameData });
    
    // Update stats
    recordingStats.value.frames++;
    recordingStats.value.duration = Date.now() - recordingStats.value.startTime;
    
  } catch (err) {
    console.error('Error sending frame:', err);
  }
}

// Lifecycle hooks
onMounted(() => {
  // Initialize canvas
  if (videoCanvas.value) {
    videoCanvas.value.width = VIDEO_WIDTH;
    videoCanvas.value.height = VIDEO_HEIGHT;
  }
});

onBeforeUnmount(() => {
  // Clean up
  if (isRecording.value) {
    stopRecording();
  }
  
  if (cameraActive.value) {
    stopCamera();
  }
  
  if (animationFrameId !== null) {
    cancelAnimationFrame(animationFrameId);
  }
  
  if (frameInterval !== null) {
    clearInterval(frameInterval);
  }
});
</script>
