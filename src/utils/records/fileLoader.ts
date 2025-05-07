// Functions for loading and processing recording files
import { stat, readDir, BaseDirectory } from '@tauri-apps/plugin-fs';
import { RecordingFile } from './types';
import { formatFileSize, updateFileDuration } from './formatters';

/**
 * Load basic file information (lightweight) without detailed stats
 * @param directoryPath The directory path to read files from
 * @param baseDir Optional BaseDirectory to use (e.g., BaseDirectory.Document)
 */
export async function loadBasicFileInfo(
  directoryPath: string,
  baseDir?: BaseDirectory
): Promise<RecordingFile[]> {
  if (!directoryPath) return [];
  
  try {
    // Read all files in the directory
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
    
    // Create lightweight file objects with just basic info
    const basicFiles = recordingEntries.map(entry => {
      const fullPath = `${directoryPath}/${entry.name}`;
      
      // Extract timestamp from filename for basic sorting
      let timestamp = 0;
      const match = entry.name.match(/serial_recording_(\d+)/);
      if (match && match[1]) {
        timestamp = parseInt(match[1]);
      }
      
      // Create a basic file object with minimal info
      const file: RecordingFile = {
        name: entry.name,
        path: fullPath,
        rawSize: 0, // Will be populated later
        size: 'Loading...', 
        modified: timestamp ? new Date(timestamp).toLocaleString() : 'Unknown',
        dateObject: timestamp ? new Date(timestamp) : null,
        key: fullPath,
        hasDetailedMetadata: false // Track if we've loaded details
      };
      
      return file;
    });
    
    // Sort files by timestamp, newest first
    return basicFiles.sort((a, b) => {
      const getTimestamp = (file: RecordingFile) => {
        const match = file.name.match(/serial_recording_(\d+)/);
        return match ? parseInt(match[1]) : 0;
      };
      
      return getTimestamp(b) - getTimestamp(a);
    });
    
  } catch (error) {
    console.error('Error loading basic file info:', error);
    return [];
  }
}

/**
 * Load all recording files from a directory with full metadata (legacy function)
 * @param directoryPath The directory path to read files from
 * @param baseDir Optional BaseDirectory to use (e.g., BaseDirectory.Document)
 * @deprecated Use loadBasicFileInfo and loadFileMetadata instead for better performance
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
    
        // Mark files as having detailed metadata
    validFiles.forEach(file => {
      file.hasDetailedMetadata = true;
    });
    
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

/**
 * Load detailed metadata for specific files
 * @param files Array of files to load metadata for
 * @param baseDir Optional BaseDirectory to use
 * @returns Promise with the same files with updated metadata
 */
export async function loadFileMetadata(
  files: RecordingFile[],
  baseDir?: BaseDirectory
): Promise<RecordingFile[]> {
  if (!files || files.length === 0) return [];
  
  try {
    // Get file stats for each file in parallel
    const updatedFiles = await Promise.all(
      files.map(async file => {
        // Skip if already has detailed metadata
        if (file.hasDetailedMetadata) return file;
        
        try {
          // Get file stats
          const fileStat = baseDir ? 
            await stat(file.path, { baseDir }) : 
            await stat(file.path);
          
          // Update file with detailed metadata
          file.rawSize = fileStat.size;
          file.size = formatFileSize(fileStat.size);
          
          // Get modified date
          const modifiedDate = fileStat.mtime || fileStat.birthtime;
          if (modifiedDate) {
            const dateObj = new Date(modifiedDate);
            file.modified = dateObj.toLocaleString();
            file.dateObject = dateObj;
          }
          
          // Calculate duration
          if (fileStat.birthtime && file.dateObject) {
            const birthtime = new Date(fileStat.birthtime);
            updateFileDuration(file, birthtime, file.dateObject);
          } else {
            // Extract timestamp from filename
            const match = file.name.match(/serial_recording_(\d+)/);
            if (match && match[1]) {
              const timestamp = parseInt(match[1]);
              const startDate = new Date(timestamp);
              
              // Handle null case explicitly for type safety
              let endDate: Date;
              if (file.dateObject) {
                endDate = file.dateObject;
              } else if (modifiedDate) {
                endDate = new Date(modifiedDate);
              } else {
                // Fallback if no dates available
                endDate = new Date();
              }
              
              updateFileDuration(file, startDate, endDate);
            }
          }
          
          // Mark as having detailed metadata
          file.hasDetailedMetadata = true;
          
          return file;
        } catch (error) {
          console.error(`Error loading metadata for ${file.name}:`, error);
          return file; // Return original file on error
        }
      })
    );
    
    return updatedFiles;
  } catch (error) {
    console.error('Error loading file metadata:', error);
    return files; // Return original files on error
  }
}
