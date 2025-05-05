import { invoke } from '@tauri-apps/api/core';
import { stat, readDir, watchImmediate } from '@tauri-apps/plugin-fs';


// File interface
export interface RecordingFile {
  name: string;
  path: string;
  size: string;
  modified: string;
  duration?: string; // Optional duration field
  rawSize: number;
  dateObject: Date | null;
  key: string;
}

// Define the unsubscribe function type that's returned by the watchImmediate function
export type UnsubscribeFn = () => void;

/**
 * Format a file size in bytes to a human-readable string
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes';

  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

/**
 * Format duration in milliseconds to a human-readable string
 */
export function formatDuration(milliseconds: number): string {
  if (!milliseconds || milliseconds <= 0) return '0s';
  
  // Convert to seconds
  const seconds = Math.floor(milliseconds / 1000);
  
  if (seconds < 60) {
    return `${seconds}s`;
  }
  
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  
  if (minutes < 60) {
    return remainingSeconds > 0 ? 
      `${minutes}m ${remainingSeconds}s` : 
      `${minutes}m`;
  }
  
  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  
  return remainingMinutes > 0 ?
    `${hours}h ${remainingMinutes}m` :
    `${hours}h`;
}

/**
 * Calculate and update the duration for a recording file
 */
export function updateFileDuration(file: RecordingFile, birthtime: Date | null, modifiedTime: Date | null): RecordingFile {
  if (birthtime instanceof Date && modifiedTime instanceof Date) {
    const durationMs = modifiedTime.getTime() - birthtime.getTime();
    if (durationMs > 0) {
      file.duration = formatDuration(durationMs);
      console.log('Updated duration:', file.duration);
    }
  }
  return file;
}

/**
 * Format date from file object
 */
export function formatDate(file: RecordingFile): string {
  try {
    if (file.dateObject instanceof Date) {
      return file.dateObject.toLocaleDateString();
    }
    
    // Try getting the date from the filename as a fallback
    const match = file.name.match(/serial_recording_(\d+)/);
    if (match && match[1]) {
      const timestamp = parseInt(match[1]);
      if (!isNaN(timestamp)) {
        return new Date(timestamp).toLocaleDateString();
      }
    }
    
    return 'Unknown';
  } catch (e) {
    console.error('Error formatting date:', e);
    return 'Unknown';
  }
}

/**
 * Format time from file object
 */
export function formatTime(file: RecordingFile): string {
  try {
    if (file.dateObject instanceof Date) {
      return file.dateObject.toLocaleTimeString();
    }
    
    // Try getting the time from the filename as a fallback
    const match = file.name.match(/serial_recording_(\d+)/);
    if (match && match[1]) {
      const timestamp = parseInt(match[1]);
      if (!isNaN(timestamp)) {
        return new Date(timestamp).toLocaleTimeString();
      }
    }
    
    // Get current time as fallback
    return new Date().toLocaleTimeString();
  } catch (e) {
    console.error('Error formatting time:', e);
    return 'Unknown';
  }
}

/**
 * Load all CSV files from a directory
 */
export async function loadDirectoryFiles(
  directoryPath: string
): Promise<RecordingFile[]> {
  if (!directoryPath) return [];
  
  try {
    // Read all files in the directory
    const entries = await readDir(directoryPath);
    
    // Filter for CSV files
    const csvEntries = entries.filter(entry => entry.name.toLowerCase().endsWith('.csv'));
    
    // Get file stats for each CSV file
    const filesWithStats = await Promise.all(
      csvEntries.map(async entry => {
        try {
          const fileStat = await stat(`${directoryPath}/${entry.name}`);
          // Get the modified date from FileInfo
          const modifiedDate = fileStat.mtime || fileStat.birthtime;
          
          // Calculate duration if both mtime and birthtime are available
          let duration = undefined;
          if (fileStat.mtime instanceof Date && fileStat.birthtime instanceof Date) {
            const durationMs = fileStat.mtime.getTime() - fileStat.birthtime.getTime();
            if (durationMs > 0) {
              duration = formatDuration(durationMs);
            }
          }
          
          return {
            name: entry.name,
            path: `${directoryPath}/${entry.name}`,
            size: formatFileSize(fileStat.size),
            modified: modifiedDate instanceof Date ? modifiedDate.toLocaleString() : 'Unknown',
            duration,
            rawSize: fileStat.size,
            dateObject: modifiedDate,
            key: entry.name,
          };
        } catch (e) {
          console.error(`Error getting stats for ${entry.name}:`, e);
          return null;
        }
      })
    );
    
    // Filter out failed entries and sort by date
    const validFiles = filesWithStats.filter(file => file !== null) as RecordingFile[];
    return validFiles.sort((a, b) => {
      if (a.dateObject instanceof Date && b.dateObject instanceof Date) {
        return b.dateObject.getTime() - a.dateObject.getTime();
      }
      // Fall back to filename timestamp if no valid dates
      const getTimestamp = (file: RecordingFile) => {
        const match = file.name.match(/serial_recording_(\d+)/);
        return match ? parseInt(match[1]) : 0;
      };
      return getTimestamp(b) - getTimestamp(a);
    });
  } catch (error) {
    console.error('Error loading directory files:', error);
    throw new Error(`Failed to load files: ${error}`);
  }
}

/**
 * Setup a watcher for the directory to detect file additions/removals
 */
export async function setupDirectoryWatcher(
  directoryPath: string,
  onChange: () => Promise<void>,
  setUnsubscribe: (unsubscribe: UnsubscribeFn) => void,
  onError: (error: any) => void
): Promise<void> {
  if (!directoryPath) return;
  
  try {
    // Set up a watcher for file changes
    const unsubscribe = await watchImmediate(
      directoryPath,
      async () => {
        // Reload directory contents when changes are detected
        await onChange();
      },
      { recursive: false }
    );
    // Pass the unsubscribe function to the callback
    setUnsubscribe(unsubscribe);
  } catch (error) {
    console.error('Error setting up directory watcher:', error);
    onError(error);
  }
}

/**
 * Setup a watcher for the active recording file
 */
export async function setupRecordingFileWatcher(
  filePath: string,
  onFileChange: () => Promise<void>,
  currentUnsubscribe: UnsubscribeFn | null
): Promise<UnsubscribeFn> {
  // Clean up existing watcher if any
  if (currentUnsubscribe) {
    currentUnsubscribe();
  }
  
  if (!filePath) {
    // Return a no-op function if no path is provided
    return () => {};
  }
  
  try {
    // Set parent directory of the file
    const parentDir = filePath.substring(0, filePath.lastIndexOf('/'));
    
    // Set up watcher on parent directory
    const unsubscribe = await watchImmediate(
      parentDir,
      async () => {
        // Only update if our specific file changed
        await onFileChange();
      },
      { recursive: false }
    );
    return unsubscribe;
  } catch (error) {
    console.error('Error setting up file watcher:', error);
    // Return a no-op function on error
    return () => {};
  }
}

/**
 * Check if a filename is the current active recording file
 * 
 * This function handles potential timestamp discrepancies between frontend and backend
 * by matching the timestamps with a tolerance of a few seconds.
 */
export function isActiveRecordingFile(filename: string, recordingFilename: string, isRecording: boolean): boolean {
  if (!isRecording) return false;
  if (!recordingFilename || !filename) return false;
  
  // Extract timestamp from the expected filename (frontend)
  const expectedMatch = recordingFilename.match(/serial_recording_(\d+)/);
  if (!expectedMatch || !expectedMatch[1]) return false;
  const expectedTimestamp = parseInt(expectedMatch[1]);
  
  // Extract timestamp from the actual filename (backend)
  const actualMatch = filename.match(/serial_recording_(\d+)/);
  if (!actualMatch || !actualMatch[1]) return false;
  const actualTimestamp = parseInt(actualMatch[1]);
  
  // If timestamps are within 10 seconds of each other, consider it a match
  // This handles slight differences between frontend and backend timestamps
  const timeDifferenceMs = Math.abs(expectedTimestamp - actualTimestamp);
  const toleranceMs = 10 * 1000; // 10 seconds tolerance
  
  return timeDifferenceMs <= toleranceMs;
}

/**
 * Find the actual recording file in the directory and update its size
 */
export async function findAndUpdateActiveRecordingFile(
  isRecording: boolean,
  recordingDirectory: string,
  recordingFilename: string,
  selectedFormat: string,
  folderFiles: RecordingFile[],
  setActiveRecordingPath: (path: string) => void,
  setRecordingFilename: (name: string) => void,
  updateFiles: (files: RecordingFile[]) => void
): Promise<void> {
  // console.log('findAndUpdateActiveRecordingFile called with:');
  // console.log('isRecording:', isRecording);
  // console.log('recordingDirectory:', recordingDirectory);
  // console.log('recordingFilename:', recordingFilename);
  // console.log('selectedFormat:', selectedFormat);
  
  if (!isRecording || !recordingDirectory) {
    // console.log('Early return: not recording or no directory');
    return;
  }
  
  try {
    // Read all files in the directory
    const entries = await readDir(recordingDirectory);
    // console.log('Found entries in directory:', entries.length);
    
    // Extract timestamp pattern from current recording filename
    let timestampPrefix = '';
    if (recordingFilename) {
      console.log('Trying to match recording filename:', recordingFilename);
      const match = recordingFilename.match(/serial_recording_(\d+)/);
      if (match && match[1]) {
        timestampPrefix = match[1].substring(0, 10);
        console.log('Extracted timestamp prefix:', timestampPrefix);
      } else {
        console.log('Failed to extract timestamp from filename');
      }
    } else {
      console.log('WARNING: recordingFilename is empty');
    }
    
    // Find files matching the timestamp pattern
    let matchFound = false;
    console.log('Looking for files matching timestamp prefix:', timestampPrefix || 'ANY');
    
    for (const entry of entries) {
      console.log('Checking entry:', entry.name);
      
      // If we have a specific filename from the backend, use direct matching
      if (recordingFilename && entry.name === recordingFilename) {
        console.log('EXACT MATCH FOUND:', entry.name);
        matchFound = true;
        
        // Found the exact recording file
        const actualPath = `${recordingDirectory}/${entry.name}`;
        setActiveRecordingPath(actualPath);
        
        // Make sure we update the recordingFilename in case it was different
        setRecordingFilename(entry.name);
        
        // Update just this file's size without refreshing the entire list
        try {
          const fileStat = await stat(actualPath);
          console.log('Got file stats, size:', fileStat.size);
          
          // Look for the file in the current list
          const fileIndex = folderFiles.findIndex(f => f.path === actualPath);
          console.log('File index in folder list:', fileIndex);
          
          if (fileIndex >= 0) {
            // Get the modified date from FileInfo
            const fileModifiedDate = fileStat.mtime || fileStat.birthtime;
            
            // Only update properties if they've changed to avoid Vue re-renders
            const existingFile = folderFiles[fileIndex];
            
            // Update size if changed
            if (existingFile.rawSize !== fileStat.size) {
              // console.log('Updating file size from', existingFile.rawSize, 'to', fileStat.size);
              existingFile.size = formatFileSize(fileStat.size);
              existingFile.rawSize = fileStat.size;
            }
            
            // Update date if changed
            if (fileModifiedDate instanceof Date && 
                (!(existingFile.dateObject instanceof Date) || 
                 existingFile.dateObject.getTime() !== fileModifiedDate.getTime())) {
              // console.log('Updating file date');
              existingFile.modified = fileModifiedDate.toLocaleString();
              existingFile.dateObject = fileModifiedDate;
            }
            
            // Calculate and update duration
            if (fileStat.birthtime instanceof Date && fileModifiedDate instanceof Date) {
              const durationMs = fileModifiedDate.getTime() - fileStat.birthtime.getTime();
              if (durationMs > 0) {
                existingFile.duration = formatDuration(durationMs);
                console.log('Updated duration:', existingFile.duration);
              }
            }
            
            // Create a new array reference to trigger reactive updates
            updateFiles([...folderFiles]);
          } else {
            // If file is not in the list yet, add just this file without a full reload
            // console.log('Adding new file to list:', entry.name);
            const fileModifiedDate = fileStat.mtime || fileStat.birthtime;
            const newFile: RecordingFile = {
              name: entry.name,
              path: actualPath,
              size: formatFileSize(fileStat.size),
              modified: fileModifiedDate instanceof Date ? fileModifiedDate.toLocaleString() : 'Unknown',
              rawSize: fileStat.size,
              dateObject: fileModifiedDate,
              key: entry.name
            };
            
            // Add duration if available
            if (fileStat.birthtime instanceof Date && fileModifiedDate instanceof Date) {
              const durationMs = fileModifiedDate.getTime() - fileStat.birthtime.getTime();
              if (durationMs > 0) {
                newFile.duration = formatDuration(durationMs);
              }
            }
            
            // Add to the front of the array (newest file)
            folderFiles.unshift(newFile);
            updateFiles([...folderFiles]);
          }
          
          // Update the actual recording filename to match what's on disk
          setRecordingFilename(entry.name);
          return; // Found and updated the exact file
        } catch (e) {
          console.error('Error updating file info:', e);
        }
      }
      
      // For more lenient matching when exact filenames might be out of sync
      if (!matchFound && timestampPrefix && entry.name.includes(timestampPrefix)) {
        console.log('TIMESTAMP PREFIX MATCH FOUND:', entry.name);
        matchFound = true;
        
        // Set the recording path for this file
        const actualPath = `${recordingDirectory}/${entry.name}`;
        setActiveRecordingPath(actualPath);
        
        // Update the actual filename in the state
        setRecordingFilename(entry.name);
        
        // Update just this file's info
        try {
          const fileStat = await stat(actualPath);
          
          // Look for the file in the current list
          const fileIndex = folderFiles.findIndex(f => f.path === actualPath);
          
          if (fileIndex >= 0) {
            // Update existing file
            const existingFile = folderFiles[fileIndex];
            const fileModifiedDate = fileStat.mtime || fileStat.birthtime;
            
            // Only update changed properties
            if (existingFile.rawSize !== fileStat.size) {
              existingFile.size = formatFileSize(fileStat.size);
              existingFile.rawSize = fileStat.size;
            }
            
            if (fileModifiedDate instanceof Date) {
              existingFile.modified = fileModifiedDate.toLocaleString();
              existingFile.dateObject = fileModifiedDate;
              
              // Calculate and update duration
              if (fileStat.birthtime instanceof Date) {
                const durationMs = fileModifiedDate.getTime() - fileStat.birthtime.getTime();
                if (durationMs > 0) {
                  existingFile.duration = formatDuration(durationMs);
                }
              }
            }
            
            // Create a new array reference to trigger reactive updates
            updateFiles([...folderFiles]);
          } else {
            // Add new file to list
            const fileModifiedDate = fileStat.mtime || fileStat.birthtime;
            const newFile: RecordingFile = {
              name: entry.name,
              path: actualPath,
              size: formatFileSize(fileStat.size),
              modified: fileModifiedDate instanceof Date ? fileModifiedDate.toLocaleString() : 'Unknown',
              rawSize: fileStat.size,
              dateObject: fileModifiedDate,
              key: entry.name
            };
            
            // Add duration if available
            if (fileStat.birthtime instanceof Date && fileModifiedDate instanceof Date) {
              const durationMs = fileModifiedDate.getTime() - fileStat.birthtime.getTime();
              if (durationMs > 0) {
                newFile.duration = formatDuration(durationMs);
              }
            }
            
            // Add to the front of the array (newest file)
            folderFiles.unshift(newFile);
            updateFiles([...folderFiles]);
          }
          
          return; // Found and updated a matching file
        } catch (e) {
          console.error('Error updating file info:', e);
        }
      }
    }
    
    // If we made it here without finding a match, and we're recording, create a placeholder
    if (isRecording && recordingFilename && !matchFound) {
      console.log('Creating placeholder for unseen recording file:', recordingFilename);
      
      // Check if we already have a placeholder
      const existingIndex = folderFiles.findIndex(f => f.name === recordingFilename);
      if (existingIndex < 0) {
        // Create a placeholder entry for the file until it's visible in the filesystem
        folderFiles.unshift({
          name: recordingFilename,
          path: `${recordingDirectory}/${recordingFilename}`,
          size: '0 B',
          modified: new Date().toLocaleString(),
          duration: '0s',
          rawSize: 0,
          dateObject: new Date(),
          key: recordingFilename
        });
        
        updateFiles([...folderFiles]);
      }
    }
  } catch (error) {
    console.error('Error finding active recording file:', error);
  }
}

/**
 * Open file in file explorer or viewer
 */
export async function syncFile(filePath: string): Promise<void> {
  try {
    // On desktop platforms, show the file in folder
    // if (platform() !== 'android' && platform() !== 'ios') {
    //   await invoke('plugin:shell|open', { path: filePath });
    // } else {
      // On mobile, we might need a different approach
      // Possibly using a Tauri plugin or a mobile-specific way to share files
    alert('Opening files directly not supported on this platform yet');
    // }
  } catch (error) {
    console.error('Error opening file location:', error);
    throw new Error(`Unable to open file: ${error}`);
  }
}

/**
 * Upload file to a remote server or cloud storage
 */
export async function uploadFile(filePath: string): Promise<void> {
  try {
    // This is a placeholder for actual upload functionality
    // In a real implementation, you would:
    // 1. Read the file content
    // 2. Upload to a server/cloud storage using an API
    // 3. Show progress and status
    
    // For now, we'll just show a notification
    alert(`File upload feature will be implemented in a future version.\nSelected file: ${filePath}`);
    
    // Example of how it might be implemented with a Tauri command:
    // await invoke('upload_file_to_server', { 
    //   filePath, 
    //   destination: 'cloud_storage',
    //   credentials: { /* auth tokens, etc */ }
    // });
  } catch (error) {
    console.error('Error uploading file:', error);
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
  try {
    // Confirm deletion
    const confirmed = confirm(`Are you sure you want to delete ${filePath.split('/').pop()}?`);
    if (!confirmed) return;
    
    // Use Tauri FS plugin to remove the file
    await invoke('plugin:fs|remove_file', { path: filePath });
    
    // Refresh the file list
    await onComplete();
  } catch (error) {
    console.error('Error deleting file:', error);
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
