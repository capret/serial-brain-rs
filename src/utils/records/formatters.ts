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
  // Handle undefined/null case explicitly
  if (milliseconds === undefined || milliseconds === null) return '--';
  
  // For zero or negative values, return 00:00:00
  if (milliseconds <= 0) return '00:00:00';
  
  // Convert to seconds, ensuring it's at least 1 for very small durations
  const totalSeconds = Math.max(1, Math.floor(milliseconds / 1000));
  
  // Calculate hours, minutes, and seconds
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  
  // Format with leading zeros
  const formattedHours = hours.toString().padStart(2, '0');
  const formattedMinutes = minutes.toString().padStart(2, '0');
  const formattedSeconds = seconds.toString().padStart(2, '0');
  
  // Return in format HH:MM:SS
  return `${formattedHours}:${formattedMinutes}:${formattedSeconds}`;
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
