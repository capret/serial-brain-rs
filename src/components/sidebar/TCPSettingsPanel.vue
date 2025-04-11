<template>
  <details class="panel">
    <summary class="panel-header">
      <span class="font-medium">TCP Settings</span>
      <span class="material-symbols-outlined transition-transform group-open:rotate-180">expand_more</span>
    </summary>
    <div class="panel-body space-y-3">
      <div>
        <label class="form-label">Host</label>
        <input type="text" placeholder="localhost" class="form-input" />
      </div>
      <div>
        <label class="form-label">Port</label>
        <input type="number" placeholder="1234" class="form-input" />
      </div>
      <div>
        <label class="form-label">Protocol</label>
        <select class="form-select">
          <option>TCP</option>
          <option>UDP</option>
        </select>
      </div>
    </div>
  </details>
</template>

<script setup>
// Component logic for TCP settings
</script>
