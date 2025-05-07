<template>
  <div>
    <!-- Section header with title and refresh button -->
    <div class="flex justify-between items-center mb-6">
      <h2 class="text-3xl font-bold text-blue-400">Recorded Files</h2>
      <button 
        @click="refreshFiles" 
        class="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-md flex items-center gap-2">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" 
          stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
        </svg>
        Refresh
      </button>
    </div>
    
    <!-- Error message if any -->
    <div v-if="errorMessage" class="bg-red-500 text-white p-4 rounded-md mb-6">
      {{ errorMessage }}
    </div>
    
    <!-- Loading spinner -->
    <div v-if="isLoading" class="flex justify-center items-center py-8">
      <div class="animate-spin h-8 w-8 border-4 border-blue-500 rounded-full border-t-transparent"></div>
    </div>
    
    <!-- Empty state -->
    <div v-else-if="files.length === 0" class="bg-gray-700 p-6 rounded-md text-center">
      <p v-if="recordingDirectory">No recording files found in the selected directory.</p>
      <p v-else>Select a directory to view recorded files.</p>
    </div>
    
    <!-- File grid with transition animations -->
    <div v-else class="file-grid grid gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3">
      <transition-group name="filter" tag="div" class="contents">
        <FileCard 
          v-for="file in files" 
          :key="file.path" 
          :file="file"
          :is-active-recording="isFileActiveRecording(file)"
          @action="handleFileAction"
          @update-file-size="handleFileSizeUpdate"
        />
      </transition-group>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref, watch, onUnmounted, onMounted } from 'vue';
import FileCard from './FileCard.vue';
import { type RecordingFile } from '../../utils/records/types';
import { loadDirectoryFiles } from '../../utils/records/fileLoader';
import { syncFile, deleteFile, uploadFile } from '../../utils/records/fileOperations';
import { isActiveRecordingFile } from '../../utils/records/recordingFileHelpers';
import { watchImmediate, BaseDirectory } from '@tauri-apps/plugin-fs';
import { listen } from '@tauri-apps/api/event';

// Props
const props = defineProps({
  recordingDirectory: {
    type: String,
    required: true
  },
  isRecording: {
    type: Boolean,
    required: true
  },
  recordingFilename: {
    type: String,
    default: ''
  }
});

// State
const files = ref<RecordingFile[]>([]);
const isLoading = ref(false);
const errorMessage = ref<string>('');
const directoryWatchUnsubscribe = ref<(() => void) | null>(null);
let filenameChangeUnsubscribe: (() => void) | null = null;

// Watch for directory changes to set up watcher
watch(() => props.recordingDirectory, async (newPath) => {
  if (newPath) {
    // Unsubscribe from previous watcher if exists
    if (directoryWatchUnsubscribe.value) {
      directoryWatchUnsubscribe.value();
      directoryWatchUnsubscribe.value = null;
    }
    
    try {
      // We know we're watching the signal_data folder in Document directory
      // No need to extract it from the path - just use the hardcoded name
      const folderName = 'signal_data';
      
      console.log(`Setting up watcher for directory: ${folderName} (from full path: ${newPath})`);
      
      // Use watchImmediate for immediate file system events
      // We need to use just the folder name, not the full path
      const unsubscribeFunction = await watchImmediate(
        'signal_data', // Use the actual folder name, not the full path
        (event) => {
          console.log('Directory change event:', event);
          // Reload files when directory content changes
          loadFiles();
        },
        {
          baseDir: BaseDirectory.Document, // This tells Tauri to look in the Documents directory
          recursive: false // Only watch top-level files in the directory
        }
      );
      
      // Store the unsubscribe function
      directoryWatchUnsubscribe.value = unsubscribeFunction;
      
      // Load files initially
      await loadFiles();
      
      // Set up periodic refresh for active recordings
      if (props.isRecording) {
        console.log('Setting up periodic refresh for file list during recording');
        const refreshInterval = setInterval(() => {
          if (props.isRecording) {
            loadFiles();
          } else {
            clearInterval(refreshInterval);
          }
        }, 2000); // Refresh every 5 seconds during active recording
        
        // Clean up interval when component is unmounted
        onUnmounted(() => {
          clearInterval(refreshInterval);
        });
      }
    } catch (error) {
      console.error('Error setting up directory watcher:', error);
      errorMessage.value = `Error watching directory: ${error}`;
    }
  }
}, { immediate: true });

// Also watch recording status to refresh files when it changes
watch(() => props.isRecording, async (isRecording) => {
  if (isRecording) {
    // When recording starts, refresh files to capture new recording
    await loadFiles();
  } else {
    // When recording stops, refresh once more to update file list
    setTimeout(() => loadFiles(), 1000); // Small delay to ensure file is closed
  }
});

// Set up event listener for recording-filename-changed
onMounted(async () => {
  // Set up listener for recording filename changes to update the file list
  filenameChangeUnsubscribe = await listen('recording-filename-changed', (event) => {
    console.log('RecordedFilesList: Recording filename changed to:', event.payload);
    // Trigger a file list refresh when a new recording segment is created
    setTimeout(() => loadFiles(), 500); // Small delay to ensure the file is created
  });
});

// Clean up event listeners
onUnmounted(() => {
  if (filenameChangeUnsubscribe) {
    filenameChangeUnsubscribe();
    filenameChangeUnsubscribe = null;
  }
});

// Methods
async function loadFiles() {
  if (!props.recordingDirectory) return;
  
  // Only show loading indicator if we have no files yet
  if (files.value.length === 0) {
    isLoading.value = true;
  }
  errorMessage.value = '';
  
  try {
    // Pass the BaseDirectory.Document as a hint to loadDirectoryFiles
    // This indicates we're using the new fs plugin approach
    const newFiles = await loadDirectoryFiles(props.recordingDirectory, BaseDirectory.Document);
    updateFilesList(newFiles);
  } catch (error) {
    console.error('Error loading files:', error);
    errorMessage.value = `Failed to load files: ${error}`;
    isLoading.value = false;
  }
}

// Efficiently update files list by finding new and deleted files
function updateFilesList(newFiles: RecordingFile[]) {
  if (!newFiles || newFiles.length === 0) {
    if (files.value.length > 0) {
      // All files were deleted
      files.value = [];
    }
    isLoading.value = false;
    return;
  }
  
  // Create maps for quick lookups
  const currentFilesMap = new Map(files.value.map(file => [file.path, file]));
  const newFilesMap = new Map(newFiles.map(file => [file.path, file]));
  
  // 1. Find files that were deleted (in current but not in new)
  const filesToRemove = files.value.filter(file => !newFilesMap.has(file.path));
  
  // 2. Find files that were added (in new but not in current)
  const filesToAdd = newFiles.filter(file => !currentFilesMap.has(file.path));
  
  // 3. Find files that were updated (in both, but changed)
  const filesToUpdate: {index: number, file: RecordingFile}[] = [];
  files.value.forEach((file, index) => {
    const newFile = newFilesMap.get(file.path);
    if (newFile) {
      // Check if any metadata changed
      if (file.rawSize !== newFile.rawSize || 
          (file.dateObject?.getTime() !== newFile.dateObject?.getTime()) ||
          file.duration !== newFile.duration) {
        filesToUpdate.push({index, file: newFile});
      }
    }
  });
  
  // If there's nothing to change, exit early
  if (filesToRemove.length === 0 && filesToAdd.length === 0 && filesToUpdate.length === 0) {
    isLoading.value = false;
    return;
  }
  
  console.log(`Files to add: ${filesToAdd.length}, remove: ${filesToRemove.length}, update: ${filesToUpdate.length}`);
  
  // Apply updates in this order: remove, update, add
  // 1. Remove deleted files
  if (filesToRemove.length > 0) {
    const pathsToRemove = new Set(filesToRemove.map(f => f.path));
    files.value = files.value.filter(file => !pathsToRemove.has(file.path));
  }
  
  // 2. Update changed files in place
  filesToUpdate.forEach(({index, file}) => {
    // Only update if the index is still valid
    if (index < files.value.length) {
      files.value[index] = {
        ...files.value[index],
        rawSize: file.rawSize,
        size: file.size,
        dateObject: file.dateObject,
        modified: file.modified,
        duration: file.duration
      };
    }
  });
  
  // 3. Add new files
  if (filesToAdd.length > 0) {
    // Make sure the new files are also sorted properly
    sortFilesByNewest(filesToAdd);
    
    // Create final array with remaining files + new files
    const remainingFiles = files.value;
    const finalFiles = [...filesToAdd, ...remainingFiles];
    
    // Sort the entire array by date again to ensure newest files are always first
    sortFilesByNewest(finalFiles);
    
    // Add each file with animation (one at a time if wanted)
    files.value = finalFiles;
  }
  
  isLoading.value = false;
}

async function refreshFiles() {
  await loadFiles();
}

// Check if a file is the current active recording
function isFileActiveRecording(file: RecordingFile): boolean {
  return isActiveRecordingFile(
    file.name, 
    props.recordingFilename,
    props.isRecording
  );
}

// Handle file actions (open, delete, upload)
async function handleFileAction({ action, file }: { action: string, file: RecordingFile }) {
  try {
    switch(action) {
      case 'open':
        await syncFile(file.path);
        break;
      case 'delete':
        await deleteFile(file.path, loadFiles);
        break;
      case 'upload':
        await uploadFile(file.path);
        break;
      default:
        console.warn('Unknown file action:', action);
    }
  } catch (error) {
    console.error(`Error performing ${action}:`, error);
    errorMessage.value = `Error: ${error}`;
  }
}

// Handle file size updates from active recording
function handleFileSizeUpdate({ path, size, formattedSize }: { path: string, size: number, formattedSize: string }) {
  // Find the file in our collection by path
  const fileIndex = files.value.findIndex(file => file.path === path);
  
  if (fileIndex !== -1) {
    // Update the file size in-place
    files.value[fileIndex] = {
      ...files.value[fileIndex],
      rawSize: size,
      size: formattedSize
    };
    console.log(`Updated file size for ${path} to ${formattedSize}`);
    
    // Re-sort files to ensure newest is always first
    sortFilesByNewest(files.value);
  }
}

// Helper function to sort files with newest first
function sortFilesByNewest(fileArray: RecordingFile[]): void {
  fileArray.sort((a, b) => {
    // First try to sort by date objects if available
    if (a.dateObject instanceof Date && b.dateObject instanceof Date) {
      return b.dateObject.getTime() - a.dateObject.getTime();
    }
    
    // Fall back to timestamp in filename
    const getTimestamp = (file: RecordingFile) => {
      const match = file.name.match(/serial_recording_(\d+)/);
      return match ? parseInt(match[1]) : 0;
    };
    
    // Newest first (higher timestamp = newer)
    return getTimestamp(b) - getTimestamp(a);
  });
}
</script>

<style scoped>
/* Filter card add/delete animations */
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
