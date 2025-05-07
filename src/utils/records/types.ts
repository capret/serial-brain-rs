// Define types used across the recording functionality

/**
 * Interface representing a recording file
 */
export interface RecordingFile {
  name: string;
  path: string;
  size: string;
  modified: string;
  duration?: string; // Optional duration field
  rawSize: number;
  dateObject: Date | null;
  key: string;
  hasDetailedMetadata?: boolean; // Flag to track if detailed metadata has been loaded
}

/**
 * Define the unsubscribe function type that's returned by the watchImmediate function
 */
export type UnsubscribeFn = () => void;
