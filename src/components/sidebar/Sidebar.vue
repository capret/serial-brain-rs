<template>
  <div class="w-[300px] border-r border-gray-200 bg-gray-50 flex flex-col h-full">
    <div class="p-4 overflow-y-auto flex-1">
      <h2 class="text-lg font-medium mb-4">Signal Configuration</h2>
      <DataSourcePanel ref="dataSourcePanel" />
      <SerialSettingsPanel ref="serialSettingsPanel" />
      <TCPSettingsPanel />
      <FakeDataSettingsPanel ref="fakeDataSettingsPanel" />
      <SignalFiltersPanel />
    </div>
    <div class="p-4 border-t border-gray-200">
      <button 
        class="btn btn-primary w-full flex items-center justify-center"
        @click="startDataAcquisition"
        :class="{'bg-red-500 hover:bg-red-600': isAcquiring, 'bg-green-500 hover:bg-green-600': !isAcquiring}"
      >
        <span class="material-symbols-outlined mr-2">{{ isAcquiring ? 'stop' : 'play_arrow' }}</span> 
        {{ isAcquiring ? 'Stop Data Acquisition' : 'Start Data Acquisition' }}
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, provide, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import DataSourcePanel from './DataSourcePanel.vue';
import SerialSettingsPanel from './SerialSettingsPanel.vue';
import TCPSettingsPanel from './TCPSettingsPanel.vue';
import FakeDataSettingsPanel from './FakeDataSettingsPanel.vue';
import SignalFiltersPanel from './SignalFiltersPanel.vue';

const dataSourcePanel = ref(null);
const serialSettingsPanel = ref(null);
const fakeDataSettingsPanel = ref(null);
const isAcquiring = ref(false);

// Provide dataSourcePanel to child components
onMounted(() => {
  provide('dataSourcePanel', dataSourcePanel.value);
});

async function startDataAcquisition() {
  if (isAcquiring.value) {
    // Stop data acquisition using the dedicated command
    try {
      await invoke('stop_data_acquisition');
      isAcquiring.value = false;
      console.log('Data acquisition stopped');
    } catch (error) {
      console.error('Error stopping data acquisition:', error);
    }
    return;
  }

  // Start data acquisition based on the selected source
  const dataSource = dataSourcePanel.value.getSelectedSource();
  
  try {
    switch (dataSource) {
      case 'serial':
        // First connect to the serial port
        await serialSettingsPanel.value.connect();
        // Then start acquisition
        await invoke('start_serial_acquisition');
        break;
      
      case 'tcp':
        // TCP connection logic would go here
        await invoke('start_tcp_acquisition');
        break;
      
      case 'fake':
        // Get fake data configuration from the dedicated panel
        const fakeConfig = fakeDataSettingsPanel.value.getConfig();
        await invoke('start_fake_data', { 
          config: fakeConfig
        });
        break;
      
      default:
        console.error('Unknown data source:', dataSource);
        return;
    }
    
    isAcquiring.value = true;
    console.log(`Started data acquisition from source: ${dataSource}`);
  } catch (error) {
    console.error('Error starting data acquisition:', error);
  }
}
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

.flex {
  display: flex;
}

.flex-col {
  flex-direction: column;
}

.h-full {
  height: 100%;
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

.mb-4 {
  margin-bottom: 1rem;
}

.w-full {
  width: 100%;
}


</style>
