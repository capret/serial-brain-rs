<template>
  <div class="w-[300px] border-r border-gray-200 bg-gray-50 flex flex-col h-full">
    <div class="p-4 overflow-y-auto flex-1">
      <h2 class="text-lg font-medium mb-4">Signal Configuration</h2>
      <DataSourcePanel />
      <SerialSettingsPanel />
      <TCPSettingsPanel />
      <FakeDataSettingsPanel />
      <SignalFiltersPanel />
    </div>
    <div class="p-4 border-t border-gray-200">
      <button class="btn btn-primary w-full">
        <span class="material-symbols-outlined mr-2">play_arrow</span> Start Data Acquisition
      </button>
    </div>
  </div>
</template>

<script setup>
import DataSourcePanel from './DataSourcePanel.vue';
import SerialSettingsPanel from './SerialSettingsPanel.vue';
import TCPSettingsPanel from './TCPSettingsPanel.vue';
import FakeDataSettingsPanel from './FakeDataSettingsPanel.vue';
import SignalFiltersPanel from './SignalFiltersPanel.vue';
</script>

<style scoped>
.w-\[300px\] {
  width: 300px;
}

.border-r {
  border-right-width: 1px;
}

.border-t {
  border-top-width: 1px;
}

.border-gray-200 {
  border-color: rgb(229 231 235);
}

.p-4 {
  padding: 1rem;
}

.bg-gray-50 {
  background-color: rgb(249 250 251);
}

.text-lg {
  font-size: 1.125rem;
  line-height: 1.75rem;
}

.font-medium {
  font-weight: 500;
}

.mb-4 {
  margin-bottom: 1rem;
}

.w-full {
  width: 100%;
}

.mr-2 {
  margin-right: 0.5rem;
}

.overflow-y-auto {
  overflow-y: auto;
}

.flex-1 {
  flex: 1 1 0%;
}

.flex {
  display: flex;
}

.flex-col {
  flex-direction: column;
}

.h-full {
  height: 100%;
}
</style>
