// Functions for loading and processing recording files
import { stat, readDir, BaseDirectory } from '@tauri-apps/plugin-fs';
import { RecordingFile } from './types';
import { formatFileSize, updateFileDuration } from './formatters';

/**
 * Load all recording files from a directory
 * @param directoryPath The directory path to read files from
 * @param baseDir Optional BaseDirectory to use (e.g., BaseDirectory.Document)
 */
export async function loadDirectoryFiles(
  directoryPath: string,
  baseDir?: BaseDirectory
): Promise<RecordingFile[]> {
  if (!directoryPath) return [];
  
  try {
    // Read all files in the directory using baseDir if provided
    const entries = baseDir ? 
      await readDir(directoryPath, { baseDir }) : 
      await readDir(directoryPath);
    
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
          // Construct full path, adjusting based on whether we're using BaseDirectory
          const fullPath = baseDir ? 
            `${directoryPath}/${entry.name}` : // When using BaseDirectory, the path is relative
            `${directoryPath}/${entry.name}`; // For absolute paths (legacy support)
            
          // Get file stats (pass baseDir if it's provided)
          const fileStat = baseDir ? 
            await stat(fullPath, { baseDir }) : 
            await stat(fullPath);
          
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
