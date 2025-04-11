<template>
  <details class="panel">
    <summary class="panel-header">
      <span class="font-medium">Serial Settings</span>
      <span class="material-symbols-outlined transition-transform group-open:rotate-180">expand_more</span>
    </summary>
    <div class="panel-body space-y-3 block">
      <div>
        <label class="form-label">Port</label>
        <select class="form-select">
          <option>COM1</option>
          <option>COM2</option>
          <option>COM3</option>
          <option>/dev/ttyUSB0</option>
          <option>/dev/ttyUSB1</option>
        </select>
      </div>
      <div>
        <label class="form-label">Baud Rate</label>
        <select class="form-select">
          <option>9600</option>
          <option>19200</option>
          <option>38400</option>
          <option>57600</option>
          <option>115200</option>
        </select>
      </div>
      <div>
        <label class="form-label">Data Bits</label>
        <select class="form-select">
          <option>8</option>
          <option>7</option>
          <option>6</option>
          <option>5</option>
        </select>
      </div>
      <div>
        <label class="form-label">Parity</label>
        <select class="form-select">
          <option>None</option>
          <option>Even</option>
          <option>Odd</option>
          <option>Mark</option>
          <option>Space</option>
        </select>
      </div>
      <div>
        <label class="form-label">Stop Bits</label>
        <select class="form-select">
          <option>1</option>
          <option>1.5</option>
          <option>2</option>
        </select>
      </div>
    </div>
  </details>
</template>

<script setup>
// Component logic for serial settings
</script>
