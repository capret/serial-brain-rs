import { ref, type Ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { loadBasicFileInfo as loadBasicFileInfoOriginal, loadFileMetadata } from './fileLoader';

// Wrapper function to handle ref parameter
async function loadBasicFileInfo(directory: string | Ref<string>) {
  const dirPath = typeof directory === 'string' ? directory : directory.value;
  return loadBasicFileInfoOriginal(dirPath);
}
import { stat } from '@tauri-apps/plugin-fs';
import { type RecordingFile } from './types';
import { formatFileSize, formatDuration } from './formatters';

// Create a reactive file store that can be shared between components
const files = ref<RecordingFile[]>([]);
const isLoading = ref(false);
const errorMessage = ref<string>('');

// Reactive recording state that components can subscribe to
const currentRecordingFilename = ref<string>('');
const isRecording = ref<boolean>(false);
const lastCompletedFilename = ref<string>('');

// Export reactive refs for direct use in components
export { files, isLoading, errorMessage, currentRecordingFilename, isRecording, lastCompletedFilename };

// Reactive state for directory management
const currentDirectory = ref<string>('');
let initialized = false;

// Export currentDirectory for components to use
export { currentDirectory };

// Store all event unsubscribe functions for cleanup
let eventUnsubscribes: Array<() => void> = [];

// Set up event listeners
async function setupEventListeners() {
  if (initialized) return;
  
  try {
    // Clear any existing listeners first
    eventUnsubscribes.forEach(unsub => unsub());
    eventUnsubscribes = [];
    
    // 1. Listen for recording-completed events - handle these first to update completed file duration
    const completedUnsubscribe = await listen('recording-completed', async (event) => {
      const completedFilename = event.payload as string;
      console.log('[FileStore] Recording completed event:', completedFilename);
      
      // Store the last completed filename for reference
      lastCompletedFilename.value = completedFilename;
      console.log(files.value)
      // First, update the specific completed file's metadata with proper duration
      try {
        // Wait a short time for the file system to settle
        setTimeout(async () => {
          // Find the file in our current list
          const completedFile = files.value.find(f => f.name === completedFilename);
          if (completedFile) {
            console.log('[FileStore] Updating metadata for completed file:', completedFilename);
            await updateSingleFileMetadata(completedFile.path);
          } else {
            console.warn('[FileStore] Completed file not found in list:', completedFilename);
          }
        }, 500);
      } catch (error) {
        console.error('[FileStore] Error handling recording-completed event:', error);
      }
    });
    eventUnsubscribes.push(completedUnsubscribe);
    
    // 2. Listen for recording-filename-changed events - to handle rotation to next file
    const filenameChangeUnsubscribe = await listen('recording-filename-changed', (event) => {
      const newFilename = event.payload as string;
      console.log('[FileStore] Recording filename changed event:', newFilename);
      
      // Update the reactive reference with the new filename
      currentRecordingFilename.value = newFilename;
      
      // Explicitly add the new file to our list immediately
      if (newFilename && currentDirectory.value) {
        console.log('[FileStore] Adding new recording file:', newFilename);
        // First add the file to ensure it appears right away
        addNewRecordingFile(newFilename);
      }
      
      // After updating the completed file, also refresh all files to get updated metadata
      // Wait a bit longer to ensure the completed file update has finished
      setTimeout(async () => {
        // Force a complete refresh of the files to get updated metadata
        await loadFiles();
        
        console.log('[FileStore] Files refreshed after rotation');
      }, 800); // Increased delay to ensure file system has updated
    });
    eventUnsubscribes.push(filenameChangeUnsubscribe);
    
    initialized = true;
    console.log('[FileStore] Event listeners initialized');
  } catch (error) {
    console.error('[FileStore] Error setting up event listeners:', error);
  }
}

// Clean up event listeners
export function cleanup() {
  // Clean up all event listeners
  eventUnsubscribes.forEach(unsub => unsub());
  eventUnsubscribes = [];
  
  // Reset state
  initialized = false;
  console.log('[FileStore] Event listeners cleaned up');
}

// Set or change the current directory
export async function setDirectory(directoryPath: string) {
  if (!directoryPath || directoryPath === currentDirectory.value) {
    return;
  }
  
  console.log(`[FileStore] Setting directory: ${directoryPath}`);
  currentDirectory.value = directoryPath;
  
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
    
    // Check if we're just refreshing the same directory
    if (files.value.length > 0) {
      console.log('[FileStore] Files already loaded, performing smart update...');
      
      // Create a map of existing files by path for quick lookup
      const existingFilesMap = new Map(files.value.map(file => [file.path, file]));
      
      // Add only new files that don't exist in the current list
      const newFiles = basicFiles.filter(newFile => !existingFilesMap.has(newFile.path));
      if (newFiles.length > 0) {
        console.log(`[FileStore] Adding ${newFiles.length} new files to the list`);
        files.value = [...newFiles, ...files.value];
      }
      
      // Remove files that no longer exist in the directory
      const basicFilePaths = new Set(basicFiles.map(file => file.path));
      const filesToRemove = files.value.filter(file => !basicFilePaths.has(file.path));
      if (filesToRemove.length > 0) {
        console.log(`[FileStore] Removing ${filesToRemove.length} files that no longer exist`);
        files.value = files.value.filter(file => basicFilePaths.has(file.path));
      }
    } else {
      // First load - simply set all files
      console.log(`[FileStore] First load - setting ${basicFiles.length} files`);
      files.value = basicFiles;
    }
    
    if (files.value.length === 0) {
      console.log('[FileStore] No recording files found');
    } else {
      console.log(`[FileStore] File list now contains ${files.value.length} files`);
    }
  } catch (error) {
    console.error('[FileStore] Error loading files:', error);
    errorMessage.value = `Error loading files: ${error}`;
    files.value = [];
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
  if (!currentDirectory.value || !filename) return;
  
  console.log(`[FileStore] Adding new recording file: ${filename}`);
  const filePath = `${currentDirectory.value}/${filename}`;
  
  // Check if file already exists in our list to prevent duplicates
  const existingFileIndex = files.value.findIndex(file => file.path === filePath);
  if (existingFileIndex !== -1) {
    console.log(`[FileStore] File already exists in list, skipping: ${filename}`);
    return;
  }
  
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

// Track previous state for optimization
let previousRecordingState = false;
let previousFilename = '';

// Update recording state with optimized file handling
export async function updateRecordingState(recording: boolean, filename: string) {
  // Skip if nothing has changed and we're not forcing an update
  if (recording === previousRecordingState && 
      filename === previousFilename && 
      isRecording.value === recording) {
    console.log('[FileStore] Recording state unchanged, skipping update');
    return;
  }
  
  console.log(`[FileStore] Updating recording state: recording=${recording}, filename=${filename}`);
  
  // Keep track of previous filename before updating state
  const prevFilename = currentRecordingFilename.value;
  
  // Update reactive state
  isRecording.value = recording;
  
  // Store new filename if provided, or keep the previous one if stopping a recording
  if (filename && filename.trim() !== '') {
    currentRecordingFilename.value = filename;
    console.log(`[FileStore] Setting current recording filename to: ${filename}`);
  } else if (!recording && prevFilename) {
    // When stopping a recording but no filename provided, keep the previous one for proper cleanup
    console.log(`[FileStore] Keeping previous filename for stopping: ${prevFilename}`);
  }
  
  // Update tracking variables for change detection
  const wasRecording = previousRecordingState;
  previousRecordingState = recording;
  previousFilename = filename || currentRecordingFilename.value; // Ensure we track the actual filename
  
  if (!currentDirectory.value) return;
  
  // Optimized handling of recording state changes
  if (recording && !wasRecording && filename) {
    // Starting a new recording - add the new file
    await addNewRecordingFile(filename);
  } else if (!recording && wasRecording) {
    // Stopping a recording - update the file's metadata
    const filenameToUpdate = filename || previousFilename;
    if (filenameToUpdate) {
      const filePath = `${currentDirectory.value}/${filenameToUpdate}`;
      await updateSingleFileMetadata(filePath);
      console.log(`[FileStore] Updated file metadata after stopping recording: ${filePath}`);
    }
  } else if (recording && filename) {
    // If already recording but navigated back to page, ensure file exists
    console.log(`[FileStore] Ensuring recording file exists in list: ${filename}`);
    const filePath = `${currentDirectory.value}/${filename}`;
    const fileExists = files.value.some(file => file.path === filePath);
    
    if (!fileExists) {
      console.log(`[FileStore] Active recording file not found in list, adding: ${filename}`);
      await addNewRecordingFile(filename);
      
      // If still not found in the list after adding (could happen if file doesn't exist yet),
      // force reload files from directory to ensure we have the latest state
      const stillNotExists = !files.value.some(file => file.path === filePath);
      if (stillNotExists) {
        console.log(`[FileStore] Active recording file still not found, reloading directory: ${currentDirectory.value}`);
        await loadFiles();
      }
    } else {
      console.log(`[FileStore] Active recording file found in list: ${filename}`);
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
  if (!isRecording.value || !currentRecordingFilename.value) return false;
  
  // Extract filename from path for comparison
  const pathParts = filePath.split('/');
  const filename = pathParts[pathParts.length - 1];
  
  // Double check against the current recording filename
  const isActive = filename === currentRecordingFilename.value;
  console.log(`[FileStore] Checking if ${filename} is active recording: ${isActive} (current: ${currentRecordingFilename.value})`);
  return isActive;
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
export async function updateSingleFileMetadata(filePath: string) {
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
          const match = filename.match(/serial_recording_(\d+)/);
          if (match && match[1]) {
            const startTimestamp = parseInt(match[1]);
            const endTimestamp = modifiedDate.getTime();
            
            // Calculate duration in milliseconds and validate
            let durationMs = endTimestamp - startTimestamp;
            
            // Ensure we have a reasonable duration (at least 1 second, max 12 hours)
            if (durationMs < 1000) {
              console.warn(`[FileStore] Duration too short (${durationMs}ms), using modified time difference`);
              // As fallback, try to use birthtime if available, or modified time as an estimate
              if (fileStats.birthtime) {
                const createdDate = new Date(fileStats.birthtime);
                durationMs = modifiedDate.getTime() - createdDate.getTime();
              } else {
                // Just ensure a minimum duration as a fallback
                durationMs = Math.max(1000, durationMs);
              }
            }
            
            if (durationMs >= 1000 && durationMs < 43200000) { // Between 1 sec and 12 hours
              updatedFile.duration = formatDuration(durationMs);
              console.log(`[FileStore] Calculated duration for ${filename}: ${updatedFile.duration} (${durationMs}ms)`);
            } else {
              console.warn(`[FileStore] Invalid duration for ${filename}: ${durationMs}ms, falling back to estimation`);
              // If duration still looks wrong, use file size to roughly estimate duration
              // Assuming ~20KB per second for CSV files as a rough estimate
              if (rawSize > 0) {
                const estimatedDuration = Math.max(1000, Math.min(rawSize * 50, 3600000)); // 1 sec to 1 hour
                updatedFile.duration = formatDuration(estimatedDuration);
                console.log(`[FileStore] Estimated duration from file size: ${updatedFile.duration}`);
              }
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
