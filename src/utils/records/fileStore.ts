import { ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { loadBasicFileInfo, loadFileMetadata } from './fileLoader';
import { stat } from '@tauri-apps/plugin-fs';
import { type RecordingFile } from './types';
import { formatFileSize, formatDuration } from './formatters';


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

// Update recording state with optimized file handling
export async function updateRecordingState(recording: boolean, filename: string) {
  // Skip if nothing has changed and we're not forcing an update
  if (recording === lastRecordingState && filename === lastFilename && isRecording === recording) {
    console.log('[FileStore] Recording state unchanged, skipping update');
    return;
  }
  
  console.log(`[FileStore] Updating recording state: recording=${recording}, filename=${filename}`);
  
  // Keep track of last filename before updating state
  const previousFilename = currentRecordingFilename;
  
  // Update state
  isRecording = recording;
  
  // Store new filename if provided, or keep the previous one if stopping a recording
  if (filename && filename.trim() !== '') {
    currentRecordingFilename = filename;
    console.log(`[FileStore] Setting current recording filename to: ${filename}`);
  } else if (!recording && previousFilename) {
    // When stopping a recording but no filename provided, keep the previous one for proper cleanup
    console.log(`[FileStore] Keeping previous filename for stopping: ${previousFilename}`);
  }
  
  // Update tracking variables
  const wasRecording = lastRecordingState;
  lastRecordingState = recording;
  lastFilename = filename || currentRecordingFilename; // Ensure we track the actual filename
  
  if (!currentDirectory) return;
  
  // Optimized handling of recording state changes
  if (recording && !wasRecording && filename) {
    // Starting a new recording - add the new file
    await addNewRecordingFile(filename);
  } else if (!recording && wasRecording) {
    // Stopping a recording - update the file's metadata
    const filenameToUpdate = filename || previousFilename;
    if (filenameToUpdate) {
      const filePath = `${currentDirectory}/${filenameToUpdate}`;
      await updateSingleFileMetadata(filePath);
      console.log(`[FileStore] Updated file metadata after stopping recording: ${filePath}`);
    }
  } else if (recording && filename) {
    // If already recording but navigated back to page, ensure file exists
    console.log(`[FileStore] Ensuring recording file exists in list: ${filename}`);
    const filePath = `${currentDirectory}/${filename}`;
    const fileExists = files.value.some(file => file.path === filePath);
    
    if (!fileExists) {
      console.log(`[FileStore] Active recording file not found in list, adding: ${filename}`);
      await addNewRecordingFile(filename);
      
      // If still not found in the list after adding (could happen if file doesn't exist yet),
      // force reload files from directory to ensure we have the latest state
      const stillNotExists = !files.value.some(file => file.path === filePath);
      if (stillNotExists) {
        console.log(`[FileStore] Active recording file still not found, reloading directory: ${currentDirectory}`);
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
  if (!isRecording || !currentRecordingFilename) return false;
  
  // Extract filename from path for comparison
  const pathParts = filePath.split('/');
  const filename = pathParts[pathParts.length - 1];
  
  // Double check against the current recording filename
  const isActive = filename === currentRecordingFilename;
  console.log(`[FileStore] Checking if ${filename} is active recording: ${isActive} (current: ${currentRecordingFilename})`);
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
      
      // Calculate duration based on file timestamps or size for all supported formats
      if (filename.endsWith('.csv') || filename.endsWith('.json') || filename.endsWith('.bin')) {
        try {
          let durationSeconds = 0;
          let durationMethod = 'unknown';
          
          if (fileStats.birthtime) {
            const createdDate = new Date(fileStats.birthtime);
            const calculatedSeconds = Math.round((modifiedDate.getTime() - createdDate.getTime()) / 1000);
            
            durationSeconds = calculatedSeconds;
            durationMethod = 'file timestamps';
            console.log(`[FileStore] Calculated duration using file timestamps: ${durationSeconds}s (created: ${createdDate.toISOString()}, modified: ${modifiedDate.toISOString()})`);
          }
          
          // Fallback: If creation time isn't available, estimate from file size
          else if (rawSize > 0) {
            // Conservative estimate based on file size (~20KB/sec for signal data)
            // Limit between 2 seconds and 1 hour for safety
            durationSeconds = Math.min(Math.max(Math.round(rawSize / 20000), 2), 3600);
            durationMethod = 'file size estimation';
            console.log(`[FileStore] Estimated duration from file size: ${durationSeconds}s (size: ${rawSize} bytes)`);
          }
          
          // Last resort - use 2 seconds as minimum duration
          else {
            durationSeconds = 2;
            durationMethod = 'minimum fallback';
            console.log(`[FileStore] Using fallback minimum duration: ${durationSeconds}s`);
          }
          
          // Convert seconds to our HH:MM:SS format and store
          updatedFile.duration = formatDuration(durationSeconds * 1000); // formatDuration expects ms
          console.log(`[FileStore] Set duration for ${filename}: ${updatedFile.duration} (${durationSeconds}s) using ${durationMethod}`);
        } catch (error) {
          console.error(`[FileStore] Error calculating duration: ${error}`);
          // Provide a fallback duration even if calculation fails
          updatedFile.duration = '00:00:02';
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
  updateSingleFileMetadata,
  
  // Computed
  get currentDirectory() {
    return currentDirectory;
  }
};

// For debugging
console.log('[FileStore] Module initialized');
