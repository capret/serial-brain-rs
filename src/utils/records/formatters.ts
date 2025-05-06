// Utility functions for formatting data related to recordings
import { RecordingFile } from './types';

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
    
    return new Date().toLocaleTimeString();
  } catch (e) {
    console.error('Error formatting time:', e);
    return 'Unknown';
  }
}
