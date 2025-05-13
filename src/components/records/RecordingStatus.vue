<template>
  <div v-if="isRecording" class="mt-6 p-4 bg-gray-700 rounded-lg">
    <div class="flex justify-between items-center">
      <h3 class="text-lg font-semibold text-green-400">{{ $t('recording.inProgress') }}</h3>
      <div class="flex items-center gap-2">
        <div class="h-3 w-3 bg-green-400 rounded-full animate-pulse"></div>
        <span class="text-sm">{{ $t('recording.recordingStatus') }}</span>
      </div>
    </div>
    <p class="mt-4 text-sm">{{ $t('recording.savingTo') }} <span class="text-blue-300">{{ recordingFilename }}</span></p>
  </div>
</template>

<script lang="ts" setup>
defineProps({
  isRecording: {
    type: Boolean,
    required: true
  },
  recordingFilename: {
    type: String,
    required: true
  }
});
</script>
