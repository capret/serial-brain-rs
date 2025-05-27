<template>
  <div class="bg-gray-800 bg-opacity-60 rounded-lg p-6">
    <h2 class="text-3xl font-bold text-blue-400 mb-4">{{ $t('streaming.title') }}</h2>

    <div class="flex flex-wrap gap-6">
      <!-- ■■■ Preview panel ■■■ -->
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
            :title="isBrightEnough ? $t('streaming.brightnessHighTitle') : $t('streaming.brightnessLowTitle')"
          >
            <span class="text-xs text-white font-bold">
              {{ isBrightEnough ? $t('streaming.brightnessOK') : $t('streaming.brightnessLow') }}
            </span>
          </div>
        </div>
        <p class="mt-2 text-sm">{{ $t('streaming.status') }}</p>
        
        <!-- Connection controls -->
        <div class="mt-4 flex flex-wrap gap-2 justify-center w-full">
          <!-- Removed button, moved to right side panel -->

          <!-- Removed button, moving to right side panel -->

          <!-- Recording now handled directly on camera cards -->
        </div>
      </div>

      <!-- ■■■ Camera Selection ■■■ -->
      <div class="flex flex-col flex-1 min-w-[300px]">
        <!-- Header with title and discover button -->
        <div class="flex justify-between items-center mb-3">
          <h3 class="text-lg font-semibold text-blue-300">{{ $t('streaming.deviceList') }}</h3>
          <button 
            @click="discoverDevices" 
            class="bg-indigo-600 hover:bg-indigo-700 px-3 py-1 rounded-md text-sm flex items-center gap-1"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="11" cy="11" r="8"></circle>
              <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
            </svg>
            {{ $t('streaming.discover') }}
          </button>
        </div>
        
        <!-- Configuration options -->
        <div class="mb-4 bg-gray-700 p-3 rounded-lg space-y-3">
          <!-- Connect/streaming toggle switch -->
          <div class="flex items-center justify-between">
            <label class="inline-flex items-center cursor-pointer">
              <span class="mr-3 text-sm font-medium text-gray-300">{{ $t('streaming.streamingModeLabel') }}</span>
              <div class="relative">
                <input 
                  type="checkbox" 
                  v-model="isStreaming" 
                  @change="toggleStreaming"
                  class="sr-only peer"
                >
                <div class="w-11 h-6 bg-gray-600 rounded-full peer peer-focus:ring-2 peer-focus:ring-blue-300 peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-0.5 after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
              </div>
            </label>
            <span class="text-xs text-gray-400 italic">{{ isStreaming ? $t('streaming.active') : $t('streaming.inactive') }}</span>
          </div>
          
          <!-- Fake data toggle switch -->
          <div class="flex items-center justify-between">
            <label class="inline-flex items-center cursor-pointer">
              <span class="mr-3 text-sm font-medium text-gray-300">{{ $t('streaming.fakeModeLabel') }}</span>
              <div class="relative">
                <input 
                  type="checkbox" 
                  v-model="fakeEnabled" 
                  @change="toggleFake"
                  class="sr-only peer"
                >
                <div class="w-11 h-6 bg-gray-600 rounded-full peer peer-focus:ring-2 peer-focus:ring-blue-300 peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-0.5 after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-green-600"></div>
              </div>
            </label>
            <span class="text-xs text-gray-400 italic">{{ fakeEnabled ? $t('streaming.enabled') : $t('streaming.disabled') }}</span>
          </div>
        </div>
        
        <!-- Manual URL input (as fallback) -->
        <div class="mb-4">
          <label class="text-sm text-gray-300 mb-1 block">{{ $t('streaming.manualUrl') }}</label>
          <div class="flex gap-2">
            <input
              v-model="streamUrl"
              type="text"
              :placeholder="$t('streaming.placeholder')"
              class="bg-gray-700 text-sm p-2 rounded-md flex-1"
            />
            <button 
              @click="selectManualUrl" 
              class="bg-gray-600 hover:bg-gray-500 px-3 py-1 rounded-md"
              :class="{'bg-blue-600 hover:bg-blue-700': selectedDeviceId === 'manual'}"
            >
              {{ $t('streaming.select') }}
            </button>
          </div>
        </div>

        <!-- Discovery status message -->
        <div v-if="isDiscovering" class="flex items-center justify-center p-4 text-gray-300">
          <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          {{ $t('streaming.discovering') }}
        </div>
        
        <div v-else-if="discoveredDevices.length === 0" class="text-center p-4 text-gray-400">
          {{ $t('streaming.noDevices') }}
        </div>

        <!-- Device cards container -->  
        <div class="max-h-[400px] overflow-auto py-2">
          <!-- Simple list layout to prevent clipping -->  
          <ul class="space-y-3 list-none m-0 p-0">
            <!-- Each card as a list item with proper spacing -->  
            <li 
              v-for="device in filteredDevices" 
              :key="device.ip + device.port"
              class="p-1 transform-none"
            >
              <!-- Card with hover effect and proper padding to prevent clipping -->  
              <div
                class="bg-gray-700 hover:bg-gray-600 p-3 rounded-lg transition-colors duration-200"
                :class="{'border border-blue-500': isDeviceSelected(device)}"
              >
                <!-- Device card header with name -->
                <div 
                  @click="selectDevice(device)"
                  class="flex items-center gap-2 mb-1 cursor-pointer"
                >
                  <!-- Camera icon -->
                  <div class="text-blue-400">
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
                      stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"></path>
                      <circle cx="12" cy="13" r="4"></circle>
                    </svg>
                  </div>
                  
                  <!-- Device name -->
                  <h4 class="font-medium text-sm truncate" :title="device.name">
                    {{ device.name }}
                  </h4>
                </div>
                
                <!-- Device info and record button side by side -->
                <div class="flex gap-2">
                  <!-- Device IP and port -->
                  <div 
                    @click="selectDevice(device)"
                    class="flex-1 cursor-pointer"
                  >
                    <div class="text-xs text-gray-400">
                      {{ $t('streaming.ipAddress') }}: <span class="text-gray-300">{{ device.ip }}</span>
                    </div>
                    <div class="text-xs text-gray-400">
                      {{ $t('streaming.port') }}: <span class="text-gray-300">{{ device.port }}</span>
                    </div>
                  </div>
                  
                  <!-- Record Test button -->
                  <button
                    @click="recordTestFromDevice(device)"
                    class="bg-blue-600 hover:bg-blue-700 text-white p-1 rounded-md text-xs flex items-center justify-center gap-1 h-full w-20"
                    :class="{'bg-red-600 hover:bg-red-700': isRecording && isDeviceSelected(device)}"
                  >
                    <!-- Record icon -->
                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <circle cx="12" cy="12" r="10"></circle>
                      <circle cx="12" cy="12" r="3"></circle>
                    </svg>
                    {{ isRecording && isDeviceSelected(device) ? $t('streaming.recording') : $t('streaming.recordTest') }}
                  </button>
                </div>
              </div>
            </li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/* -------------------------------------------------------------------
   imports
------------------------------------------------------------------- */
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { streamingActive, streamUrl as sharedStreamUrl, toggleStreamingState } from '../../store/appState';
import * as path from '@tauri-apps/api/path';
import { mkdir, BaseDirectory } from '@tauri-apps/plugin-fs';

/* -------------------------------------------------------------------
   initialize i18n
------------------------------------------------------------------- */
useI18n(); // Use the global $t in template

/* -------------------------------------------------------------------
   constants
------------------------------------------------------------------- */
const SRC_W = 320;           // incoming frame width
const SRC_H = 240;           // incoming frame height

/* -------------------------------------------------------------------
   reactive state
------------------------------------------------------------------- */
// Use a local reference for the UI, but sync with the shared state
// Initialize with empty string - will be populated from backend default or first discovered device
const streamUrl      = ref('');

// Function to fetch the default stream URL from backend
async function fetchDefaultStreamUrl() {
  try {
    const url = await invoke<string>('get_default_stream_url');
    console.log('Fetched default stream URL from backend:', url);
    
    if (url) {
      // If we have a stored default URL, use it
      streamUrl.value = url;
      sharedStreamUrl.value = url;
      console.log('Using stored default stream URL:', url);
    }
    
    return url;
  } catch (error) {
    console.error('Error fetching default stream URL:', error);
    return '';
  }
}

// Function to set the default stream URL in the backend
async function setDefaultStreamUrl(url: string) {
  if (!url) return;
  
  try {
    await invoke('set_default_stream_url', { url });
    console.log('Default stream URL set in backend:', url);
    
    // Also update shared state
    sharedStreamUrl.value = url;
  } catch (error) {
    console.error('Error setting default stream URL:', error);
  }
}
const isStreaming    = ref(streamingActive.value);
const fakeEnabled    = ref(false);
const isBrightEnough = ref(true);
const isRecording    = ref(false);

// Sync local streaming state with shared state
watch(streamingActive, (newValue) => {
  if (newValue !== isStreaming.value) {
    console.log(`Streaming state changed externally to: ${newValue}`);
    isStreaming.value = newValue;
  }
});

// Sync local stream URL with shared URL
watch(sharedStreamUrl, (newValue) => {
  if (newValue && newValue !== streamUrl.value) {
    console.log(`Stream URL changed externally to: ${newValue}`);
    streamUrl.value = newValue;
  }
});

// mDNS discovery state
const isDiscovering = ref(false);
const discoveredDevices = ref<any[]>([]);
const selectedDeviceId = ref(''); // format: "ip:port" or "manual"

/* -------------------------------------------------------------------
   canvas / contexts
------------------------------------------------------------------- */
// preview canvas (visible)
const canvas = ref<HTMLCanvasElement | null>(null);
let   prevCtx: CanvasRenderingContext2D;

// off‑screen canvas for high‑res recording
const recCanvas = document.createElement('canvas');
recCanvas.width = SRC_W;
recCanvas.height = SRC_H;
const recCtx = recCanvas.getContext('2d')!;

/* -------------------------------------------------------------------
   event unlisten handles
------------------------------------------------------------------- */
let frameUnlisten: () => void;
let errorUnlisten: () => void;
let analysisUnlisten: () => void;

/* ===================================================================
   streaming controls
=================================================================== */
async function toggleStreaming() {
  try {
    // The checkbox already updated isStreaming.value through v-model
    // So we need to act according to the new value
    const success = await toggleStreamingState(isStreaming.value, streamUrl.value);
    
    if (success) {
      console.log(`Streaming ${isStreaming.value ? 'started' : 'stopped'} successfully`);
      
      // If stopping streaming and recording is active, stop recording as well
      if (!isStreaming.value && isRecording.value) {
        await stopRecording();   // safety
      }
    } else {
      console.error('Failed to toggle streaming state');
      // Revert the checkbox state if there was an error
      isStreaming.value = !isStreaming.value;
    }
  } catch (error) {
    console.error('Error toggling streaming:', error);
    // Revert the checkbox state if there was an error
    isStreaming.value = !isStreaming.value;
  }
}

async function toggleFake() {
  try {
    // Toggle fake camera state in the backend (this is for the camera streaming)
    const newFakeState = await invoke<boolean>('toggle_fake_data');
    
    // Update local state to match backend state
    fakeEnabled.value = newFakeState;
    
    // If currently streaming, restart with the new fake state
    if (isStreaming.value) {
      await invoke('stop_streaming');
      await invoke('start_streaming', { path: streamUrl.value, fake: fakeEnabled.value });
    }
    
    console.log(`Fake camera ${fakeEnabled.value ? 'enabled' : 'disabled'}`);
  } catch (error) {
    console.error('Error toggling fake camera:', error);
  }
}

/* ===================================================================
   recording controls - using backend video recorder
=================================================================== */

async function startRecording() {
  try {
    // Get document directory
    const documentDir = await path.documentDir();
    
    // Create video_data directory if it doesn't exist
    try {
      await mkdir('video_data', {
        baseDir: BaseDirectory.Document,
      });
    } catch (error) {
      // Directory might already exist, ignore that error
      console.log('Directory setup note:', error);
    }
    
    // Join the document dir with video_data folder to get the full path
    const fullPath = await path.join(documentDir, 'video_data');
    
    // Create a filename with timestamp
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    const filename = `video_${timestamp}.mp4`;
    const savePath = await path.join(fullPath, filename);
    
    console.log('Using path for video recording:', savePath);
    
    // Start recording in the backend
    const result = await invoke('start_stream_recording', { filePath: savePath });
    
    if (result) {
      isRecording.value = true;
      console.log('Backend video recording started');
    } else {
      console.error('Failed to start backend video recording');
    }
  } catch (err) {
    console.error('Error starting recording:', err);
    alert(`Failed to start recording: ${err}`);
  }
}

async function stopRecording() {
  try {
    const result = await invoke('stop_stream_recording');
    console.log('Backend video recording stopped:', result);
    isRecording.value = false;
  } catch (err) {
    console.error('Error stopping recording:', err);
    alert(`Failed to stop recording: ${err}`);
  }
}

/* ===================================================================
   lifecycle
=================================================================== */
onMounted(async () => {
  // preview context
  prevCtx = canvas.value!.getContext('2d')!;

  // Get all streaming view state from backend in a single call
  try {
    // Get all state information in a single call
    const viewState = await invoke<any>('get_streaming_view_state');
    
    // Update component state with values from backend
    isStreaming.value = viewState.isStreaming;
    fakeEnabled.value = viewState.isFakeEnabled;
    isRecording.value = viewState.isRecording;
    
    console.log('Initial states from backend:', viewState);
    
    // Fetch the default stream URL from the backend
    await fetchDefaultStreamUrl();
  } catch (error) {
    console.error('Error fetching initial states:', error);
  }

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
  
  // Set up device discovery listeners
  await setupDeviceListeners();
  
  // Always load cached devices to ensure the list is populated
  loadCachedDevices();
  
  // Only start new device discovery if we're not already streaming
  if (!isStreaming.value) {
    discoverDevices();
  } else {
    console.log('Streaming already active, skipping device discovery scan');
  }
});

onUnmounted(() => {
  frameUnlisten?.();
  analysisUnlisten?.();
  errorUnlisten?.();
  if (isRecording.value) stopRecording();
});

/* ===================================================================
   Device discovery & selection
=================================================================== */
// Filter discovered devices with more lenient criteria to ensure we show potential cameras
const filteredDevices = computed(() => {
  // If we have no devices, return empty array
  if (!discoveredDevices.value || discoveredDevices.value.length === 0) {
    return [];
  }
  
  console.log('Filtering devices from:', discoveredDevices.value);
  
  // Show all devices for now, to ensure we're not missing any cameras
  // We can add more specific filtering later if needed
  return discoveredDevices.value;
});

// Load cached devices from the backend without initiating a new scan
async function loadCachedDevices() {
  try {
    console.log('Loading cached devices from backend');
    const devices = await invoke<any[]>('get_discovered_devices');
    
    console.log('Raw device data received:', devices);
    
    if (devices && devices.length > 0) {
      // Make sure we have valid device objects
      const validDevices = devices.filter(device => {
        if (!device || typeof device !== 'object') {
          console.warn('Invalid device data:', device);
          return false;
        }
        return true;
      });
      
      discoveredDevices.value = validDevices;
      console.log('Loaded cached devices:', discoveredDevices.value.length);
      console.log('Device details:', validDevices);
      
      // Automatically select the first device if we haven't selected one yet
      if (filteredDevices.value.length > 0 && selectedDeviceId.value === '') {
        selectDevice(filteredDevices.value[0]);
      }
    } else {
      console.log('No cached devices found');
      // Force a new discovery scan to make sure we have devices
      if (!isStreaming.value && !isDiscovering.value) {
        console.log('No devices in cache, forcing a new discovery scan');
        discoverDevices();
      }
    }
  } catch (error) {
    console.error('Error loading cached devices:', error);
    // Show the error to help debugging
    alert(`Error loading devices: ${error}. Check console for details.`);
  }
}

// Discover devices using mDNS
async function discoverDevices() {
  // Skip discovery if we're already streaming
  if (isStreaming.value) {
    console.log('Streaming is active, skipping device discovery');
    return;
  }
  
  try {
    isDiscovering.value = true;
    
    // Clear previous devices
    discoveredDevices.value = [];
    
    // Call backend to start discovery
    await invoke('discover_streaming_devices');
    
    // Set a timeout to fetch the results after discovery is complete
    setTimeout(async () => {
      try {
        // Get the discovered devices from the backend
        const devices = await invoke<any[]>('get_discovered_devices');
        discoveredDevices.value = devices || [];
        console.log('Discovered devices:', discoveredDevices.value);
      } catch (error) {
        console.error('Error getting discovered devices:', error);
      } finally {
        isDiscovering.value = false;
      }
    }, 4000); // Wait 4 seconds for discovery to complete
  } catch (error) {
    console.error('Error discovering devices:', error);
    isDiscovering.value = false;
  }
}

// Set up device-related event listeners
async function setupDeviceListeners() {
  // Listen for individual device discovery events
  await listen<any>('mdns_device_discovered', ({ payload }) => {
    // Check if we already have this device
    const existingIndex = discoveredDevices.value.findIndex(
      d => d.ip === payload.ip && d.port === payload.port
    );
    
    if (existingIndex >= 0) {
      // Update existing device
      discoveredDevices.value[existingIndex] = payload;
    } else {
      // Add new device
      discoveredDevices.value.push(payload);
    }
  });
  
  // Listen for the complete list of devices
  await listen<any[]>('mdns_devices_list', async ({ payload }) => {
    discoveredDevices.value = payload || [];
    isDiscovering.value = false;
    
    // Only auto-select a device if we don't have a URL yet or if selected device is not in the list
    if (filteredDevices.value.length > 0) {
      if (!streamUrl.value) {
        // No URL yet, select the first device
        console.log('No stream URL set yet, selecting first discovered device');
        await selectDevice(filteredDevices.value[0]);
      } else {
        // We have a URL, but check if it's still valid in the device list
        const currentDevice = filteredDevices.value.find(
          d => streamUrl.value.includes(`${d.ip}:${d.port}`)
        );
        
        if (!currentDevice) {
          console.log('Current stream URL device no longer available, selecting first discovered device');
          await selectDevice(filteredDevices.value[0]);
        } else {
          console.log('Current stream URL device still available, keeping the current selection');
        }
      }
    }
  });
}

// Select a device from the discovered list
async function selectDevice(device: any) {
  selectedDeviceId.value = `${device.ip}:${device.port}`;
  
  // Set the stream URL based on the device
  const newUrl = `http://${device.ip}:${device.port}/stream`;
  streamUrl.value = newUrl;
  console.log(`Selected device: ${device.name} at ${streamUrl.value}`);
  
  // Save this URL as the default in backend state
  await setDefaultStreamUrl(newUrl);
}

// Check if a device is currently selected
function isDeviceSelected(device: any): boolean {
  return selectedDeviceId.value === `${device.ip}:${device.port}`;
}

// Handle Record Test button click on a device card
async function recordTestFromDevice(device: any) {
  // First select this device
  selectDevice(device);
  
  // Construct the URL for recording
  const deviceUrl = constructDeviceUrl(device);
  console.log('Recording test from device URL:', deviceUrl);
  
  try {
    if (isRecording.value) {
      // If already recording, stop recording
      await stopRecording();
      
      // Also stop streaming if it was active
      if (isStreaming.value) {
        isStreaming.value = false;
        await invoke('stop_streaming');
      }
      return;
    }
    
    // Set the stream URL for both streaming and recording
    streamUrl.value = deviceUrl;
    
    // First enable streaming (required for recording)
    if (!isStreaming.value) {
      isStreaming.value = true; // Update UI toggle
      await invoke('start_streaming', { path: streamUrl.value, fake: fakeEnabled.value });
      console.log('Streaming started for recording');
    }
    
    // Then start recording
    await startRecording();
    
    console.log('Started test recording for device:', device.name);
  } catch (error) {
    console.error('Failed to start test recording:', error);
    alert(`Failed to start test recording: ${error}`);
    
    // Revert streaming state if there was an error
    if (isStreaming.value) {
      isStreaming.value = false;
      try {
        await invoke('stop_streaming');
      } catch (e) {
        console.error('Error stopping streaming after recording failure:', e);
      }
    }
  }
}

// Helper function to construct device URL
function constructDeviceUrl(device: any) {
  // Default to http if not specified
  const protocol = device.service_type?.includes('rtsp') ? 'rtsp' : 'http';
  return `${protocol}://${device.ip}:${device.port}/stream`;
}

// Select the manually entered URL
async function selectManualUrl() {
  if (!streamUrl.value) {
    console.warn('No URL entered manually, cannot select');
    return;
  }
  
  selectedDeviceId.value = 'manual';
  console.log(`Using manual URL: ${streamUrl.value}`);
  
  // Save this URL as the default in backend state
  await setDefaultStreamUrl(streamUrl.value);
}
</script>

<style scoped>
/* optional extra styling */
</style>
