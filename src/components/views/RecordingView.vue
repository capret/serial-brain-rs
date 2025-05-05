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
        @click="loadDirectoryFiles"
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
          class="bg-gray-700 hover:bg-gray-600 p-4 rounded-lg transition-all duration-300 transform hover:scale-[1.02] flex flex-col"
          :class="{ 'ring-2 ring-green-500 ring-opacity-75 shadow-lg shadow-green-500/30 recording-pulse': isActiveRecordingFile(file.name) }">
          
          <!-- File card header with name and actions -->
          <div class="flex justify-between items-start mb-3">
            <h4 class="font-medium truncate flex-grow" :title="file.name">{{ file.name }}</h4>
            <div class="flex items-center gap-2 ml-2 flex-shrink-0">
              <span v-if="isActiveRecordingFile(file.name)" 
                class="text-xs px-2 py-1 rounded bg-green-600 text-white font-medium">
                Recording
              </span>
              <button 
                @click="deleteFile(file.path)" 
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
              @click="openFileLocation(file.path)"
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
              @click="uploadFile(file.path)"
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
import { exists, mkdir, stat, readDir, watchImmediate, BaseDirectory } from '@tauri-apps/plugin-fs';
import * as path from '@tauri-apps/api/path';
import { platform } from '@tauri-apps/plugin-os';
import { recordingDirectory } from '../../store/appState';

// State variables
const selectedFormat = ref('csv'); // Default format
const autoStart = ref(false);
const maxDuration = ref(30); // Default 30 minutes
const isRecording = ref(false);
const recordingFilename = ref('');
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
    await watchUnsubscribe.value();
  }
  
  // Clean up file watcher if active
  if (fileWatchUnsubscribe.value) {
    await fileWatchUnsubscribe.value();
  }
});

// Check if a filename is the current active recording file
function isActiveRecordingFile(filename) {
  if (!isRecording.value) return false;
  if (recordingFilename.value) {
    const match = recordingFilename.value.match(/serial_recording_(\d+)/);
    if (match && match[1]) {
      const timestampPrefix = match[1].substring(0, 10);
      return filename.includes(timestampPrefix);
    }
  }
  
  return false;
}

// Use AppData directory for recordings
async function selectDirectory() {
  const home = await path.documentDir();
  recordingDirectory.value = home;
  console.log(home);
  
  await loadDirectoryFiles();
  await setupDirectoryWatcher();
}

// Function to load files from the selected directory
async function loadDirectoryFiles() {
  if (!recordingDirectory.value) return;
  
  // Only show loading indicator if we have no files yet
  if (folderFiles.value.length === 0) {
    isLoading.value = true;
  }
  errorMessage.value = '';
  
  try {
    // Read all files in the directory
    const entries = await readDir(recordingDirectory.value);
    
    // Filter for CSV files
    const csvEntries = entries.filter(entry => !entry.children && entry.name.toLowerCase().endsWith('.csv'));
    
    // Get file stats for each CSV file
    const filesWithStats = await Promise.all(
      csvEntries.map(async entry => {
        try {
          const fileStat = await stat(`${recordingDirectory.value}/${entry.name}`);
          // Get the modified date from FileInfo
          const modifiedDate = fileStat.mtime || fileStat.birthtime;
          
          return {
            name: entry.name,
            path: `${recordingDirectory.value}/${entry.name}`,
            size: formatFileSize(fileStat.size),
            modified: modifiedDate instanceof Date ? modifiedDate.toLocaleString() : 'Unknown',
            rawSize: fileStat.size,
            dateObject: modifiedDate,
            // Add unique key for Vue transitions
            key: entry.name
          };
        } catch (error) {
          console.error(`Error getting stats for ${entry.name}:`, error);
          return {
            name: entry.name,
            path: `${recordingDirectory.value}/${entry.name}`,
            size: 'Unknown',
            modified: 'Unknown',
            rawSize: 0,
            dateObject: null,
            key: entry.name
          };
        }
      })
    );
    
    // Sort files by modified date (newest first)
    const sortedFiles = filesWithStats.sort((a, b) => {
      // Use Date objects for comparison if available
      if (a.dateObject instanceof Date && b.dateObject instanceof Date) {
        return b.dateObject.getTime() - a.dateObject.getTime();
      }
      // Fall back to filename timestamp if no valid dates
      const getTimestamp = (file) => {
        const match = file.name.match(/serial_recording_(\d+)/);
        return match ? parseInt(match[1]) : 0;
      };
      return getTimestamp(b) - getTimestamp(a);
    });
    
    // Update existing files in-place to avoid flashing
    updateFilesInPlace(sortedFiles);
  } catch (error) {
    console.error('Error loading directory files:', error);
    errorMessage.value = `Failed to load files: ${error}`;
    folderFiles.value = [];
  } finally {
    isLoading.value = false;
  }
}

// Format file size in a human-readable format
function formatFileSize(bytes) {
  if (bytes === 0) return '0 Bytes';
  
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  
  return parseFloat((bytes / Math.pow(1024, i)).toFixed(2)) + ' ' + sizes[i];
}

// Format the date portion of a file timestamp
function formatDate(file) {
  // Check if file has valid date information from FileInfo
  if (!file) return 'Unknown';
  
  try {
    // Use the modified date directly from file.modified which is now formatted correctly
    if (file.modified && file.modified.includes(',')) {
      return file.modified.split(',')[0].trim();
    }
    
    // Try getting the date from the filename as a fallback
    const match = file.name.match(/serial_recording_(\d+)/);
    if (match && match[1]) {
      const timestamp = parseInt(match[1]);
      if (!isNaN(timestamp)) {
        return new Date(timestamp).toLocaleDateString();
      }
    }
    
    // Get today's date as fallback
    return new Date().toLocaleDateString();
  } catch (e) {
    console.error('Error formatting date:', e);
    return 'Unknown';
  }
}

// Format the time portion of a file timestamp
function formatTime(file) {
  // Check if file has valid date information from FileInfo
  if (!file) return 'Unknown';
  
  try {
    // Use the modified date directly from file.modified which is now formatted correctly
    if (file.modified && file.modified.includes(',')) {
      return file.modified.split(',')[1]?.trim() || 'Unknown';
    }
    
    // Try getting the time from the filename as a fallback
    const match = file.name.match(/serial_recording_(\d+)/);
    if (match && match[1]) {
      const timestamp = parseInt(match[1]);
      if (!isNaN(timestamp)) {
        return new Date(timestamp).toLocaleTimeString();
      }
    }
    
    // Get current time as fallback
    return new Date().toLocaleTimeString();
  } catch (e) {
    console.error('Error formatting time:', e);
    return 'Unknown';
  }
}

// Set up a watcher for the directory to detect file additions/removals
async function setupDirectoryWatcher() {
  // Clean up existing watcher if any
  if (watchUnsubscribe.value) {
    await watchUnsubscribe.value();
    watchUnsubscribe.value = null;
  }
  
  if (!recordingDirectory.value) return;
  
  try {
    // Watch the directory for changes (new files, deleted files, etc) with immediate notifications
    watchUnsubscribe.value = await watchImmediate(
      recordingDirectory.value,
      async (event) => {
        console.log('Directory event:', event);
        // Reload directory contents when changes are detected
        await loadDirectoryFiles();
      },
      { recursive: false }
    );
    console.log('Directory watcher set up successfully with immediate notifications');
  } catch (error) {
    console.error('Error setting up directory watcher:', error);
    errorMessage.value = `Failed to watch directory: ${error}`;
  }
}

// Set up a watcher for the directory during recording to track active file changes
async function setupRecordingFileWatcher() {
  // Clean up any existing file watcher
  if (fileWatchUnsubscribe.value) {
    await fileWatchUnsubscribe.value();
    fileWatchUnsubscribe.value = null;
  }
  
  if (!isRecording.value || !recordingDirectory.value) return;
  
  // Instead of watching a specific file, we'll watch the directory for changes
  // and update any file that matches our timestamp pattern
  try {
    // Create a directory watcher with immediate notifications
    fileWatchUnsubscribe.value = await watchImmediate(
      recordingDirectory.value,
      async (event) => {
        console.log('Recording file event:', event);
        // When directory changes, look for the active recording file and update it
        await findAndUpdateActiveRecordingFile();
      },
      { recursive: false }
    );
    console.log('Recording directory watcher set up successfully with immediate notifications');
    
    // Do an immediate check to find the file
    await findAndUpdateActiveRecordingFile();
  } catch (error) {
    console.error('Error setting up recording watcher:', error);
    // Fall back to polling if watch fails
    const intervalId = setInterval(async () => {
      if (isRecording.value) {
        await findAndUpdateActiveRecordingFile();
      } else {
        clearInterval(intervalId);
      }
    }, 1000);
  }
}

// Helper function to update files in-place to avoid UI flashing
function updateFilesInPlace(newFiles) {
  if (!newFiles || newFiles.length === 0) {
    isLoading.value = false;
    return;
  }

  // Special case: if we have no existing files, use the special Vue set operation to avoid flicker
  if (folderFiles.value.length === 0) {
    // Add files one by one with a stable key to enable proper transitions
    newFiles.forEach(file => {
      file.key = file.path; // Ensure stable key for Vue's transition system
      folderFiles.value.push(file);
    });
    isLoading.value = false;
    return;
  }
  
  // Create maps for quick lookups
  const existingFilesMap = new Map(folderFiles.value.map(file => [file.path, file]));
  const newFilesMap = new Map(newFiles.map(file => [file.path, file]));
  
  // Track which files we've already processed to avoid duplication
  const processedPaths = new Set();
  
  // 1. First update existing files in place (most important to avoid flicker)
  for (let i = 0; i < folderFiles.value.length; i++) {
    const existingFile = folderFiles.value[i];
    const newFile = newFilesMap.get(existingFile.path);
    
    if (newFile) {
      // File exists in both arrays - update properties individually
      // instead of replacing the entire object to preserve reactivity
      if (existingFile.rawSize !== newFile.rawSize) {
        existingFile.rawSize = newFile.rawSize;
        existingFile.size = newFile.size;
      }
      
      if (newFile.dateObject instanceof Date && 
          (!existingFile.dateObject instanceof Date || 
           existingFile.dateObject.getTime() !== newFile.dateObject.getTime())) {
        existingFile.dateObject = newFile.dateObject;
        existingFile.modified = newFile.modified;
      }
      
      // Mark as processed
      processedPaths.add(existingFile.path);
    }
  }
  
  // 2. Remove files that don't exist in the new list (using reverse loop to avoid index issues)
  for (let i = folderFiles.value.length - 1; i >= 0; i--) {
    const existingFile = folderFiles.value[i];
    if (!newFilesMap.has(existingFile.path)) {
      // Remove this file - it's not in the new list
      folderFiles.value.splice(i, 1);
    }
  }
  
  // 3. Add new files that weren't in the original list
  for (const newFile of newFiles) {
    if (!processedPaths.has(newFile.path)) {
      // This is a new file we need to add
      // Ensure it has a stable key for Vue transitions
      newFile.key = newFile.path;
      
      // Find the right position based on date sorting
      let insertIndex = folderFiles.value.findIndex(file => {
        if (file.dateObject instanceof Date && newFile.dateObject instanceof Date) {
          return newFile.dateObject.getTime() > file.dateObject.getTime();
        }
        // Fallback to filename timestamp comparison if dates aren't available
        const getTimestamp = (f) => {
          const match = f.name.match(/serial_recording_(\d+)/);
          return match ? parseInt(match[1]) : 0;
        };
        return getTimestamp(newFile) > getTimestamp(file);
      });
      
      // If no appropriate position found, add to the end
      if (insertIndex === -1) {
        folderFiles.value.push(newFile);
      } else {
        // Insert at the proper position
        folderFiles.value.splice(insertIndex, 0, newFile);
      }
    }
  }
  
  isLoading.value = false;
}

// Find the actual recording file in the directory and update its size
async function findAndUpdateActiveRecordingFile() {
  if (!isRecording.value || !recordingDirectory.value) return;
  
  try {
    // Read all files in the directory
    const entries = await readDir(recordingDirectory.value);
    
    // Extract timestamp pattern from current recording filename
    let timestampPrefix = '';
    if (recordingFilename.value) {
      const match = recordingFilename.value.match(/serial_recording_(\d+)/);
      if (match && match[1]) {
        timestampPrefix = match[1].substring(0, 10);
      }
    }
    
    // Find files matching the timestamp pattern
    for (const entry of entries) {
      if (!entry.children && entry.name.includes(timestampPrefix) && 
          entry.name.toLowerCase().endsWith(`.${selectedFormat.value}`)) {
        // Found the actual recording file
        const actualPath = `${recordingDirectory.value}/${entry.name}`;
        activeRecordingPath.value = actualPath;
        
        // Update just this file's size without refreshing the entire list
        try {
          const fileStat = await stat(actualPath);
          
          // Find and update the file in the folderFiles array
          const fileIndex = folderFiles.value.findIndex(file => file.path === actualPath);
          
          if (fileIndex >= 0) {
            // Get the modified date from FileInfo
            const modifiedDate = fileStat.mtime || fileStat.birthtime;
            
            // Only update properties if they've changed to avoid Vue re-renders
            const existingFile = folderFiles.value[fileIndex];
            
            // Update size if changed
            if (existingFile.rawSize !== fileStat.size) {
              existingFile.size = formatFileSize(fileStat.size);
              existingFile.rawSize = fileStat.size;
            }
            
            // Update date if changed
            if (modifiedDate instanceof Date && 
                (!existingFile.dateObject instanceof Date || 
                 existingFile.dateObject.getTime() !== modifiedDate.getTime())) {
              existingFile.modified = modifiedDate.toLocaleString();
              existingFile.dateObject = modifiedDate;
            }
          } else {
            // If file is not in the list yet, add just this file without a full reload
            const newFile = {
              name: entry.name,
              path: actualPath,
              size: formatFileSize(fileStat.size),
              modified: modifiedDate instanceof Date ? modifiedDate.toLocaleString() : 'Unknown',
              rawSize: fileStat.size,
              dateObject: modifiedDate,
              key: entry.name
            };
            
            // Insert at the beginning (newest files first)
            folderFiles.value.unshift(newFile);
          }
          
          // Update the actual recording filename to match what's on disk
          recordingFilename.value = entry.name;
          break;
        } catch (error) {
          console.error(`Error getting stats for ${entry.name}:`, error);
        }
      }
    }
  } catch (error) {
    console.error('Error finding active recording file:', error);
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
    
    // Start Android foreground service to keep app alive in background
    try {
      await invoke('plugin:android-forward-service|start_forward_service');
      console.log('Android foreground service started');
    } catch (e) {
      console.warn('Forward service not available or failed to start:', e);
    }
    
    isRecording.value = true;
    
    // Set up a watcher for the recording file
    await setupRecordingFileWatcher();
    
    // Reload directory to ensure the new file shows up
    setTimeout(async () => {
      await loadDirectoryFiles();
    }, 1000); // Give the file system a moment to create the file
  } catch (error) {
    console.error('Error starting recording:', error);
    alert(`Failed to start recording: ${error}`);
  }
}

// Open file in file explorer or viewer
async function openFileLocation(filePath) {
  try {
    // On desktop platforms, show the file in folder
    if (platform() !== 'android' && platform() !== 'ios') {
      await invoke('plugin:shell|open', { path: filePath });
    } else {
      // On mobile, we might need a different approach
      // Possibly using a Tauri plugin or a mobile-specific way to share files
      alert('Opening files directly not supported on this platform yet');
    }
  } catch (error) {
    console.error('Error opening file location:', error);
    alert(`Unable to open file: ${error}`);
  }
}

// Upload file to a remote server or cloud storage
async function uploadFile(filePath) {
  try {
    // This is a placeholder for actual upload functionality
    // In a real implementation, you would:
    // 1. Read the file content
    // 2. Upload to a server/cloud storage using an API
    // 3. Show progress and status
    
    // For now, we'll just show a notification
    alert(`File upload feature will be implemented in a future version.\nSelected file: ${filePath}`);
    
    // Example of how it might be implemented with a Tauri command:
    // await invoke('upload_file_to_server', { 
    //   filePath, 
    //   destination: 'cloud_storage',
    //   credentials: { /* auth tokens, etc */ }
    // });
  } catch (error) {
    console.error('Error uploading file:', error);
    alert(`Failed to upload file: ${error}`);
  }
}

// Delete file from the filesystem
async function deleteFile(filePath) {
  try {
    // Confirm deletion
    const confirmed = confirm(`Are you sure you want to delete ${filePath.split('/').pop()}?`);
    if (!confirmed) return;
    
    // Use Tauri FS plugin to remove the file
    // await invoke('plugin:fs|remove_file', { path: filePath });
    
    // Refresh the file list
    await loadDirectoryFiles();
  } catch (error) {
    console.error('Error deleting file:', error);
    alert(`Failed to delete file: ${error}`);
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
      await fileWatchUnsubscribe.value();
      fileWatchUnsubscribe.value = null;
    }
    
    // Clear the active recording path
    activeRecordingPath.value = '';
    
    isRecording.value = false;
    recordingFilename.value = '';
    
    // Final refresh of directory contents
    await loadDirectoryFiles();
  } catch (error) {
    console.error('Error stopping recording:', error);
    alert(`Failed to stop recording: ${error}`);
  }
}
</script>

<style scoped>
/* Animations for recording indicator */
.recording-pulse {
  animation: pulse 2s infinite;
  box-shadow: 0 0 15px rgba(16, 185, 129, 0.5);
}

@keyframes pulse {
  0% {
    box-shadow: 0 0 5px rgba(16, 185, 129, 0.5);
  }
  50% {
    box-shadow: 0 0 25px rgba(16, 185, 129, 0.8);
  }
  100% {
    box-shadow: 0 0 5px rgba(16, 185, 129, 0.5);
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
