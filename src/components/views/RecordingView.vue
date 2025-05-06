<template>
  <div class="bg-gray-800 bg-opacity-60 rounded-lg p-6">
    <!-- Recording Setup Section -->
    <RecordingSetup
      :is-recording="isRecording"
      :recording-directory="recordingDirectory"
      :connection-status="connectionStatus"
      @start-recording="startRecording"
      @stop-recording="stopRecording"
    />

    <div class="space-y-6">
      <!-- Storage Location is included in RecordingSetup -->
      
      <!-- Recording Format Section -->
      <RecordingFormatSelector
        v-model="recordingFormat"
        :disabled="isRecording"
      />
      
      <!-- Recording Options Section -->
      <RecordingOptions
        v-model:autostart="autoStartRecording"
        v-model:max-recording-duration="maxRecordingDuration"
        :disabled="isRecording"
      />
      
      <!-- Recording Status Section -->
      <RecordingStatus
        v-if="isRecording"
        :is-recording="isRecording"
        :recording-filename="recordingFilename"
      />
    </div>
    
    <div class="border-t border-gray-700 my-8"></div>
    
    <!-- Recorded Files Section -->
    <RecordedFilesList
      :recording-directory="recordingDirectory"
      :is-recording="isRecording"
      :recording-filename="recordingFilename"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import * as path from '@tauri-apps/api/path';
import { 
  recordingDirectory, 
  connectionStatus, 
  maxRecordingDuration,
  recordingFormat,
  autoStartRecording
} from '../../store/appState';
import { 
  formatDuration,
  setupRecordingFileWatcher,
  findAndUpdateActiveRecordingFile,
  UnsubscribeFn
} from '../../utils/records';

// Import the components
import { 
  RecordingSetup,
  RecordingFormatSelector,
  RecordingOptions,
  RecordingStatus,
  RecordedFilesList
} from '../records';

// State variables for recording
const isRecording = ref<boolean>(false);
const recordingFilename = ref<string>('');
const activeRecordingPath = ref<string>('');
const fileWatchUnsubscribe = ref<UnsubscribeFn | null>(null);

// Event listener unsubscribe function
let recordingCompletedUnsubscribe: (() => void) | null = null;

// Listen for recording-completed events from the backend
async function setupEventListeners() {
  // Clean up previous listener if it exists
  if (recordingCompletedUnsubscribe) {
    recordingCompletedUnsubscribe();
    recordingCompletedUnsubscribe = null;
  }
  
  // Listen for when a recording completes due to reaching max duration
  recordingCompletedUnsubscribe = await listen('recording-completed', async (event) => {
    console.log('Recording completed event received:', event);
    const payload = event.payload as {
      format: string;
      directory: string;
      maxDurationMinutes: number;
      shouldRestartRecording: boolean;
    };
    
    // Only restart if shouldRestartRecording is true
    if (payload.shouldRestartRecording) {
      console.log('Auto-starting new recording...');
      
      // Update UI state to show recording is temporarily stopped
      isRecording.value = false;
      
      // Start a new recording with the same parameters
      try {
        // Small delay to ensure the previous recording is properly closed
        setTimeout(async () => {
          await startRecording(payload.format, payload.directory, payload.maxDurationMinutes);
        }, 500);
      } catch (error) {
        console.error('Failed to auto-restart recording:', error);
        alert(`Failed to auto-restart recording: ${error}`);
      }
    }
  });
}

// Reference to the update interval for active recordings
let activeUpdateInterval: number | undefined = undefined;

// Function to set up periodic updates for active recording files
function setupActiveRecordingUpdates() {
  // Clear any existing interval
  if (activeUpdateInterval) {
    clearInterval(activeUpdateInterval);
    activeUpdateInterval = undefined;
  }
  
  // Only set up if recording is active
  if (isRecording.value) {
    console.log('Setting up periodic updates for active recording');
    activeUpdateInterval = setInterval(async () => {
      if (isRecording.value) {
        console.log('Checking active recording status');
        try {
          // Get the latest filename from the backend to ensure we stay in sync
          const currentFilename = await invoke('get_recording_filename') as string;
          if (currentFilename && currentFilename !== recordingFilename.value) {
            console.log('Updating recording filename from backend:', currentFilename);
            recordingFilename.value = currentFilename;
          }
          
          // Check the file size of the current recording
          if (recordingFilename.value && recordingDirectory.value) {
            const fullPath = `${recordingDirectory.value}/${recordingFilename.value}`;
            try {
              // Get the actual file stats to update size
              interface FileStats {
                size: number; 
                modified: number;
                created: number;
              }
              const fileStat = await invoke<FileStats>('get_file_stats', { path: fullPath });
              if (fileStat && fileStat.size) {
                console.log(`Current recording file size: ${fileStat.size} bytes`);
                // Trigger a file list refresh to update the size
                await findAndUpdateActiveRecordingFile(
                  isRecording.value,
                  recordingDirectory.value,
                  recordingFilename.value,
                  recordingFormat.value,
                  [],
                  (path) => activeRecordingPath.value = path,
                  (name) => recordingFilename.value = name,
                  () => {} // We'll handle the UI update separately
                );
              }
            } catch (error) {
              console.warn('Error checking file stats:', error);
            }
          }
        } catch (e) {
          console.error('Error updating recording filename:', e);
        }
      } else {
        // Stop interval if recording is no longer active
        console.log('Recording stopped, clearing update interval');
        clearInterval(activeUpdateInterval);
        activeUpdateInterval = undefined;
      }
    }, 1000) as unknown as number; // Update every second for more responsive updates
  }
}

// Initialize recording directory and check recording status when component mounts
onMounted(async () => {
  try {
    // First, set up the recordings directory in AppData
    await selectDirectory();
    const status = await invoke('get_recording_status');
    isRecording.value = status as boolean;
    
    // Set up event listeners
    await setupEventListeners();
    
    // If recording is active, get the current recording filename from the backend
    if (isRecording.value) {
      // Get the current recording filename from the backend
      try {
        const filename = await invoke('get_recording_filename') as string;
        if (filename) {
          console.log('Retrieved current recording filename from backend:', filename);
          recordingFilename.value = filename;
          
          // Set up periodic updates for the active recording
          setupActiveRecordingUpdates();
        }
      } catch (error) {
        console.error('Failed to get recording filename:', error);
      }
    }
  } catch (error) {
    console.error('Error initializing component:', error);
  }
});

// Clean up when component unmounts
onUnmounted(() => {
  // NOTE: We no longer stop recording when navigating away from this page
  // This allows recording to continue while using other app features

  // Clean up file watcher if active
  if (fileWatchUnsubscribe.value) {
    fileWatchUnsubscribe.value();
    fileWatchUnsubscribe.value = null;
  }
  
  // Remove event listeners
  if (recordingCompletedUnsubscribe) {
    recordingCompletedUnsubscribe();
    recordingCompletedUnsubscribe = null;
  }
  
  // Clean up the update interval for active recordings
  if (activeUpdateInterval) {
    clearInterval(activeUpdateInterval);
    activeUpdateInterval = undefined;
  }
});

// Use AppData directory for recordings
async function selectDirectory(): Promise<void> {
  const home = await path.documentDir();
  recordingDirectory.value = home;
  console.log('Using documents directory for recordings:', home);
}

async function startRecording(format?: string, directory?: string, duration?: number): Promise<void> {
  // Use provided parameters or defaults from UI controls
  const recordFormat = format || recordingFormat.value;
  const recordDir = directory || recordingDirectory.value;
  const recordDuration = duration || maxRecordingDuration.value;
  
  if (!recordDir) {
    alert('Please select a directory to save recordings');
    return;
  }
  
  try {
    console.log('Starting recording with format:', recordFormat);
    console.log('Directory:', recordDir);
    console.log('Duration:', recordDuration);
    
    const actualFilename = await invoke('start_recording', {
      format: recordFormat,
      directory: recordDir,
      maxDurationMinutes: recordDuration,
      autoStart: autoStartRecording.value
    }) as string;
    
    console.log('Received filename from backend:', actualFilename);
    recordingFilename.value = actualFilename;
    
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
          recordingFormat.value,
          [],
          (path) => activeRecordingPath.value = path,
          (name) => recordingFilename.value = name,
          () => {} // No need to update files here as that's handled by RecordedFilesList
        );
      },
      fileWatchUnsubscribe.value
    );
    
  } catch (error) {
    console.error('Error starting recording:', error);
    alert(`Failed to start recording: ${error}`);
  }
}

async function stopRecording(): Promise<void> {
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
    
    activeRecordingPath.value = '';
    isRecording.value = false;
    recordingFilename.value = '';
  } catch (error) {
    console.error('Error stopping recording:', error);
    alert(`Failed to stop recording: ${error}`);
  }
}
</script>
