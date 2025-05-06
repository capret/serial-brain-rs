// Functions for loading and processing recording files
import { stat, readDir } from '@tauri-apps/plugin-fs';
import { RecordingFile } from './types';
import { formatFileSize, updateFileDuration } from './formatters';

/**
 * Load all recording files from a directory
 */
export async function loadDirectoryFiles(
  directoryPath: string
): Promise<RecordingFile[]> {
  if (!directoryPath) return [];
  
  try {
    // Read all files in the directory
    const entries = await readDir(directoryPath);
    
    // Filter for recording files (CSV, JSON, binary)
    const recordingEntries = entries.filter(entry => {
      const name = entry.name.toLowerCase();
      return name.endsWith('.csv') || 
             name.endsWith('.json') || 
             name.endsWith('.bin');
    });
    
    // Get file stats for each recording file
    const filesWithStats = await Promise.all(
      recordingEntries.map(async entry => {
        try {
          const fullPath = `${directoryPath}/${entry.name}`;
          const fileStat = await stat(fullPath);
          
          // Get the modified date from FileInfo
          const modifiedDate = fileStat.mtime || fileStat.birthtime;
          let dateObj = null;
          
          try {
            if (modifiedDate) {
              dateObj = new Date(modifiedDate);
            }
          } catch (e) {
            console.error('Error parsing date:', e);
          }
          
          // Create file object
          const file: RecordingFile = {
            name: entry.name,
            path: fullPath,
            rawSize: fileStat.size,
            size: formatFileSize(fileStat.size),
            modified: dateObj ? dateObj.toLocaleString() : 'Unknown',
            dateObject: dateObj,
            key: fullPath
          };
          
          // Try to determine creation time for calculating duration
          if (fileStat.birthtime && dateObj) {
            const birthtime = new Date(fileStat.birthtime);
            updateFileDuration(file, birthtime, dateObj);
          } else {
            // Fall back to filename timestamp if no valid dates
            const getTimestamp = (file: RecordingFile) => {
              const match = file.name.match(/serial_recording_(\d+)/);
              return match ? parseInt(match[1]) : 0;
            };
            const timestamp = getTimestamp(file);
            if (timestamp > 0) {
              const creationDate = new Date(timestamp);
              updateFileDuration(file, creationDate, dateObj);
            }
          }
          
          return file;
        } catch (error) {
          console.error(`Error getting stats for ${entry.name}:`, error);
          return null;
        }
      })
    );
    
    // Filter out any failed entries (null values)
    const validFiles = filesWithStats.filter(file => file !== null) as RecordingFile[];
    
    // Sort files by date, newest first
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
    return [];
  }
}
