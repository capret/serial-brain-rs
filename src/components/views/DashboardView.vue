<template>
  <div class="bg-gray-800 bg-opacity-60 rounded-lg p-6 space-y-6">
    <div class="flex flex-wrap gap-6">
      <StreamingView class="flex-1 min-w-[320px]" />
      <SignalConfigView
        class="flex-1 min-w-[320px]"
        :selected-data-source="selectedDataSource"
        :serial-settings="serialSettings"
        :tcp-settings="tcpSettings"
        :fake-data-settings="fakeDataSettings"
        @data-source-changed="onDataSourceChanged"
      />
    </div>
    <div class="flex gap-4">
      <button @click="connectAll" class="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-md text-white">
        Connect Both
      </button>
      <button
        @click="toggleRecording"
        :class="isRecording ? 'bg-red-600 hover:bg-red-700' : 'bg-green-600 hover:bg-green-700'"
        class="px-4 py-2 rounded-md text-white"
      >
        {{ isRecording ? 'Stop Recording' : 'Record Both' }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { mkdir, BaseDirectory } from '@tauri-apps/plugin-fs';
import * as path from '@tauri-apps/api/path';
import StreamingView from './StreamingView.vue';
import SignalConfigView from './SignalConfigView.vue';
import { serialSettings, tcpSettings, fakeDataSettings } from '../../store/appState';

const selectedDataSource = ref('fake');
function onDataSourceChanged(source: 'serial' | 'tcp' | 'fake') {
  selectedDataSource.value = source;
}

const streamUrl = ref('http://192.168.1.123:81/stream');
const isRecording = ref(false);

async function connectAll() {
  try {
    await invoke('start_streaming', { path: streamUrl.value, fake: false });

    if (selectedDataSource.value === 'serial') {
      await invoke('connect_serial', {
        port: serialSettings.port,
        baudRate: serialSettings.baudRate,
        stopBits: serialSettings.stopBits,
        parity: serialSettings.parity,
        dataBits: serialSettings.dataBits
      });
    } else if (selectedDataSource.value === 'tcp') {
      await invoke('connect_socket', { host: tcpSettings.host, port: tcpSettings.port });
    } else {
      await invoke('start_fake_data', {
        config: {
          min_value: fakeDataSettings.minValue,
          max_value: fakeDataSettings.maxValue,
          frequency: fakeDataSettings.frequency,
          channel_count: fakeDataSettings.channelCount,
          waveform: fakeDataSettings.waveform
        }
      });
    }
  } catch (err) {
    console.error('Error connecting:', err);
  }
}

async function toggleRecording() {
  if (isRecording.value) {
    await invoke('stop_stream_recording');
    await invoke('stop_recording');
    isRecording.value = false;
  } else {
    try {
      const documentDir = await path.documentDir();
      try { await mkdir('video_data', { baseDir: BaseDirectory.Document }); } catch (_) {}
      const videoDir = await path.join(documentDir, 'video_data');
      const ts = new Date().toISOString().replace(/[:.]/g, '-');
      const videoFile = await path.join(videoDir, `video_${ts}.mp4`);
      await invoke('start_stream_recording', { filePath: videoFile });

      try { await mkdir('signal_data', { baseDir: BaseDirectory.Document }); } catch (_) {}
      const signalDir = await path.join(documentDir, 'signal_data');
      await invoke('start_recording', { format: 'csv', directory: signalDir, maxDurationMinutes: 30, autoStart: false });
      isRecording.value = true;
    } catch (err) {
      console.error('Error starting recording:', err);
    }
  }
}
</script>
