<template>
  <div class="flex flex-wrap justify-between items-start mb-6">
    <div class="max-[800px]:w-full">
      <h2 class="text-3xl font-bold text-blue-400">{{ $t('recording.setup') }}</h2>
    </div>
    <div class="flex gap-3 max-[800px]:w-full max-[800px]:mt-4">
      <button 
        v-if="!isRecording"
        @click="startRecording"
        :disabled="!recordingDirectory || connectionStatus !== 'connected'"
        class="bg-blue-600 hover:bg-blue-700 px-6 py-3 rounded-md font-semibold flex items-center justify-center gap-2 transition-all duration-300 transform hover:scale-105 shadow-lg max-[800px]:w-full"
        :class="{ 'opacity-50 cursor-not-allowed': !recordingDirectory || connectionStatus !== 'connected' }">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
          stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polygon points="5 3 19 12 5 21 5 3"></polygon>
        </svg>
        {{ $t('streaming.start') }}
      </button>
      <button 
        v-else
        @click="stopRecording"
        class="bg-red-600 hover:bg-red-700 px-6 py-3 rounded-md font-semibold flex items-center justify-center gap-2 transition-all duration-300 transform hover:scale-105 shadow-lg max-[800px]:w-full">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
          stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="6" y="6" width="12" height="12"></rect>
        </svg>
        {{ $t('streaming.stop') }}
      </button>
    </div>
  </div>

  <div class="space-y-6">
    <div>
      <h3 class="text-lg font-semibold mb-4">{{ $t('recording.storageLocation') }}</h3>
      <div class="mt-2">
        <div class="flex">
          <input 
            type="text" 
            :placeholder="recordingDirectory ? recordingDirectory : $t('recording.noFolderSelected')" 
            :value="recordingDirectory"
            readonly
            class="bg-gray-700 px-3 py-2 rounded-md flex-grow text-gray-300" />
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
defineProps({
  isRecording: {
    type: Boolean,
    required: true
  },
  recordingDirectory: {
    type: String,
    required: true
  },
  connectionStatus: {
    type: String,
    required: true
  }
});

const emit = defineEmits(['start-recording', 'stop-recording']);

function startRecording() {
  emit('start-recording');
}

function stopRecording() {
  emit('stop-recording');
}
</script>
