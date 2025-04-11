<template>
  <details class="panel">
    <summary class="panel-header">
      <span class="font-medium">Fake Data Settings</span>
      <span class="material-symbols-outlined transition-transform group-open:rotate-180">expand_more</span>
    </summary>
    <div class="panel-body space-y-3">
      <div>
        <label class="form-label">Min Value</label>
        <input type="number" placeholder="-10" class="form-input" />
      </div>
      <div>
        <label class="form-label">Max Value</label>
        <input type="number" placeholder="10" class="form-input" />
      </div>
      <div>
        <label class="form-label">Frequency (Hz)</label>
        <input type="number" placeholder="1" class="form-input" />
      </div>
      <div>
        <label class="form-label">Number of Channels</label>
        <input type="number" placeholder="4" min="1" max="8" class="form-input" />
      </div>
      <div>
        <label class="form-label">Waveform</label>
        <select class="form-select">
          <option>Sine</option>
          <option>Square</option>
          <option>Triangle</option>
          <option>Sawtooth</option>
          <option>Random</option>
        </select>
      </div>
    </div>
  </details>
</template>

<script setup>
// Component logic for fake data settings
</script>
