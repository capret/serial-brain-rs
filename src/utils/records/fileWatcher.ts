// Functions for watching files and directories for changes
import { watchImmediate } from '@tauri-apps/plugin-fs';
import { UnsubscribeFn } from './types';

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
    // Cleanup any existing watcher
    
    // Set up a new watcher
    const unsubscribe = await watchImmediate(
      directoryPath,
      async (event) => {
        // Handle the type property correctly based on Tauri's API
        const eventType = String(event.type);
        if (['add', 'remove', 'modify'].includes(eventType)) {
          console.log(`Directory change detected (${eventType}), refreshing files...`);
          await onChange();
        }
      },
      { recursive: true } // Watch nested directories too
    );
    
    // Store the unsubscribe function
    setUnsubscribe(unsubscribe);
    console.log('Directory watcher set up successfully');
    
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
  // Clean up existing watcher if there is one
  if (currentUnsubscribe) {
    try {
      currentUnsubscribe();
      console.log('Previous file watcher unsubscribed');
    } catch (e) {
      console.warn('Failed to unsubscribe previous watcher:', e);
    }
  }
  
  if (!filePath) {
    return () => {}; // Return a no-op if no path
  }
  
  try {
    console.log('Setting up file watcher for:', filePath);
    
    // Set up a new watcher
    const unsubscribe = await watchImmediate(
      filePath,
      async (event) => {
        // Handle the type property correctly based on Tauri's API
        const eventType = String(event.type);
        if (eventType === 'modify') {
          console.log('Recording file modified, updating...');
          await onFileChange();
        }
      }
    );
    
    return unsubscribe;
  } catch (error) {
    console.error('Error setting up recording file watcher:', error);
    return () => {}; // Return a no-op function on error
  }
}
