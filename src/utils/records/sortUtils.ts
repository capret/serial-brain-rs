// Utility functions for sorting files
import { RecordingFile } from './types';

/**
 * Sort files by date, with newest first
 */
export function sortFilesByNewest(files: RecordingFile[]): RecordingFile[] {
  return files.sort((a, b) => {
    // First try to compare based on dateObject
    if (a.dateObject instanceof Date && b.dateObject instanceof Date) {
      return b.dateObject.getTime() - a.dateObject.getTime();
    }
    
    // Fall back to filename timestamp
    const getTimestamp = (file: RecordingFile) => {
      const match = file.name.match(/serial_recording_(\d+)/);
      return match ? parseInt(match[1]) : 0;
    };
    
    return getTimestamp(b) - getTimestamp(a);
  });
}
