<template>
  <div 
    :class="[
      'bg-gray-700 hover:bg-gray-600 p-4 rounded-lg transition-all duration-300 transform hover:scale-[1.02] flex flex-col relative',
      {'recording-pulse': isActiveRecording}
    ]">
    
    <!-- File card header with name and actions -->
    <div class="flex flex-wrap justify-between items-start gap-2 mb-3">
      <div class="flex items-center gap-2 flex-1 min-w-0">
        <!-- File icon -->
        <div class="text-blue-400">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
            <polyline points="14 2 14 8 20 8"></polyline>
            <line x1="16" y1="13" x2="8" y2="13"></line>
            <line x1="16" y1="17" x2="8" y2="17"></line>
            <polyline points="10 9 9 9 8 9"></polyline>
          </svg>
        </div>
        
        <!-- File name -->
        <h4 class="font-medium text-sm truncate w-full" :title="file.name">
          {{ file.name }}
        </h4>
      </div>
      
      <!-- Action buttons -->
      <div class="flex flex-wrap gap-1 text-gray-400">
        <button @click.stop="openFile" class="p-1 hover:text-white" title="Open file">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
            <polyline points="15 3 21 3 21 9"></polyline>
            <line x1="10" y1="14" x2="21" y2="3"></line>
          </svg>
        </button>
        <button @click.stop="uploadFile" class="p-1 hover:text-white" title="Upload file">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
            <polyline points="17 8 12 3 7 8"></polyline>
            <line x1="12" y1="3" x2="12" y2="15"></line>
          </svg>
        </button>
        <button @click.stop="confirmDelete" class="p-1 hover:text-red-500" title="Delete file">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="3 6 5 6 21 6"></polyline>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
            <line x1="10" y1="11" x2="10" y2="17"></line>
            <line x1="14" y1="11" x2="14" y2="17"></line>
          </svg>
        </button>
      </div>
    </div>
    
    <!-- Video badge (only shown if we've checked) -->  
    <div v-if="hasVideo !== null" class="mb-2 flex justify-start">
      <span v-if="hasVideo" class="px-2 py-1 text-xs rounded-full bg-blue-600 text-white font-medium mr-2">
        {{ $t('recording.video') }}
      </span>
      <span v-else class="px-2 py-1 text-xs rounded-full bg-gray-600 text-white font-medium mr-2">
        {{ $t('recording.noVideo') }}
      </span>
    </div>
    
    <!-- File metadata -->
    <div class="flex justify-between text-xs text-gray-400 mb-2">
      <div>{{ $t('recording.size') }}: <span class="text-gray-300" :class="{'text-green-300': isActiveRecording}">{{ file.size }}</span></div>
      <div>{{ $t('recording.duration') }}: <span class="text-gray-300" :class="{'text-green-300': isActiveRecording}">{{ liveDuration }}</span></div>
    </div>
    
    <!-- Created date and time -->
    <div class="text-xs text-gray-400">
      {{ $t('recording.created') }}: <span class="text-gray-300">{{ formattedDate }} {{ formattedTime }}</span>
    </div>
    
    <!-- Active recording indicator - moved to bottom right -->
    <div v-if="isActiveRecording" class="absolute bottom-0 right-0 mb-1 mr-1 flex items-center gap-2">
      <!-- Recording badge -->
      <!-- <span class="text-xs px-2 py-1 rounded bg-red-600 text-white font-bold shadow-md shadow-red-500/50">
        RECORDING
      </span> -->
      <!-- Pulsing indicator -->
      <span class="relative flex h-3 w-3">
        <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
        <span class="relative inline-flex rounded-full h-3 w-3 bg-green-500"></span>
      </span>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref, onMounted, onUnmounted, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { exists } from '@tauri-apps/plugin-fs';
import type { RecordingFile } from '../../utils/records/types';
import { formatDate, formatTime, formatDuration, formatFileSize } from '../../utils/records/formatters';
import { lastCompletedFilename } from '../../utils/records/fileStore';

// Define props
const props = defineProps({
  file: {
    type: Object as () => RecordingFile,
    required: true
  },
  isActiveRecording: {
    type: Boolean,
    default: false
  }
});

// Define emits
const emit = defineEmits(['action', 'update-file-size']);

// Initialize i18n
const { t } = useI18n();

// Computed properties
const formattedDate = computed(() => formatDate(props.file));
const formattedTime = computed(() => formatTime(props.file));

// For live duration updating during recording
const startTime = ref<number | null>(null);
const currentDuration = ref<string | null>(null);
let updateInterval: number | undefined = undefined;

// Store listeners for cleanup
const listeners: Array<() => void> = [];

// Video badge state
const hasVideo = ref<boolean | null>(null); // null = not checked yet, true/false = has/doesn't have video

// Computed property that shows either the static duration or the live updated one
const liveDuration = computed(() => {
  // For active recordings, use our live-updating duration
  if (props.isActiveRecording && currentDuration.value) {
    return currentDuration.value;
  }
  // For completed recordings, use the stored duration if available
  if (props.file.duration) {
    return props.file.duration;
  }
  // If we have no duration info yet, extract it from the filename
  return extractDurationFromFilename(props.file) || '—';
});

// Set up the live duration updating for active recordings
onMounted(() => {
  // Set up live updates for active recordings
  setupLiveDurationUpdates();
  
  // Check if a corresponding video file exists
  checkVideoExists();
  
    // Watch the lastCompletedFilename from fileStore to detect completed recordings
  watch(lastCompletedFilename, (newCompletedFilename) => {
    // Skip if empty
    if (!newCompletedFilename) return;
    
    // Check if this card represents the file that just completed
    if (props.file.name === newCompletedFilename) {
      console.log(`This file just completed rotation: ${newCompletedFilename}`);
      
      // Force duration recalculation immediately since fileStore has already waited
      try {
        // Re-extract duration from filename using the latest file metadata
        const extractedDuration = extractDurationFromFilename(props.file);
        if (extractedDuration) {
          currentDuration.value = extractedDuration;
          console.log(`Updated duration for completed recording ${props.file.name}: ${currentDuration.value}`);
        }
      } catch (error) {
        console.error('Error updating completed recording duration:', error);
      }
    }
  });
});

// Clean up on unmount
onUnmounted(() => {
  // Clear the update interval
  if (updateInterval) {
    clearInterval(updateInterval);
    updateInterval = undefined;
  }
  
  // Clean up all event listeners
  listeners.forEach(unlisten => unlisten());
});

// Watch for changes in the active recording status
// Watch for active recording state changes
watch(() => props.isActiveRecording, (isActive: boolean) => {
  if (isActive) {
    setupLiveDurationUpdates();
  } else if (updateInterval) {
    clearInterval(updateInterval);
    updateInterval = undefined;
  }
});

// Watch for file changes to update duration calculation
watch(() => props.file, (newFile) => {
  console.log(`File changed: ${newFile.name}`);
  
  // Reset internal state
  startTime.value = null;
  currentDuration.value = null;
  
  // If this is an active recording, setup the live updates immediately
  if (props.isActiveRecording) {
    console.log(`Setting up live duration updates for active recording: ${newFile.name}`);
    setupLiveDurationUpdates();
  } else {
    // For non-active recordings, calculate duration once
    console.log(`Calculating one-time duration for completed recording: ${newFile.name}`);
    const extractedDuration = extractDurationFromFilename(newFile);
    if (extractedDuration) {
      currentDuration.value = extractedDuration;
    }
  }
  
  // Check if video exists for this file
  checkVideoExists();
}, { immediate: true });

// Extract recording start time from filename
function extractDurationFromFilename(file: RecordingFile): string | null {
  // Extract timestamp from filename
  const match = file.name.match(/serial_recording_(\d+)/);
  if (!match || !match[1]) return null;
  
  const startTimestamp = parseInt(match[1]);
  if (isNaN(startTimestamp)) return null;
  
  let endTimestamp: number;
  if (props.isActiveRecording) {
    // For active recordings, use current time
    endTimestamp = Date.now();
  } else if (file.dateObject instanceof Date) {
    // For completed recordings, prioritize modified time from file stats
    endTimestamp = file.dateObject.getTime();
    
    // Perform a sanity check - if the calculated duration is too short (less than a second),
    // something might be wrong with the timestamps. In this case, try to use file size to estimate duration.
    const durationMs = endTimestamp - startTimestamp;
    if (durationMs < 1000 && file.rawSize) {
      // Roughly estimate duration based on file size (assuming ~1KB per second as a minimum)
      // This is a fallback for when the timestamp approach fails
      const estimatedDurationMs = Math.max(1000, file.rawSize / 1024 * 1000);
      console.log(`Using estimated duration for ${file.name}: ${formatDuration(estimatedDurationMs)} based on file size ${file.rawSize} bytes`);
      return formatDuration(estimatedDurationMs);
    }
  } else {
    return null;
  }
  
  const durationMs = endTimestamp - startTimestamp;
  if (durationMs <= 0) {
    // If we still have an invalid duration, use at least 1 second
    console.log(`Invalid duration for ${file.name}, using minimum 1 second`);
    return formatDuration(1000); // Minimum 1 second
  }
  
  console.log(`Calculated duration for ${file.name}: ${formatDuration(durationMs)}`);
  return formatDuration(durationMs);
}

// Set up interval to update duration in real-time for active recordings
function setupLiveDurationUpdates(): void {
  // Parse start time from filename
  const match = props.file.name.match(/serial_recording_(\d+)/);
  if (match && match[1]) {
    const timestamp = parseInt(match[1]);
    if (!isNaN(timestamp)) {
      startTime.value = timestamp;
      
      // Calculate duration immediately instead of waiting for first interval
      const now = Date.now();
      const initialDurationMs = now - timestamp;
      currentDuration.value = formatDuration(initialDurationMs);
      console.log(`Initial duration for ${props.file.name}: ${currentDuration.value}`);
    } else {
      console.warn(`Invalid timestamp in filename: ${props.file.name}`);
      // Set a fallback duration (current time minus 1 second) to avoid showing 00:00:01
      currentDuration.value = formatDuration(1000); // Start with at least 1 second
    }
  } else {
    console.warn(`Could not extract timestamp from filename: ${props.file.name}`);
    // Set a fallback duration
    currentDuration.value = formatDuration(1000); // Start with at least 1 second
  }
  
  // Clear any existing interval
  if (updateInterval) {
    clearInterval(updateInterval);
  }
  
  // Set up interval to update duration and file size
  updateInterval = setInterval(async () => {
    if (startTime.value) {
      // Update duration
      const now = Date.now();
      const durationMs = now - startTime.value;
      currentDuration.value = formatDuration(durationMs);
      
      // For active recordings, check file size directly from backend
      if (props.isActiveRecording) {
        try {
          // Define FileStats interface for Tauri response
          interface FileStats {
            size: number;
            modified: number;
            created: number;
          }
          
          // Get real-time stats from the file
          const stats = await invoke<FileStats>('get_file_stats', { 
            path: props.file.path 
          });
          
          if (stats && stats.size) {
            // Update size if it's changed
            if (stats.size !== props.file.rawSize) {
              // Emit an event to parent to update this file
              emit('update-file-size', {
                path: props.file.path,
                size: stats.size,
                formattedSize: formatFileSize(stats.size)
              });
            }
          }
        } catch (error) {
          console.warn('Error updating file size:', error);
        }
      }
    }
  }, 1000) as unknown as number; // Update every second
}

// Methods for handling actions
function openFile() {
  emit('action', { action: 'open', file: props.file });
}

function uploadFile() {
  emit('action', { action: 'upload', file: props.file });
}

function confirmDelete() {
  // We need to add the confirm delete translation to our language files
  if (confirm(`${t('recording.confirmDeletePrefix')} ${props.file.name}?`)) {
    emit('action', { action: 'delete', file: props.file });
  }
}

// Check if a corresponding video file exists
async function checkVideoExists() {
  try {
    // Get base file path without extension
    const filePath = props.file.path;
    const basePath = filePath.replace(/\.[^/.]+$/, '');
    
    // Check if MP4 file exists
    const videoPath = `${basePath}.mp4`;
    const videoExists = await exists(videoPath);
    
    hasVideo.value = videoExists;
    console.log(`Video check for ${props.file.name}: ${hasVideo.value ? 'has video' : 'no video'}`);
  } catch (error) {
    console.error('Error checking for video file:', error);
    hasVideo.value = false;
  }
}
</script>

<style scoped>
/* Animations for recording indicator */
.recording-pulse {
  background-color: rgba(16, 185, 129, 0.15) !important;
  border: 1px solid #10b981 !important;
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
</style>
