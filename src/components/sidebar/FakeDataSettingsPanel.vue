<template>
  <details class="panel" :open="isDataSourceFake">
    <summary class="panel-header">
      <span class="font-medium">Fake Data Settings</span>
      <span class="material-symbols-outlined transition-transform group-open:rotate-180">expand_more</span>
    </summary>
    <div class="panel-body space-y-3">
      <div>
        <label class="form-label">Min Value</label>
        <input type="number" v-model.number="config.minValue" placeholder="-10" class="form-input" />
      </div>
      <div>
        <label class="form-label">Max Value</label>
        <input type="number" v-model.number="config.maxValue" placeholder="10" class="form-input" />
      </div>
      <div>
        <label class="form-label">Frequency (Hz)</label>
        <input type="number" v-model.number="config.frequency" placeholder="1" min="0.1" max="1000" step="0.1" class="form-input" />
      </div>
      <div>
        <label class="form-label">Number of Channels</label>
        <input type="number" v-model.number="config.channelCount" placeholder="4" min="1" max="8" class="form-input" />
      </div>
      <div>
        <label class="form-label">Waveform</label>
        <select class="form-select" v-model="config.waveform">
          <option value="sine">Sine</option>
          <option value="square">Square</option>
          <option value="triangle">Triangle</option>
          <option value="sawtooth">Sawtooth</option>
          <option value="random">Random</option>
        </select>
      </div>
    </div>
  </details>
</template>

<script setup>
import { ref, defineExpose, inject, computed } from 'vue';

// Get the selected data source from parent or provide a default value
const dataSourcePanel = inject('dataSourcePanel', null);
const isDataSourceFake = computed(() => {
  if (dataSourcePanel) {
    return dataSourcePanel.getSelectedSource() === 'fake';
  }
  return false;
});

// Fake data configuration settings
const config = ref({
  minValue: -10,
  maxValue: 10,
  frequency: 5,
  channelCount: 4,
  waveform: 'sine'
});

// Expose configuration methods to parent components
defineExpose({
  getConfig: () => config.value
});
</script>
