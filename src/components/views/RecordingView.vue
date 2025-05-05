<template>
  <div class="recording-view">
    <div class="bg-gray-800 bg-opacity-60 rounded-lg p-6 mb-6">
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
  
  <!-- Recorded Files Section -->
  <div class="bg-gray-800 bg-opacity-60 rounded-lg p-6">
    <div class="flex justify-between items-center mb-6">
      <h2 class="text-3xl font-bold text-blue-400">Recorded Files</h2>
      <button 
        @click="loadFiles"
        class="bg-gray-600 hover:bg-gray-700 px-4 py-2 rounded-md text-sm flex items-center gap-2 transition-colors">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
          stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 2v6h-6"></path>
          <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"></path>
          <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L16 16"></path>
          <path d="M16 16h6v6"></path>
        </svg>
        Refresh
      </button>
    </div>
    
    <div v-if="recordingDirectory" class="mb-4 text-sm text-gray-400">
      Monitoring: {{ recordingDirectory }}
    </div>
    
    <div v-if="errorMessage" class="bg-red-900 bg-opacity-50 p-4 rounded-md mb-4 text-sm">
      {{ errorMessage }}
    </div>
    
    <div v-if="isLoading" class="flex justify-center items-center py-8">
      <div class="animate-spin h-8 w-8 border-4 border-blue-500 rounded-full border-t-transparent"></div>
    </div>
    
    <div v-else-if="folderFiles.length === 0" class="bg-gray-700 p-6 rounded-md text-center">
      <p v-if="recordingDirectory">No CSV files found in the selected directory.</p>
      <p v-else>Select a directory to view recorded files.</p>
    </div>
    
    <div v-else class="file-grid grid gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3">
      <transition-group name="filter" tag="div" class="contents">
        <div 
          v-for="file in folderFiles" 
          :key="file.path" 
          :class="[
            'bg-gray-700 hover:bg-gray-600 p-4 rounded-lg transition-all duration-300 transform hover:scale-[1.02] flex flex-col relative',
            {'recording-pulse': isActiveRecordingFile(file)}
          ]"
          @click="logRecordingStatus(file)">
          
          <!-- File card header with name and actions -->
          <div class="flex justify-between items-start mb-3">
            <h4 class="font-medium truncate flex-grow" :title="file.name">{{ file.name }}</h4>
            <div class="flex items-center gap-2 ml-2 flex-shrink-0">
              <!-- Use a simpler badge + border approach -->
              <span v-if="isActiveRecordingFile(file)" 
                class="text-xs px-2 py-1 rounded bg-red-600 text-white font-bold shadow-md shadow-red-500/50">
                RECORDING
              </span>
              <button 
                @click="handleDeleteFile(file.path)" 
                class="text-xs p-1 rounded bg-gray-600 hover:bg-gray-500 text-white">
                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7"/>
                  <path stroke-linecap="round" stroke-linejoin="round" d="M10 11v6M14 11v6"/>
                  <path stroke-linecap="round" stroke-linejoin="round" d="M9 7h6"/>
                </svg>
              </button>
            </div>
          </div>
          
          <!-- File details -->
          <div class="space-y-3 flex-grow text-sm">
            <div class="flex items-center gap-2">
              <span class="text-sm text-gray-400">Size:</span>
              <span class="text-white">{{ file.size }}</span>
            </div>
            <div class="flex items-center gap-2">
              <span class="text-sm text-gray-400">Date:</span>
              <span class="text-white">{{ formatDate(file) }}</span>
            </div>
            <div class="flex items-center gap-2">
              <span class="text-sm text-gray-400">Time:</span>
              <span class="text-white">{{ formatTime(file) }}</span>
            </div>
          </div>
          
          <!-- Actions footer -->
          <div class="flex gap-2 mt-4 justify-between">
            <button 
              @click="handleOpenFile(file.path)"
              class="bg-gray-600 hover:bg-gray-500 text-white px-3 py-1.5 rounded text-xs font-medium flex items-center gap-1 transition-colors">
              <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none"
                stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
                <polyline points="15 3 21 3 21 9"></polyline>
                <line x1="10" y1="14" x2="21" y2="3"></line>
              </svg>
              Open
            </button>
            <button 
              @click="handleUploadFile(file.path)"
              class="bg-blue-600 hover:bg-blue-700 text-white px-3 py-1.5 rounded text-xs font-medium flex items-center gap-1 transition-colors flex-grow justify-center">
              <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none"
                stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                <polyline points="17 8 12 3 7 8"></polyline>
                <line x1="12" y1="3" x2="12" y2="15"></line>
              </svg>
              Upload
            </button>
          </div>
        </div>
      </transition-group>
    </div>
  </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { exists, mkdir, BaseDirectory } from '@tauri-apps/plugin-fs';
import * as path from '@tauri-apps/api/path';
import { recordingDirectory } from '../../store/appState';
import { 
  loadDirectoryFiles,
  setupDirectoryWatcher,
  setupRecordingFileWatcher,
  findAndUpdateActiveRecordingFile,
  isActiveRecordingFile as checkActiveFile,
  openFileLocation,
  uploadFile,
  deleteFile,
  updateFilesInPlace,
  formatDate,
  formatTime,
} from '../../utils/fileManager';

// Note: We'll use the native file structure without trying to import the interface

// State variables
const selectedFormat = ref('csv'); // Default format
const autoStart = ref(false);
const maxDuration = ref(30); // Default 30 minutes
const isRecording = ref(false);
const recordingFilename = ref('');
// Keep track of files in the selected folder
const folderFiles = ref([]);
const isLoading = ref(false);
const errorMessage = ref('');
const activeRecordingPath = ref('');
const watchUnsubscribe = ref(null);
const fileWatchUnsubscribe = ref(null);

// Initialize recording directory and check recording status when component mounts
onMounted(async () => {
  try {
    // First, set up the recordings directory in AppData
    await selectDirectory();
    const status = await invoke('get_recording_status');
    isRecording.value = status;
    
    // If recording is active, find and update the active recording file
    if (isRecording.value) {
      await findRecordingFile();
    }
  } catch (error) {
    console.error('Error initializing component:', error);
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
  
  // Clean up directory watcher if active
  if (watchUnsubscribe.value) {
    watchUnsubscribe.value();
  }

  // Clean up file watcher if active
  if (fileWatchUnsubscribe.value) {
    fileWatchUnsubscribe.value();
  }
});

// We're using checkActiveFile from the fileManager utility directly

// Use AppData directory for recordings
async function selectDirectory() {
  const home = await path.documentDir();
  // await mkdir(home, { baseDir: BaseDirectory.Home });
  recordingDirectory.value = home;
  console.log(home);
  
  // Load files from the directory
  await loadFiles();
  
  // Set up a watcher for the directory
  await initDirectoryWatcher();
}

// Refresh directory files
async function loadFiles() {
  if (!recordingDirectory.value) return;
  errorMessage.value = ''; // Clear any previous errors
  
  // Only show loading indicator if we have no files yet to avoid flickering
  if (folderFiles.value.length === 0) {
    isLoading.value = true;
  }
  
  try {
    // Load all files from the directory
    const files = await loadDirectoryFiles(recordingDirectory.value);
    
    // Use optimized update function to maintain transitions
    updateFilesInPlace(
      files, 
      folderFiles.value, 
      newFiles => folderFiles.value = newFiles, 
      loading => isLoading.value = loading
    );
  } catch (error) {
    console.error('Error loading files:', error);
    errorMessage.value = `Error loading files: ${error}`;
    isLoading.value = false;
  }
}

// We're using the formatDate and formatTime functions directly from fileManager

// Set up a watcher for the directory to detect file additions/removals
async function initDirectoryWatcher() {
  await setupDirectoryWatcher(
    recordingDirectory.value,
    async () => await loadFiles(),
    (unsubscribeFn) => watchUnsubscribe.value = unsubscribeFn,
    (error) => errorMessage.value = `Failed to watch directory: ${error}`
  );
}

// Find the active recording file and update its information
async function findRecordingFile() {
  await findAndUpdateActiveRecordingFile(
    isRecording.value,
    recordingDirectory.value,
    recordingFilename.value,
    selectedFormat.value,
    folderFiles.value,
    (path) => activeRecordingPath.value = path,
    (name) => recordingFilename.value = name,
    (files) => folderFiles.value = files
  );
}

// Helper function to check if a file is the active recording file
function isActiveRecordingFile(file) {
  // Additional debugging
  console.log('Checking if file is active recording:', file.name);
  console.log('Current recording filename:', recordingFilename.value); 
  console.log('Is recording:', isRecording.value);
  
  // Simple but most reliable check since we get filename directly from backend
  return isRecording.value && file.name === recordingFilename.value;
}

// Debug function to check the recording status
function logRecordingStatus(file) {
  console.log('File name:', file.name);
  console.log('Recording filename:', recordingFilename.value);
  console.log('Is Recording:', isRecording.value);
  console.log('Match?', file.name === recordingFilename.value);
  console.log('Should show green:', isActiveRecordingFile(file));
}

// Handle file operations with extracted utility functions
async function handleDeleteFile(filePath) {
  try {
    await deleteFile(filePath, loadFiles);
  } catch (error) {
    alert(error.message);
  }
}

async function handleOpenFile(filePath) {
  try {
    await openFileLocation(filePath);
  } catch (error) {
    alert(error.message);
  }
}

async function handleUploadFile(filePath) {
  try {
    await uploadFile(filePath);
  } catch (error) {
    alert(error.message);
  }
}



// No longer needed - replaced by findAndUpdateActiveRecordingFile

// Start recording using the selected settings
async function startRecording() {
  if (!recordingDirectory.value) {
    alert('Please select a directory to save recordings');
    return;
  }
  
  try {
    console.log('Starting recording with format:', selectedFormat.value);
    console.log('Directory:', recordingDirectory.value);
    
    // Start recording through Tauri command and get the actual filename
    const actualFilename = await invoke('start_recording', {
      format: selectedFormat.value,
      directory: recordingDirectory.value,
      maxDurationMinutes: maxDuration.value,
      autoStart: autoStart.value
    });
    
    console.log('Received filename from backend:', actualFilename);
    
    // Store the actual filename returned from the backend
    recordingFilename.value = actualFilename;
    console.log('Set recordingFilename.value to:', recordingFilename.value);
    
    // Log the recording filename in the global window object for debugging
    // @ts-ignore
    window.recordingFilename = recordingFilename.value;
    
    // Start Android foreground service to keep app alive in background
    try {
      await invoke('plugin:android-forward-service|start_forward_service');
      console.log('Android foreground service started');
    } catch (e) {
      console.warn('Forward service not available or failed to start:', e);
    }
    
    isRecording.value = true;
  console.log('Recording started - filename:', recordingFilename.value);
    
    // Set up a watcher for the recording file
    const fullPath = `${recordingDirectory.value}/${recordingFilename.value}`;
    fileWatchUnsubscribe.value = await setupRecordingFileWatcher(
      fullPath,
      async () => {
        // When file changes, update it without triggering a full reload
        await findAndUpdateActiveRecordingFile(
          isRecording.value,
          recordingDirectory.value,
          recordingFilename.value,
          selectedFormat.value,
          folderFiles.value,
          (path) => activeRecordingPath.value = path,
          (name) => recordingFilename.value = name,
          (files) => folderFiles.value = files
        );
      },
      fileWatchUnsubscribe.value
    );
    
    // Also set up a periodic update for the active recording file
    // This ensures the size and time get updated continuously
    const updateInterval = setInterval(async () => {
      if (isRecording.value) {
        console.log('Periodic update check - recording filename:', recordingFilename.value);
        
        await findAndUpdateActiveRecordingFile(
          isRecording.value,
          recordingDirectory.value,
          recordingFilename.value,
          selectedFormat.value,
          folderFiles.value,
          (path) => {
            console.log('Setting activeRecordingPath to:', path);
            activeRecordingPath.value = path;
          },
          (name) => {
            console.log('Setting recordingFilename to:', name);
            recordingFilename.value = name;
          },
          (files) => {
            console.log('Updating folderFiles, count:', files.length);
            folderFiles.value = files;
          }
        );
        
        // Double check if we have the actual recording file in our list
        const recordingFileExists = folderFiles.value.some(file => file.name === recordingFilename.value);
        console.log('Recording file exists in folder list?', recordingFileExists);
      } else {
        console.log('Recording stopped, clearing interval');
        clearInterval(updateInterval);
      }
    }, 2000); // Update every 2 seconds
    
    // Reload directory to ensure the new file shows up
    setTimeout(async () => {
      await loadFiles();
    }, 1000); // Give the file system a moment to create the file
  } catch (error) {
    console.error('Error starting recording:', error);
    alert(`Failed to start recording: ${error}`);
  }
}



async function stopRecording() {
  try {
    await invoke('stop_recording');
    
    // Stop Android foreground service
    try {
      await invoke('plugin:android-forward-service|stop_forward_service');
      console.log('Android foreground service stopped');
    } catch (e) {
      console.warn('Forward service not available or failed to stop:', e);
    }
    
    // Clean up file watcher if active
    if (fileWatchUnsubscribe.value) {
      fileWatchUnsubscribe.value();
      fileWatchUnsubscribe.value = null;
    }
    
    // Clear any update intervals that might be running
    // This is handled automatically since we check isRecording.value
    
    // Clear the active recording path
    activeRecordingPath.value = '';
    
    isRecording.value = false;
    recordingFilename.value = '';
    
    // Final refresh of directory contents
    await loadFiles();
  } catch (error) {
    console.error('Error stopping recording:', error);
    alert(`Failed to stop recording: ${error}`);
  }
}
</script>

<style scoped>
/* Animations for recording indicator */
.recording-pulse {
  background-color: rgba(16, 185, 129, 0.15) !important;
  border: 4px solid #10b981 !important;
  animation: recording-card-pulse 2s ease-in-out infinite alternate;
}

@keyframes recording-card-pulse {
  0% {
    box-shadow: 0 0 5px rgba(16, 185, 129, 0.4);
    border-color: #10b981;
  }
  100% {
    box-shadow: 0 0 15px rgba(16, 185, 129, 0.8);
    border-color: #34d399;
  }
}

/* Filter card add/delete animations - same as in FilterConfigView */
.filter-enter-active, .filter-leave-active {
  transition: all 200ms ease-in-out;
}
.filter-enter-from, .filter-leave-to {
  opacity: 0;
  transform: scale(0.9);
}
.filter-enter-to, .filter-leave-from {
  opacity: 1;
  transform: scale(1);
}

/* Prevent flicker during updates */
.file-grid {
  position: relative;
  min-height: 100px; /* Ensure grid has minimum height while loading */
}

.contents > * {
  will-change: opacity, transform;
}
</style>
