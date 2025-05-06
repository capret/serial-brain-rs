// Re-export all record utility functions and types from the records module
// This provides backward compatibility with existing code

// Re-export types
export type { RecordingFile, UnsubscribeFn } from './records/types';

// Re-export all the functions
export {
  // Formatters
  formatFileSize,
  formatDuration,
  updateFileDuration,
  formatDate,
  formatTime
} from './records/formatters';

export {
  // File loading
  loadDirectoryFiles
} from './records/fileLoader';

export {
  // File watching
  setupDirectoryWatcher,
  setupRecordingFileWatcher
} from './records/fileWatcher';

export {
  // Recording helpers
  isActiveRecordingFile,
  findAndUpdateActiveRecordingFile
} from './records/recordingFileHelpers';

export {
  // File operations
  syncFile,
  uploadFile,
  deleteFile,
  updateFilesInPlace
} from './records/fileOperations';
