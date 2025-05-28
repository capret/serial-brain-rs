<template>
  <div>
    <!-- Section header with title and refresh button -->
    <div class="flex justify-between items-center mb-6">
      <h2 class="text-3xl font-bold text-blue-400">{{ $t('recording.files') }}</h2>
      <button 
        @click="refreshFiles" 
        class="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-md flex items-center gap-2">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" 
          stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
        </svg>
        {{ $t('common.refresh') }}
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
      <p v-if="recordingDirectory">{{ $t('recording.noFiles') }}</p>
      <p v-else>{{ $t('recording.selectDirectory') }}</p>
    </div>
    
    <!-- File grid with transition animations -->
    <div v-else :key="componentKey">
      <div class="file-grid-container">
        <div class="pages-slider" :style="`transform: translateX(-${(currentPage-1) * 100}%)`">
          <!-- Generate all pages in a horizontal slider -->
          <div v-for="page in totalPages" :key="page" class="page-slide">
            <transition-group 
              name="file-card" 
              tag="div" 
              class="file-grid grid gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3">
              <FileCard 
                v-for="file in getFilesForPage(page)" 
                :key="file.path" 
                :file="file"
                :is-active-recording="fileStore.isActiveRecording(file.path)"
                @action="handleFileAction"
                @update-file-size="handleFileSizeUpdate"
              />
            </transition-group>
          </div>
        </div>
      </div>
      <!-- Pagination Controls with ellipsis for many pages -->
      <div v-if="totalPages > 1" class="flex justify-center items-center mt-6 space-x-2">
        <button 
          :disabled="currentPage === 1" 
          @click="goToPage(currentPage - 1)" 
          class="px-2 py-1 rounded bg-gray-700 text-white disabled:opacity-50 flex items-center justify-center" 
          aria-label="Previous page"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" 
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="15 18 9 12 15 6"></polyline>
          </svg>
        </button>
        
        <!-- First page button always visible -->
        <button v-if="totalPages > 0" 
          @click="goToPage(1)" 
          :class="['px-3 py-1 rounded', 1 === currentPage ? 'bg-blue-600 text-white' : 'bg-gray-700 text-white']">
          1
        </button>
        
        <!-- Left ellipsis -->
        <span v-if="showLeftEllipsis" class="px-1 text-gray-400">...</span>
        
        <!-- Page buttons around current page -->
        <button v-for="page in middlePages" :key="page" 
          @click="goToPage(page)" 
          :class="['px-3 py-1 rounded', page === currentPage ? 'bg-blue-600 text-white' : 'bg-gray-700 text-white']">
          {{ page }}
        </button>
        
        <!-- Right ellipsis -->
        <span v-if="showRightEllipsis" class="px-1 text-gray-400">...</span>
        
        <!-- Last page button if not already shown -->
        <button v-if="totalPages > 1 && !middlePages.includes(totalPages)" 
          @click="goToPage(totalPages)" 
          :class="['px-3 py-1 rounded', totalPages === currentPage ? 'bg-blue-600 text-white' : 'bg-gray-700 text-white']">
          {{ totalPages }}
        </button>
        
        <button 
          :disabled="currentPage === totalPages" 
          @click="goToPage(currentPage + 1)" 
          class="px-2 py-1 rounded bg-gray-700 text-white disabled:opacity-50 flex items-center justify-center"
          aria-label="Next page"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" 
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="9 18 15 12 9 6"></polyline>
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed } from 'vue';
import { ref, watch, onUnmounted, onMounted } from 'vue';
import FileCard from './FileCard.vue';
import { type RecordingFile } from '../../utils/records/types';
import { syncFile, deleteFile, uploadFile } from '../../utils/records/fileOperations';
import fileStore from '../../utils/records/fileStore';

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

// Use the persistent file store
const files = fileStore.files;
const isLoading = fileStore.isLoading;
const errorMessage = fileStore.errorMessage;

// Track if we've loaded metadata for current page
const currentPageMetadataLoaded = ref(false);

// Pagination state
const currentPage = ref(1);
const pageSize = ref(6); // Show 6 files per page (2 rows of 3)

const totalPages = computed(() => Math.max(1, Math.ceil(files.value.length / pageSize.value)));

const paginatedFiles = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value;
  const pageFiles = files.value.slice(start, start + pageSize.value);
  
  // Set flag to load metadata for this page
  currentPageMetadataLoaded.value = false;
  
  return pageFiles;
});

// Pagination display logic - which page numbers to show
const MAX_VISIBLE_PAGES = 3; // Max number of page buttons to show (excluding ellipsis)

const middlePages = computed(() => {
  if (totalPages.value <= MAX_VISIBLE_PAGES + 2) { // +2 for first and last pages
    // If few pages, show all pages from 2 to totalPages-1
    return Array.from({ length: Math.max(0, totalPages.value - 2) }, (_, i) => i + 2);
  }
  
  // Calculate range around current page
  const rangeStart = Math.max(2, currentPage.value - 1);
  const rangeEnd = Math.min(totalPages.value - 1, currentPage.value + 1);
  
  return Array.from(
    { length: rangeEnd - rangeStart + 1 },
    (_, i) => i + rangeStart
  );
});

const showLeftEllipsis = computed(() => 
  totalPages.value > MAX_VISIBLE_PAGES + 2 && middlePages.value[0] > 2
);

const showRightEllipsis = computed(() => 
  totalPages.value > MAX_VISIBLE_PAGES + 2 && 
  middlePages.value[middlePages.value.length - 1] < totalPages.value - 1
);

function getFilesForPage(page: number) {
  const start = (page - 1) * pageSize.value;
  const pageFiles = files.value.slice(start, start + pageSize.value);
  
  // If this is the current page or adjacent pages, preload metadata
  if (Math.abs(page - currentPage.value) <= 1 && pageFiles.length > 0) {
    fileStore.loadMetadataForPage(pageFiles);
  }
  
  return pageFiles;
}

function goToPage(page: number) {
  if (page === currentPage.value || page < 1 || page > totalPages.value) {
    return;
  }
  
  // Preload data for the next page to avoid data fetching delays
  getFilesForPage(page); // Just call the function to trigger preloading
  
  // Apply page change with smooth animation
  requestAnimationFrame(() => {
    currentPage.value = page;
  });
}

// Initialize on mount
// We need a key to force re-rendering when needed
const componentKey = ref(0);

// Flag to prevent double initialization
const isInitialized = ref(false);

// Initialize component - handle both mounting and directory changes
async function initializeComponent(directoryPath: string) {
  if (!directoryPath || directoryPath.trim() === '') {
    console.log('Skipping initialization - empty directory path');
    return;
  }
  
  console.log('Initializing component with directory:', directoryPath);
  
  // Set recording state first
  fileStore.updateRecordingState(props.isRecording, props.recordingFilename);
  
  // First initialization or directory change
  if (!isInitialized.value) {
    console.log('First initialization of component');
    await fileStore.setDirectory(directoryPath);
    isInitialized.value = true;
    currentPage.value = 1; // Start at first page
    currentPageMetadataLoaded.value = false; // Reset metadata loaded state
    console.log('Component first initialization complete');
  } 
  // Directory has changed
  else if (directoryPath !== fileStore.currentDirectory) {
    console.log('Directory changed, reloading files');
    await fileStore.setDirectory(directoryPath);
    currentPage.value = 1; // Reset to first page on new directory
    currentPageMetadataLoaded.value = false; // Reset metadata loaded state
    console.log('Component re-initialization complete with new directory');
  }
  // Same directory, returning to the page - don't reload everything
  else {
    console.log('Returning to recording page with same directory, using existing file list');
    // No need to call setDirectory or loadFiles, just use the existing state
  }
}
onMounted(async () => {
  console.log('RecordedFilesList mounted...');
  
  // Check if we have a valid directory on mount
  if (props.recordingDirectory && props.recordingDirectory.trim() !== '') {
    console.log(`Directory available on mount: ${props.recordingDirectory}`);
    await initializeComponent(props.recordingDirectory);
    
    // Force a re-render if we have files but UI isn't updating
    if (files.value.length > 0 && paginatedFiles.value.length === 0) {
      console.log('Forcing component re-render');
      componentKey.value++;
    }
  } else {
    // If no directory yet, we'll rely on the watch to initialize when it becomes available
    console.log('No directory available on mount, waiting for directory to be set');
  }
});

// Watch for directory changes
watch(
  () => props.recordingDirectory,
  async (newDirectory, oldDirectory) => {
    console.log(`Recording directory changed: ${oldDirectory} -> ${newDirectory}`);
    if (newDirectory && newDirectory !== oldDirectory) {
      await initializeComponent(newDirectory);
    }
  }
);

// Watch paginated files to load detailed metadata for current page
watch(
  () => paginatedFiles.value,
  async (newPageFiles) => {
    if (!currentPageMetadataLoaded.value && newPageFiles.length > 0) {
      console.log('Loading metadata for current page files...');
      await fileStore.loadMetadataForPage(newPageFiles);
      currentPageMetadataLoaded.value = true;
    }
  },
  { immediate: true }
);

// Watch for directory changes - with special handling for initial directory setting
watch(() => props.recordingDirectory, async (newPath, oldPath) => {
  if (!newPath || newPath.trim() === '') {
    console.log('Ignoring empty directory path');
    return;
  }
  
  // Handle initial directory setting (when it wasn't available on mount)
  if (!isInitialized.value) {
    console.log(`Directory now available: ${newPath}`);
    await initializeComponent(newPath);
  }
  // Handle directory change
  else if (newPath !== fileStore.currentDirectory) {
    console.log(`Directory changed from: ${oldPath} to: ${newPath}`);
    await initializeComponent(newPath);
  }
});

// Store the last known recording filename to handle stop recording properly
const lastRecordingFilename = ref('');

// Watch for recording state changes
watch([() => props.isRecording, () => props.recordingFilename], async ([isRecording, filename]) => {
  // Skip if not initialized yet (will be handled during initialization)
  if (!isInitialized.value) return;
  
  // Store the last valid filename when we have one
  if (filename && filename.trim() !== '') {
    lastRecordingFilename.value = filename;
    console.log('Saved recording filename:', lastRecordingFilename.value);
  }
  
  // When recording stops but filename is empty, use the last known filename
  if (!isRecording && (!filename || filename.trim() === '') && lastRecordingFilename.value) {
    console.log('Recording stopped with empty filename, using last known filename:', lastRecordingFilename.value);
    await fileStore.updateRecordingState(isRecording, lastRecordingFilename.value);
  } else {
    console.log('Recording state changed:', isRecording, filename);
    await fileStore.updateRecordingState(isRecording, filename);
  }
});

// Clean up on unmount
onUnmounted(() => {
  fileStore.cleanup();
});

// Methods
async function refreshFiles() {
  await fileStore.loadFiles();
  currentPage.value = 1; // Reset to first page on refresh
  componentKey.value++; // Force re-render after refresh
}

// No longer needed as we're using fileStore.isActiveRecording directly in the template

// Handle file actions (open, delete, upload)
async function handleFileAction({ action, file }: { action: string, file: RecordingFile }) {
  try {
    switch(action) {
      case 'open':
        await syncFile(file.path);
        break;
      case 'delete':
        await deleteFile(file.path, fileStore.loadFiles);
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
  fileStore.updateFileSize(path, size, formattedSize);
}


</script>

<style scoped>
/* Filter card add/delete animations - for normal file updates */
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

/* Fixed height container to prevent layout shifts */
.file-grid-container {
  position: relative;
  min-height: 232px; /* Accommodates 2 rows of cards */
  overflow: hidden; /* Prevents content from causing layout shifts */
  perspective: 1000px; /* Adds depth to transitions */
  transform-style: preserve-3d; /* Better 3D positioning */
  isolation: isolate; /* Creates a new stacking context */
  background: inherit; /* Match parent background */
}

/* Horizontal slider that contains all pages */
.pages-slider {
  display: flex;
  width: 100%;
  transition: transform 0.4s cubic-bezier(0.165, 0.84, 0.44, 1);
  will-change: transform;
  position: relative;
}

/* Individual page slide */
.page-slide {
  flex: 0 0 100%;
  width: 100%;
  min-width: 100%;
}

/* File grid inside each page */
.file-grid {
  position: relative;
  width: 100%;
}

/* File card animations */
.file-card-move {
  transition: transform 0.4s cubic-bezier(0.165, 0.84, 0.44, 1);
}

.file-card-enter-active {
  transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
  transition-delay: 0.05s; /* Slight delay to avoid visual glitches */
}

.file-card-leave-active {
  transition: all 0.25s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.file-card-enter-from {
  opacity: 0;
  transform: scale(0.9);
}

.file-card-leave-to {
  opacity: 0;
  transform: scale(0.9);
}

.contents > * {
  will-change: opacity, transform;
}
</style>
