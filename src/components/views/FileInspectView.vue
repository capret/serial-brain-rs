<template>
  <div class="bg-gray-800 bg-opacity-60 rounded-lg p-6">
    <h2 class="text-3xl font-bold text-blue-400 mb-6">{{ $t('fileInspect.title') }}</h2>

    <!-- Recorded Files Section -->
    <RecordedFilesList
      :recording-directory="recordingDirectory"
      :is-recording="isRecording"
      :recording-filename="recordingFilename"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import * as path from '@tauri-apps/api/path';
import { exists, mkdir, BaseDirectory } from '@tauri-apps/plugin-fs';
import { recordingDirectory } from '../../store/appState';
import { RecordedFilesList } from '../records';

// Initialize i18n
useI18n();

// State variables
const isRecording = ref<boolean>(false);
const recordingFilename = ref<string>('');

// Initialize component
onMounted(async () => {
  try {
    // Set up the recordings directory
    await selectDirectory();

    // Check if we're currently recording
    try {
      interface RecordingStatusResponse {
        isRecording: boolean;
        filename: string;
      }

      const recordingStatus = await invoke('get_app_state', {
        category: 'recording',
        key: 'status'
      }) as RecordingStatusResponse;

      if (recordingStatus) {
        isRecording.value = recordingStatus.isRecording;
        recordingFilename.value = recordingStatus.filename;

        console.log('Recording status:', isRecording.value ? 'Active' : 'Inactive');
        if (recordingFilename.value) {
          console.log('Current recording filename:', recordingFilename.value);
        }
      }
    } catch (error) {
      console.error('Failed to get recording status:', error);
    }
  } catch (error) {
    console.error('Error initializing file inspect view:', error);
  }
});

// Use Document directory with signal_data folder for recordings
async function selectDirectory(): Promise<void> {
  try {
    // Get the document directory full path
    const documentDir = await path.documentDir();

    // Check if signal_data directory exists
    const folderExists = await exists('signal_data', {
      baseDir: BaseDirectory.Document,
    });

    // Create the directory if it doesn't exist
    if (!folderExists) {
      console.log('Creating signal_data directory in Documents');
      await mkdir('signal_data', {
        baseDir: BaseDirectory.Document,
      });
    }

    // Join the document dir with signal_data folder to get the full path
    const fullPath = await path.join(documentDir, 'signal_data');

    // Set the recording directory path
    recordingDirectory.value = fullPath;
    console.log('Using full path for file inspection:', fullPath);
  } catch (error) {
    console.error('Error setting up signal_data directory:', error);
  }
}
</script>
