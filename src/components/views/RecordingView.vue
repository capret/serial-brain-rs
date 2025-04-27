<template>
  <div class="bg-gray-800 bg-opacity-60 rounded-lg p-6">
    <div class="flex flex-wrap justify-between items-start mb-6">
      <div class="max-[800px]:w-full">
        <h2 class="text-3xl font-bold text-blue-400">Recording Setup</h2>
      </div>
      <div class="flex gap-3 max-[800px]:w-full max-[800px]:mt-4">
        <button 
          v-if="!isRecording"
          @click="startRecording"
          :disabled="!recordingDirectory"
          class="bg-blue-600 hover:bg-blue-700 px-6 py-3 rounded-md font-semibold flex items-center justify-center gap-2 transition-all duration-300 transform hover:scale-105 shadow-lg max-[800px]:w-full"
          :class="{ 'opacity-50 cursor-not-allowed': !recordingDirectory }">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polygon points="5 3 19 12 5 21 5 3"></polygon>
          </svg>
          Start Recording
        </button>
        <button 
          v-else
          @click="stopRecording"
          class="bg-red-600 hover:bg-red-700 px-6 py-3 rounded-md font-semibold flex items-center justify-center gap-2 transition-all duration-300 transform hover:scale-105 shadow-lg max-[800px]:w-full">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="6" y="6" width="12" height="12"></rect>
          </svg>
          Stop Recording
        </button>
      </div>
    </div>
    <div class="space-y-6">
      <div>
        <h3 class="text-lg font-semibold mb-4">Storage Location</h3>
        <div class="mt-2">
          <div class="flex items-center justify-between mb-2">
            <p class="text-sm">Select folder to record data</p>
            <span class="text-xs text-gray-400">Required</span>
          </div>
          <div class="flex">
            <input 
              type="text" 
              :placeholder="recordingDirectory ? recordingDirectory : 'No folder selected'" 
              v-model="recordingDirectory"
              readonly
              class="bg-gray-700 px-3 py-2 rounded-l-md flex-grow text-gray-300" />
            <button
              @click="selectDirectory"
              :disabled="isRecording"
              class="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-r-md transition-colors duration-300"
              :class="{ 'opacity-50 cursor-not-allowed': isRecording }">
              Browse
            </button>
          </div>
        </div>
      </div>
      
      <div>
        <h3 class="text-lg font-semibold mb-4">Recording Format</h3>
        <div class="grid grid-cols-3 gap-4">
          <div 
            class="bg-gray-700 p-4 rounded-lg cursor-pointer" 
            :class="{'border-2 border-blue-500': selectedFormat === 'csv'}"
            @click="selectedFormat = 'csv'">
            <div class="flex items-center gap-2">
              <input type="radio" name="format" :checked="selectedFormat === 'csv'" id="csv" />
              <label for="csv" class="font-medium">CSV</label>
            </div>
            <p class="text-xs text-gray-400 mt-1">Standard format compatible with most analysis tools</p>
          </div>
          <div 
            class="bg-gray-700 p-4 rounded-lg cursor-pointer" 
            :class="{'border-2 border-blue-500': selectedFormat === 'binary'}"
            @click="selectedFormat = 'binary'">
            <div class="flex items-center gap-2">
              <input type="radio" name="format" :checked="selectedFormat === 'binary'" id="binary" />
              <label for="binary" class="font-medium">Binary</label>
            </div>
            <p class="text-xs text-gray-400 mt-1">Compact storage for large datasets</p>
          </div>
          <div 
            class="bg-gray-700 p-4 rounded-lg cursor-pointer" 
            :class="{'border-2 border-blue-500': selectedFormat === 'json'}"
            @click="selectedFormat = 'json'">
            <div class="flex items-center gap-2">
              <input type="radio" name="format" :checked="selectedFormat === 'json'" id="json" />
              <label for="json" class="font-medium">JSON</label>
            </div>
            <p class="text-xs text-gray-400 mt-1">Structured format with metadata</p>
          </div>
        </div>
      </div>
      
      <div>
        <h3 class="text-lg font-semibold mb-4">Recording Options</h3>
        <div class="space-y-4">
          <div class="flex items-center">
            <input 
              type="checkbox" 
              id="autostart" 
              v-model="autoStart" 
              :disabled="isRecording"
              class="mr-2" />
            <label for="autostart">Auto-start recording when signal is detected</label>
          </div>
          <div>
            <label class="block mb-2 text-sm">Maximum recording duration (minutes)</label>
            <input 
              type="number" 
              v-model="maxDuration" 
              min="1" 
              :disabled="isRecording"
              class="bg-gray-700 px-3 py-2 rounded-md w-32" />
          </div>
        </div>
      </div>
      
      <div v-if="isRecording" class="mt-6 p-4 bg-gray-700 rounded-lg">
        <div class="flex justify-between items-center">
          <h3 class="text-lg font-semibold text-green-400">Recording in Progress</h3>
          <div class="flex items-center gap-2">
            <div class="animate-pulse h-3 w-3 rounded-full bg-red-500"></div>
            <span class="text-sm">Recording...</span>
          </div>
        </div>
        <p class="mt-4 text-sm">Saving data to: <span class="text-blue-300">{{ recordingFilename }}</span></p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { recordingDirectory } from '../../store/appState';

// State variables
const selectedFormat = ref('csv'); // Default format
const autoStart = ref(false);
const maxDuration = ref(30); // Default 30 minutes
const isRecording = ref(false);
const recordingFilename = ref('');

// Check initial recording status when component mounts
onMounted(async () => {
  try {
    const status = await invoke('get_recording_status');
    isRecording.value = status;
  } catch (error) {
    console.error('Error checking recording status:', error);
  }
});

// Clean up when component unmounts
onUnmounted(async () => {
  // If recording is still active when leaving the view, ask if we should stop it
  if (isRecording.value) {
    const shouldStop = confirm('Recording is still in progress. Stop recording?');
    if (shouldStop) {
      await stopRecording();
    }
  }
});

// Select directory using Tauri file dialog
async function selectDirectory() {
  try {
    const directory = await invoke('select_recording_directory');
    if (directory) {
      recordingDirectory.value = directory;
    }
  } catch (error) {
    console.error('Error selecting directory:', error);
  }
}

// Start recording using the selected settings
async function startRecording() {
  if (!recordingDirectory.value) {
    alert('Please select a directory to save recordings');
    return;
  }
  
  try {
    // Generate recording filename based on current timestamp
    const timestamp = new Date().getTime();
    recordingFilename.value = `serial_recording_${timestamp}.${selectedFormat.value}`;
    
    // Start recording through Tauri command
    await invoke('start_recording', {
      format: selectedFormat.value,
      directory: recordingDirectory.value,
      maxDurationMinutes: maxDuration.value,
      autoStart: autoStart.value
    });
    
    isRecording.value = true;
  } catch (error) {
    console.error('Error starting recording:', error);
    alert(`Failed to start recording: ${error}`);
  }
}

// Stop the current recording
async function stopRecording() {
  try {
    await invoke('stop_recording');
    isRecording.value = false;
    recordingFilename.value = '';
  } catch (error) {
    console.error('Error stopping recording:', error);
    alert(`Failed to stop recording: ${error}`);
  }
}
</script>

<style scoped>
.recording-pulse {
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0% {
    opacity: 1;
  }
  50% {
    opacity: 0.3;
  }
  100% {
    opacity: 1;
  }
}
</style>
