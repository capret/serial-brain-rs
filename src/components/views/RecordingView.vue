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
import { ref, onMounted, onUnmounted, inject } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { exists, mkdir, BaseDirectory } from '@tauri-apps/plugin-fs';
import * as path from '@tauri-apps/api/path';
import { 
  recordingDirectory, 
  connectionStatus, 
  maxRecordingDuration,
  recordingFormat,
  autoStartRecording,
  isConnected,
  fetchConnectionState,
  streamingActive,
  toggleStreamingState
} from '../../store/appState';
import fileStore from '../../utils/records/fileStore';
import { formatFileSize } from '../../utils/records/formatters';
import {
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

// For changing sidebar view
const setActiveView = inject<(view: string) => void>('setActiveView');
const setAdditionalViews = inject<(view: string) => void>('setAdditionalViews');


// Define interfaces for backend response types
interface RecordingStatusResponse {
  isRecording: boolean;
  filename: string;
}

// Initialize i18n
useI18n();

// State variables for recording
const isRecording = ref<boolean>(false);
const recordingFilename = ref<string>('');
const activeRecordingPath = ref<string>('');
const fileWatchUnsubscribe = ref<UnsubscribeFn | null>(null);

// Track if the Android service is running
const isServiceRunning = ref<boolean>(false);

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
      baseFilename?: string; // New parameter for video recording segments
    };
    
    // Only restart if shouldRestartRecording is true
    if (payload.shouldRestartRecording) {
      console.log('Auto-starting new recording segment...');
      
      // Store the current recording filename before changing it
      const previousFilename = recordingFilename.value;
      
      // Temporarily set recording state to false
      // but we'll keep the Android service running
      isRecording.value = false;
      
      // First explicitly stop the video recording to ensure it's properly finalized
      try {
        console.log('Stopping video recording for segment change...');
        await invoke('stop_video_recording');
        console.log('Video recording stopped for segment change');
      } catch (error) {
        console.warn('Error stopping video recording for segment change:', error);
        // Continue with new segment even if stopping video fails
      }
      
      // Explicitly update the metadata for the previous segment file
      if (previousFilename) {
        try {
          console.log(`Updating metadata for previous segment: ${previousFilename}`);
          // Import fileStore dynamically to avoid circular dependencies
          const fileStore = (await import('../../utils/records/fileStore')).default;
          
          // Explicitly update the metadata of the previous segment file
          if (recordingDirectory.value) {
            const previousFilePath = `${recordingDirectory.value}/${previousFilename}`;
            
            // Wait a moment to ensure the file is fully written to disk
            await new Promise(resolve => setTimeout(resolve, 500));
            
            // Get the file stats manually to calculate duration
            try {
              const fs = await import('@tauri-apps/plugin-fs');
              const stats = await fs.stat(previousFilePath);
              
              if (stats) {
                console.log(`Previous segment file stats: created=${new Date(stats.birthtime || 0).toISOString()}, modified=${new Date(stats.mtime || 0).toISOString()}`);
                
                // Calculate duration directly using file timestamps
                if (stats.birthtime && stats.mtime) {
                  const createdTime = new Date(stats.birthtime).getTime();
                  const modifiedTime = new Date(stats.mtime).getTime();
                  const durationSec = Math.max(Math.floor((modifiedTime - createdTime) / 1000), 2); // At least 2 seconds
                  
                  console.log(`Calculated duration for previous segment: ${durationSec} seconds`);
                }
              }
            } catch (statsError) {
              console.warn(`Error getting file stats: ${statsError}`);
            }
            
            // Now update metadata through fileStore
            await fileStore.updateSingleFileMetadata(previousFilePath);
            console.log(`Metadata updated for previous segment: ${previousFilename}`);
            
            // Force a complete refresh of the file list to update UI
            await fileStore.loadFiles();
          }
        } catch (error) {
          console.warn(`Error updating previous segment metadata: ${error}`);
        }
      }
      
      // Start a new recording segment with the same parameters
      // but don't restart the Android service
      try {
        // Small delay to ensure the previous recording is properly closed
        setTimeout(async () => {
          await startRecording(payload.format, payload.directory, payload.maxDurationMinutes, true);
        }, 800); // Increased delay to give more time for file updates
      } catch (error) {
        console.error('Failed to auto-restart recording:', error);
        alert(`Failed to auto-restart recording: ${error}`);
      }
    }
  });
  
  // Listen for recording filename changes (especially important for segment changes)
  await listen('recording-filename-changed', async (event) => {
    const newFilename = event.payload as string;
    console.log('Recording filename changed to:', newFilename);
    
    // Store the previous filename before updating it
    const previousFilename = recordingFilename.value;
    
    // Update to the new filename
    recordingFilename.value = newFilename;
    
    // If we had a previous filename and we're still recording, this is likely a segment change
    // We need to update the previous segment's duration
    if (previousFilename && previousFilename !== newFilename && isRecording.value) {
      console.log(`Segment change detected: ${previousFilename} -> ${newFilename}`);
      
      try {
        // Import fileStore dynamically to avoid circular dependencies
        const fileStore = (await import('../../utils/records/fileStore')).default;
        
        // Update the previous segment's metadata
        if (recordingDirectory.value) {
          const previousFilePath = `${recordingDirectory.value}/${previousFilename}`;
          
          // Short delay to ensure file is completely written
          await new Promise(resolve => setTimeout(resolve, 300));
          
          console.log(`Updating metadata for previous segment: ${previousFilename}`);
          await fileStore.updateSingleFileMetadata(previousFilePath);
          console.log(`Metadata updated for previous segment: ${previousFilename}`);
          
          // Refresh file list to update UI
          await fileStore.loadFiles();
        }
      } catch (error) {
        console.warn(`Error updating previous segment metadata: ${error}`);
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
                
                // Update the file store with the latest recording information
                // This ensures the file exists in the list and has the current state
                await fileStore.updateRecordingState(true, recordingFilename.value);
                
                // Update the file size in the store
                fileStore.updateFileSize(fullPath, fileStat.size, formatFileSize(fileStat.size));
                
                // Trigger a file list refresh to update the size
                await findAndUpdateActiveRecordingFile(
                  isRecording.value,
                  recordingDirectory.value,
                  recordingFilename.value,
                  recordingFormat.value,
                  fileStore.files.value, // Pass current files instead of empty array
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
    
    // Fetch the connection state from the backend using our shared function
    try {
      console.log('Fetching connection status from backend...');
      await fetchConnectionState();
      
      // Update the connection status based on isConnected
      // This ensures consistency with the other views
      connectionStatus.value = isConnected.value ? 'connected' : 'disconnected';
      console.log('Updated connection status to:', connectionStatus.value);
    } catch (error) {
      console.warn('Failed to get connection status from backend:', error);
    }
    
    // Check if we're already recording when navigating to this page
    try {
      const recordingStatus = await invoke('get_app_state', {
        category: 'recording',
        key: 'status'
      }) as RecordingStatusResponse;
      
      if (recordingStatus) {
        isRecording.value = recordingStatus.isRecording;
        recordingFilename.value = recordingStatus.filename;
        
        console.log('Recording status from backend:', isRecording.value ? 'Active' : 'Inactive');
        if (recordingFilename.value) {
          console.log('Current recording filename:', recordingFilename.value);
        }
      }
    } catch (error) {
      console.error('Failed to get recording status:', error);
    }
    
    // Set up event listeners
    await setupEventListeners();
    
    // If recording is active, set up file watcher and periodic updates
    if (isRecording.value && recordingFilename.value) {
      // Set up a watcher for the recording file
      const fullPath = `${recordingDirectory.value}/${recordingFilename.value}`;
      fileWatchUnsubscribe.value = await setupRecordingFileWatcher(
        fullPath,
        async () => {
          // When file changes, update it without triggering a full reload
          await findAndUpdateActiveRecordingFile(
            true,
            recordingDirectory.value,
            recordingFilename.value,
            recordingFormat.value,
            [],
            (path) => activeRecordingPath.value = path,
            (name) => recordingFilename.value = name,
            () => {} // No need to update files here
          );
        },
        null
      );
      
      // Set up periodic updates for the active recording
      setupActiveRecordingUpdates();
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

// Use Document directory with signal_data folder for recordings
async function selectDirectory(): Promise<void> {
  try {
    // Get the document directory full path
    const documentDir = await path.documentDir();
    
    // Check if signal_data directory exists
    const folderExists = await exists('signal_data', {
      baseDir: BaseDirectory.Document,
    });
    
    // Create the directory if it doesn't exist
    if (!folderExists) {
      console.log('Creating signal_data directory in Documents');
      await mkdir('signal_data', {
        baseDir: BaseDirectory.Document,
      });
    }
    
    // Join the document dir with signal_data folder to get the full path
    const fullPath = await path.join(documentDir, 'signal_data');
    
    // Set the recording directory path to the full absolute path
    recordingDirectory.value = fullPath;
    console.log('Using full path for recordings:', fullPath);
  } catch (error) {
    console.error('Error setting up signal_data directory:', error);
    alert(`Failed to set up recordings directory: ${error}`);
  }
}

async function startRecording(format?: string, directory?: string, duration?: number, isSegmentChange: boolean = false): Promise<void> {
  // Check if camera is streaming - if not, toggle the streaming page on and start streaming
  try {
    // Use the shared streaming state to check if streaming is active
    if (!streamingActive.value) {
      console.log('Camera not streaming, activating streaming view and starting camera stream');
      
      // Toggle the streaming view on in the sidebar
      if (setActiveView && setAdditionalViews) {
        setAdditionalViews('streaming');
      }
      
      // Start streaming using the shared state function
      const streamUrlResponse = await invoke('get_app_state', {
        category: 'stream',
        key: 'default_stream_url'
      });
      const streamUrl = streamUrlResponse as string;
      if (streamUrl) {
        const success = await toggleStreamingState(true, streamUrl);
        if (success) {
          console.log('Streaming started successfully with URL:', streamUrl);
        } else {
          console.warn('Failed to start streaming, proceeding with recording anyway');
        }
      } else {
        console.warn('No default stream URL available, proceeding without camera');
      }
    } else {
      console.log('Camera already streaming, proceeding with recording');
    }
  } catch (error) {
    console.warn('Error checking streaming status:', error);
    // Continue with recording anyway
  }
  
  // Use provided parameters or defaults from UI controls
  const recordFormat = format || recordingFormat.value;
  const recordDir = directory || recordingDirectory.value; // This is now a full path
  const recordDuration = duration || maxRecordingDuration.value;
  
  if (!recordDir) {
    alert('Please select a directory to save recordings');
    return;
  }
  
  try {
    console.log('Starting recording with format:', recordFormat);
    console.log('Directory:', recordDir);
    console.log('Duration:', recordDuration);
    
    // Start both signal recording and video recording with the same base filename
    const actualFilename = await invoke('start_recording', {
      format: recordFormat,
      directory: recordDir,
      maxDurationMinutes: recordDuration,
      autoStart: autoStartRecording.value
    }) as string;
    
    console.log('Received filename from backend:', actualFilename);
    recordingFilename.value = actualFilename;
    
    // Start video recording with the same base name but mp4 extension
    try {
      // Extract base name without extension
      const baseFilename = actualFilename.replace(/\.[^/.]+$/, '');
      
      console.log(`Starting video recording for ${isSegmentChange ? 'segment change' : 'new recording'}`);
      console.log('Video base filename:', baseFilename);
      console.log('Video directory:', recordDir);
      
      // Start the video recording using the Tauri plugin command
      await invoke('start_video_recording', {
        filename: baseFilename,
        directory: recordDir
      });
      
      console.log('Video recording started successfully with base filename:', baseFilename);
    } catch (error) {
      console.warn('Failed to start video recording:', error);
      // Continue with signal recording even if video recording fails
    }
    
    // Only start the Android service if it's not already running
    if (!isServiceRunning.value && !isSegmentChange) {
      try {
        await invoke('plugin:android-forward-service|start_forward_service');
        console.log('Android foreground service started');
        isServiceRunning.value = true;
      } catch (e) {
        console.warn('Forward service not available or failed to start:', e);
      }
    } else if (isSegmentChange) {
      console.log('Segment change - not restarting Android service');
    } else {
      console.log('Android service already running - not restarting');
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
    if (isServiceRunning.value) {
      try {
        await invoke('plugin:android-forward-service|stop_forward_service');
        console.log('Android foreground service stopped');
        isServiceRunning.value = false;
      } catch (e) {
        console.warn('Forward service not available or failed to stop:', e);
      }
    }
    
    // Clean up file watcher if active
    if (fileWatchUnsubscribe.value) {
      fileWatchUnsubscribe.value();
      fileWatchUnsubscribe.value = null;
    }
    
    // Stop video recording
    try {
      await invoke('stop_video_recording');
      console.log('Video recording stopped');
      
      // Also update the shared streaming state to reflect that streaming has stopped
      if (streamingActive.value) {
        await toggleStreamingState(false);
        console.log('Shared streaming state updated to inactive');
      }
    } catch (error) {
      console.warn('Error stopping video recording:', error);
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
