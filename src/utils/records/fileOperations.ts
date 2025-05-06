// File operations like syncing, uploading, and deleting
import { invoke } from '@tauri-apps/api/core';
import { RecordingFile } from './types';

/**
 * Open file in file explorer or viewer
 */
export async function syncFile(filePath: string): Promise<void> {
  if (!filePath) {
    throw new Error('No file path provided');
  }
  
  try {
    await invoke('show_in_folder', { path: filePath });
    console.log('File opened in explorer:', filePath);
  } catch (error) {
    console.error('Failed to open file in explorer:', error);
    throw new Error(`Failed to open file: ${error}`);
  }
}

/**
 * Upload file to a remote server or cloud storage
 */
export async function uploadFile(filePath: string): Promise<void> {
  if (!filePath) {
    throw new Error('No file path provided');
  }
  
  try {
    // Here we would implement the actual upload functionality
    // This could involve:
    // 1. Reading the file
    // 2. Setting up a progress indicator
    // 3. Making an HTTP request to upload the file
    // 4. Handling success/failure
    
    console.log('Upload functionality not yet implemented');
    alert('Upload functionality will be implemented in a future update.');
    
    /*
    // Example implementation sketch:
    const response = await invoke('upload_file', {
      path: filePath,
      destination: 'some-cloud-service',
      credentials: { /* credentials would go here */ /*}
    });
    
    console.log('File uploaded successfully:', response);
    */
  } catch (error) {
    console.error('Failed to upload file:', error);
    throw new Error(`Failed to upload file: ${error}`);
  }
}

/**
 * Delete file from the filesystem
 */
export async function deleteFile(
  filePath: string, 
  onComplete: () => Promise<void>
): Promise<void> {
  if (!filePath) {
    throw new Error('No file path provided');
  }
  
  try {
    await invoke('delete_file', { path: filePath });
    console.log('File deleted:', filePath);
    
    // Call the completion handler to refresh the file list
    await onComplete();
  } catch (error) {
    console.error('Failed to delete file:', error);
    throw new Error(`Failed to delete file: ${error}`);
  }
}

/**
 * Update files in-place to avoid UI flashing
 */
export function updateFilesInPlace(
  newFiles: RecordingFile[], 
  currentFiles: RecordingFile[],
  setFiles: (files: RecordingFile[]) => void,
  setIsLoading: (loading: boolean) => void
): void {
  if (!newFiles || newFiles.length === 0) {
    setIsLoading(false);
    return;
  }

  // Special case: if we have no existing files, use the special Vue set operation to avoid flicker
  if (currentFiles.length === 0) {
    // Add files one by one with a stable key to enable proper transitions
    setFiles(newFiles.map(file => ({
      ...file,
      key: file.path  // Ensure stable key for Vue's transition system
    })));
    setIsLoading(false);
    return;
  }
  
  // Create map for quick lookups
  const newFilesMap = new Map(newFiles.map(file => [file.path, file]));
  
  // Track which files we've already processed to avoid duplication
  const processedPaths = new Set<string>();
  
  // Make a copy of the current files array to work with
  const updatedFiles = [...currentFiles];
  
  // 1. First update existing files in place (most important to avoid flicker)
  for (let i = 0; i < updatedFiles.length; i++) {
    const existingFile = updatedFiles[i];
    const newFile = newFilesMap.get(existingFile.path);
    
    if (newFile) {
      // File exists in both arrays - update properties individually
      // instead of replacing the entire object to preserve reactivity
      if (existingFile.rawSize !== newFile.rawSize) {
        updatedFiles[i] = { 
          ...existingFile,
          rawSize: newFile.rawSize,
          size: newFile.size
        };
      }
      
      if (newFile.dateObject instanceof Date && 
          (!(existingFile.dateObject instanceof Date) || 
           existingFile.dateObject.getTime() !== newFile.dateObject.getTime())) {
        updatedFiles[i] = { 
          ...updatedFiles[i],
          dateObject: newFile.dateObject,
          modified: newFile.modified
        };
      }
      
      // Update duration if available
      if (newFile.duration && (!existingFile.duration || existingFile.duration !== newFile.duration)) {
        updatedFiles[i] = {
          ...updatedFiles[i],
          duration: newFile.duration
        };
      }
      
      // Mark as processed
      processedPaths.add(existingFile.path);
    }
  }
  
  // 2. Filter out files that don't exist in the new list
  const remainingFiles = updatedFiles.filter(file => newFilesMap.has(file.path));
  
  // 3. Add new files that weren't in the original list
  const filesToAdd: RecordingFile[] = [];
  for (const newFile of newFiles) {
    if (!processedPaths.has(newFile.path)) {
      // This is a new file we need to add
      // Ensure it has a stable key for Vue transitions
      filesToAdd.push({
        ...newFile,
        key: newFile.path
      });
    }
  }
  
  // Sort the new files to be added by date
  filesToAdd.sort((a, b) => {
    if (a.dateObject instanceof Date && b.dateObject instanceof Date) {
      return b.dateObject.getTime() - a.dateObject.getTime();
    }
    // Fallback to filename timestamp comparison
    const getTimestamp = (file: RecordingFile) => {
      const match = file.name.match(/serial_recording_(\d+)/);
      return match ? parseInt(match[1]) : 0;
    };
    return getTimestamp(b) - getTimestamp(a);
  });
  
  // Create final array with remaining files + new files
  const finalFiles = [...filesToAdd, ...remainingFiles];
  
  // Sort the entire array by date again to ensure proper order
  finalFiles.sort((a, b) => {
    if (a.dateObject instanceof Date && b.dateObject instanceof Date) {
      return b.dateObject.getTime() - a.dateObject.getTime();
    }
    const getTimestamp = (file: RecordingFile) => {
      const match = file.name.match(/serial_recording_(\d+)/);
      return match ? parseInt(match[1]) : 0;
    };
    return getTimestamp(b) - getTimestamp(a);
  });
  
  // Update the files array
  setFiles(finalFiles);
  setIsLoading(false);
}
