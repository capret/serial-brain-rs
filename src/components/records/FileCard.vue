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
import { formatDate, formatTime, formatDuration, formatFileSize } from '../../utils/records/formatters';
import type { RecordingFile } from '../../utils/records/types';

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

// For video check interval
let videoCheckInterval: number | undefined = undefined;

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

// Set up the live duration updating only for active recordings
onMounted(async () => {
  // Only set up live updates if this is an active recording
  if (props.isActiveRecording) {
    setupLiveDurationUpdates();
  }
  
  // Check if a corresponding video file exists
  await checkVideoExists();
});

// Clean up on unmount
onUnmounted(() => {
  // Clean up duration update interval
  if (updateInterval) {
    clearInterval(updateInterval);
    updateInterval = undefined;
  }
  
  // Clean up video check interval
  if (videoCheckInterval) {
    clearInterval(videoCheckInterval);
    videoCheckInterval = undefined;
  }
});

// Watch for changes in the active recording status
watch(() => props.isActiveRecording, (isActive: boolean) => {
  // Clear any existing intervals first
  if (videoCheckInterval) {
    clearInterval(videoCheckInterval);
    videoCheckInterval = undefined;
  }
  
  if (isActive) {
    // Start the duration updates
    setupLiveDurationUpdates();
    
    // For active recordings, check for video immediately and then periodically
    checkVideoExists();
    
    // Set up interval to check for video every 3 seconds for active recordings
    videoCheckInterval = setInterval(async () => {
      await checkVideoExists();
    }, 3000) as unknown as number;
  } else {
    // Clean up duration update interval
    if (updateInterval) {
      clearInterval(updateInterval);
      updateInterval = undefined;
    }
  }
});

// Extract recording start time from filename
function extractDurationFromFilename(file: RecordingFile): string | null {
  const match = file.name.match(/serial_recording_(\d+)/);
  if (!match || !match[1]) return null;
  
  const startTimestamp = parseInt(match[1]);
  if (isNaN(startTimestamp)) return null;
  
  let endTimestamp: number;
  if (props.isActiveRecording) {
    // For active recordings, use current time
    endTimestamp = Date.now();
  } else if (file.dateObject instanceof Date) {
    // For completed recordings, use modified time
    endTimestamp = file.dateObject.getTime();
  } else {
    return null;
  }
  
  const durationMs = endTimestamp - startTimestamp;
  if (durationMs <= 0) return null;
  
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
    }
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
    
    // For active recordings, try multiple times with short delays
    // Sometimes the video file is created after the CSV file
    if (props.isActiveRecording && !hasVideo.value) {
      // Try up to 3 times with increasing delays for active recordings
      for (let attempt = 0; attempt < 3; attempt++) {
        const videoExists = await exists(videoPath);
        
        if (videoExists) {
          hasVideo.value = true;
          console.log(`Video found for active recording ${props.file.name} on attempt ${attempt + 1}`);
          break;
        } else if (attempt < 2) { // Don't wait after the last attempt
          // Increase delay with each attempt (200ms, 400ms, 600ms)
          await new Promise(resolve => setTimeout(resolve, 200 * (attempt + 1)));
        }
      }
    } else {
      // Standard check for non-active recordings
      const videoExists = await exists(videoPath);
      if (hasVideo.value !== videoExists) {
        hasVideo.value = videoExists;
        console.log(`Video check for ${props.file.name}: ${hasVideo.value ? 'has video' : 'no video'}`);
      }
    }
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
