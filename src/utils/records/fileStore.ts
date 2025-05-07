import { ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { loadBasicFileInfo, loadFileMetadata } from './fileLoader';
import { stat } from '@tauri-apps/plugin-fs';
import { type RecordingFile } from './types';
import { formatFileSize, formatDuration } from './formatters';
import { sortFilesByNewest } from './sortUtils';

// Create a reactive file store that can be shared between components
const files = ref<RecordingFile[]>([]);
const isLoading = ref(false);
const errorMessage = ref<string>('');
let currentDirectory = '';
let isRecording = false;
let currentRecordingFilename = '';
let filenameChangeUnsubscribe: (() => void) | null = null;
let initialized = false;

// Track recording state changes
let lastRecordingState = false;
let lastFilename = '';

// Set up event listeners
async function setupEventListeners() {
  if (initialized) return;
  
  filenameChangeUnsubscribe = await listen('recording-filename-changed', (event) => {
    console.log('[FileStore] Recording filename changed event:', event.payload);
    setTimeout(() => loadFiles(), 500);
  });
  
  initialized = true;
  console.log('[FileStore] Event listeners initialized');
}

// Clean up event listeners
export function cleanup() {
  if (filenameChangeUnsubscribe) {
    filenameChangeUnsubscribe();
    filenameChangeUnsubscribe = null;
  }
  initialized = false;
  console.log('[FileStore] Event listeners cleaned up');
}

// Set or change the current directory
export async function setDirectory(directoryPath: string) {
  if (!directoryPath || directoryPath === currentDirectory) {
    return;
  }
  
  console.log(`[FileStore] Setting directory: ${directoryPath}`);
  currentDirectory = directoryPath;
  
  // Set up event listeners if not already done
  await setupEventListeners();
  
  // Load files from the new directory
  await loadFiles();
}

// Load basic file info (lightweight) from current directory
export async function loadFiles() {
  if (!currentDirectory) {
    console.error('[FileStore] Cannot load files: No directory set');
    errorMessage.value = 'No recording directory set';
    return;
  }
  
  isLoading.value = true;
  errorMessage.value = '';
  
  try {
    console.log(`[FileStore] Loading basic file info from: ${currentDirectory}`);
    
    // Step 1: Quickly load basic file info without detailed stats
    const basicFiles = await loadBasicFileInfo(currentDirectory);
    
    // Update the reactive state with basic file info
    files.value = basicFiles;
    
    if (basicFiles.length === 0) {
      console.log('[FileStore] No recording files found');
    } else {
      console.log(`[FileStore] Loaded ${basicFiles.length} recording files with basic info`);
    }
  } catch (error) {
    console.error('[FileStore] Error loading files:', error);
    errorMessage.value = `Error loading files: ${error}`;
  } finally {
    isLoading.value = false;
  }
}

// Load detailed metadata for specific files (for pagination)
export async function loadMetadataForPage(pageFiles: RecordingFile[]) {
  if (!pageFiles || pageFiles.length === 0) return;
  
  // Only load metadata for files that don't have it yet
  const filesToUpdate = pageFiles.filter(file => !file.hasDetailedMetadata);
  
  if (filesToUpdate.length === 0) {
    console.log('[FileStore] All files on this page already have metadata');
    return;
  }
  
  console.log(`[FileStore] Loading detailed metadata for ${filesToUpdate.length} files on current page`);
  
  try {
    // Load detailed metadata for the specific files
    const updatedFiles = await loadFileMetadata(filesToUpdate);
    
    // Individually update the files in our main array
    updatedFiles.forEach(updatedFile => {
      const index = files.value.findIndex(f => f.key === updatedFile.key);
      if (index !== -1) {
        files.value[index] = updatedFile;
      }
    });
    
    console.log('[FileStore] Successfully loaded metadata for current page');
  } catch (error) {
    console.error('[FileStore] Error loading metadata for page:', error);
  }
}

// Add a single new recording file without reloading all files
export async function addNewRecordingFile(filename: string) {
  if (!currentDirectory || !filename) return;
  
  console.log(`[FileStore] Adding new recording file: ${filename}`);
  const filePath = `${currentDirectory}/${filename}`;
  
  try {
    // Create a minimal file object for the new recording
    const now = new Date();
    const newFile: RecordingFile = {
      name: filename,
      path: filePath,
      size: '0 B',
      rawSize: 0,
      modified: now.toLocaleString(),
      dateObject: now,
      key: filePath,
      hasDetailedMetadata: false // Will be loaded on demand
    };
    
    // Add the new file to the beginning of the array
    files.value = [newFile, ...files.value];
    console.log(`[FileStore] Added new recording file to list (total: ${files.value.length})`);
  } catch (error) {
    console.error('[FileStore] Error adding new recording file:', error);
  }
}

// Update recording state with optimized file handling
export function updateRecordingState(recording: boolean, filename: string) {
  // Skip if nothing has changed
  if (recording === lastRecordingState && filename === lastFilename) {
    console.log('[FileStore] Recording state unchanged, skipping update');
    return;
  }
  
  console.log(`[FileStore] Updating recording state: recording=${recording}, filename=${filename}`);
  
  // Keep track of last filename before updating state
  const previousFilename = currentRecordingFilename;
  
  // Update state
  isRecording = recording;
  
  // Set current filename only if provided
  if (filename) {
    currentRecordingFilename = filename;
  }
  
  // Track previous state for next comparison
  const wasRecording = lastRecordingState;
  lastRecordingState = recording;
  lastFilename = filename || previousFilename;
  
  // Only proceed if we have a directory set
  if (!currentDirectory) return;
  
  // Optimized handling of recording state changes
  if (recording && !wasRecording && filename) {
    // Starting a new recording - add the new file
    addNewRecordingFile(filename);
  } else if (!recording && wasRecording) {
    // Stopping a recording - update the file's metadata
    const filenameToUpdate = filename || previousFilename;
    if (filenameToUpdate) {
      const filePath = `${currentDirectory}/${filenameToUpdate}`;
      updateSingleFileMetadata(filePath);
      console.log(`[FileStore] Updated file metadata after stopping recording: ${filePath}`);
    }
  }
}

// Remove a file from the list
export async function removeFile(filePath: string) {
  console.log(`[FileStore] Removing file from list: ${filePath}`);
  
  try {
    // Filter out the file to be removed
    files.value = files.value.filter(file => file.path !== filePath);
    console.log(`[FileStore] File removed, remaining files: ${files.value.length}`);
    return true;
  } catch (error) {
    console.error(`[FileStore] Error removing file: ${error}`);
    return false;
  }
}

// Helper function to check if a file is currently being recorded
export function isActiveRecording(filePath: string): boolean {
  if (!isRecording || !currentRecordingFilename) return false;
  
  // Extract filename from path for comparison
  const pathParts = filePath.split('/');
  const filename = pathParts[pathParts.length - 1];
  
  return filename === currentRecordingFilename;
}

// Update just the file size (for active recordings)
export function updateFileSize(filePath: string, size: number, formattedSize: string) {
  if (!filePath) return;
  
  console.log(`[FileStore] Updating file size for active recording: ${filePath}, size: ${formattedSize}`);
  
  // Find the file in our collection by path
  const fileIndex = files.value.findIndex(file => file.path === filePath);
  
  if (fileIndex !== -1) {
    // Only update the size properties
    files.value[fileIndex] = {
      ...files.value[fileIndex],
      rawSize: size,
      size: formattedSize
    };
    
    console.log(`[FileStore] Updated size for active recording: ${filePath}`);
  } else {
    console.warn(`[FileStore] Could not find file to update size: ${filePath}`);
  }
}

// Update a single file's metadata (e.g., when recording stops)
async function updateSingleFileMetadata(filePath: string) {
  if (!filePath) return;
  
  console.log(`[FileStore] Updating metadata for single file: ${filePath}`);
  
  try {
    // Get updated file stats
    const fileStats = await stat(filePath);
    
    // Extract filename from path
    const pathParts = filePath.split('/');
    const filename = pathParts[pathParts.length - 1];
    
    // Calculate file size
    const rawSize = fileStats.size;
    const formattedSize = formatFileSize(rawSize);
    
    // Create a date object for the modified time
    const modifiedDate = new Date(fileStats.mtime || Date.now());
    
    // Find the file in our collection by path
    const fileIndex = files.value.findIndex(file => file.path === filePath);
    
    if (fileIndex !== -1) {
      // Update the file with new metadata
      let updatedFile = {
        ...files.value[fileIndex],
        rawSize,
        size: formattedSize,
        modified: modifiedDate.toLocaleString(),
        dateObject: modifiedDate,
        hasDetailedMetadata: true
      };
      
      // Calculate duration based on file timestamps
      if (filename.endsWith('.csv') || filename.endsWith('.json') || filename.endsWith('.bin')) {
        try {
          // Try to extract the timestamp from the filename (e.g., serial_recording_1746593732545.csv)
          const match = filename.match(/serial_recording_(\\d+)/);
          if (match && match[1]) {
            const startTimestamp = parseInt(match[1]);
            const endTimestamp = modifiedDate.getTime();
            
            // Calculate duration in milliseconds and validate
            const durationMs = endTimestamp - startTimestamp;
            
            if (durationMs > 0 && durationMs < 3600000) { // Cap at 1 hour to prevent unreasonable values
              updatedFile.duration = formatDuration(durationMs);
              console.log(`[FileStore] Calculated duration for ${filename}: ${updatedFile.duration} (${durationMs}ms)`);
            } else {
              console.warn(`[FileStore] Invalid duration for ${filename}: ${durationMs}ms`);
            }
          }
        } catch (error) {
          console.error(`[FileStore] Error calculating duration: ${error}`);
        }
      }
      
      // Update the file in-place
      files.value[fileIndex] = updatedFile;
      
      console.log(`[FileStore] Updated single file metadata for: ${filePath}`);
      console.log(`[FileStore] New size: ${formattedSize}, Modified: ${modifiedDate.toLocaleString()}`);
    } else {
      console.warn(`[FileStore] File not found in list, could not update metadata: ${filePath}`);
    }
  } catch (error) {
    console.error(`[FileStore] Error updating file metadata: ${error}`);
  }
}

// Export store methods and state
export default {
  // Reactive state
  files,
  isLoading,
  errorMessage,
  
  // Methods
  loadFiles,
  setDirectory,
  addNewRecordingFile,
  updateRecordingState,
  removeFile,
  isActiveRecording,
  cleanup,
  loadMetadataForPage,
  updateFileSize,
  
  // Computed
  get currentDirectory() {
    return currentDirectory;
  }
};

// For debugging
console.log('[FileStore] Module initialized');
