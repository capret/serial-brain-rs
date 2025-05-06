// Helpers for working with active recording files
import { RecordingFile } from './types';
import { loadDirectoryFiles } from './fileLoader';

/**
 * Check if a filename is the current active recording file
 * 
 * This function handles potential timestamp discrepancies between frontend and backend
 * by matching the timestamps with a tolerance of a few seconds.
 */
export function isActiveRecordingFile(
  filename: string, 
  recordingFilename: string, 
  isRecording: boolean
): boolean {
  if (!isRecording || !recordingFilename || !filename) return false;
  
  // Exact match check
  if (filename === recordingFilename) return true;
  
  // Check for timestamp-based filenames with potential slight discrepancies
  const extractTimestamp = (name: string) => {
    const match = name.match(/serial_recording_(\d+)/);
    return match ? parseInt(match[1]) : null;
  };
  
  const fileTimestamp = extractTimestamp(filename);
  const recordingTimestamp = extractTimestamp(recordingFilename);
  
  if (fileTimestamp && recordingTimestamp) {
    // Allow for up to 3 seconds difference to account for timestamp differences
    // between frontend and backend
    const timeDiff = Math.abs(fileTimestamp - recordingTimestamp);
    return timeDiff < 3000; // 3 seconds in milliseconds
  }
  
  return false;
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
  if (!isRecording || !recordingFilename || !recordingDirectory) {
    return;
  }

  try {
    // First attempt: Look for exact file match in our existing files
    let foundInCurrentFiles = false;
    let fullPath = '';
    
    // Check if the recording file is already in our file list
    for (const file of folderFiles) {
      if (isActiveRecordingFile(file.name, recordingFilename, isRecording)) {
        foundInCurrentFiles = true;
        fullPath = file.path;
        
        // If the names didn't exactly match (timestamp differences), update the recording filename
        if (file.name !== recordingFilename) {
          console.log('Updating recording filename from:', recordingFilename, 'to:', file.name);
          setRecordingFilename(file.name);
        }
        
        break;
      }
    }

    // Second attempt: Get fresh files from the directory if not found
    if (!foundInCurrentFiles) {
      console.log('Active recording file not found in current files, refreshing directory');
      const freshFiles = await loadDirectoryFiles(recordingDirectory);
      
      // Look for matching file in the fresh files
      for (const file of freshFiles) {
        if (isActiveRecordingFile(file.name, recordingFilename, isRecording)) {
          fullPath = file.path;
          
          // If the names didn't exactly match (timestamp differences), update the recording filename
          if (file.name !== recordingFilename) {
            console.log('Updating recording filename from:', recordingFilename, 'to:', file.name);
            setRecordingFilename(file.name);
          }
          
          // Update the full file list
          updateFiles(freshFiles);
          break;
        }
      }
    }

    // Third attempt: If none of the above worked, check specifically for the file extension
    if (!fullPath) {
      console.log('Still unable to find recording file, trying with extension check');
      const expectedExtension = `.${selectedFormat.toLowerCase()}`;
      const nameWithoutExt = recordingFilename.replace(/\.[^/.]+$/, '');
      let fileWithCorrectExt = `${nameWithoutExt}${expectedExtension}`;
      
      // Set the filename with the correct extension
      if (fileWithCorrectExt !== recordingFilename) {
        console.log('Updating recording filename with correct extension:', fileWithCorrectExt);
        setRecordingFilename(fileWithCorrectExt);
      }
      
      fullPath = `${recordingDirectory}/${fileWithCorrectExt}`;
    }

    // Set the active recording path for watching
    if (fullPath) {
      setActiveRecordingPath(fullPath);
    }
  } catch (error) {
    console.error('Error finding/updating active recording file:', error);
  }
}
