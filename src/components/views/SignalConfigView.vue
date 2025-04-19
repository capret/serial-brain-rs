<template>
  <div class="bg-gray-800 bg-opacity-60 rounded-lg p-6">
    <div class="flex justify-between items-start mb-6">
      <div>
        <h2 class="text-3xl font-bold text-blue-400">Signal Configuration</h2>
      </div>
      <div class="flex gap-3">
        <button
          @click="handleConnectionToggle"
          :disabled="isDisabled"
          :class="[
            buttonClass,
            'px-6 py-3 rounded-md font-semibold flex items-center gap-2 transition-all duration-300 transform hover:scale-105 shadow-lg',
            { 'opacity-50 cursor-not-allowed': isDisabled }
          ]"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polygon points="5 3 19 12 5 21 5 3"></polygon>
          </svg>
          {{ buttonLabel }}
        </button>
      </div>
    </div>
    
    <!-- Data Source Selection Panel -->
    <div class="mb-6">
      <h3 class="text-lg font-semibold mb-4">Select Data Source</h3>
      <div class="grid grid-cols-3 gap-4">
        <div 
          @click="onDataSourceChanged('serial')"
          class="bg-gray-700 p-4 rounded-lg cursor-pointer transition-all duration-300 transform hover:scale-[1.02]"
          :class="{'border-2 border-blue-500': selectedDataSource === 'serial'}"
        >
          <div>
            <label class="font-medium cursor-pointer">Serial Connection</label>
          </div>
          <p class="text-xs text-gray-400 mt-1">Connect to devices via USB serial port</p>
        </div>
        
        <div 
          @click="onDataSourceChanged('tcp')"
          class="bg-gray-700 p-4 rounded-lg cursor-pointer transition-all duration-300 transform hover:scale-[1.02]"
          :class="{'border-2 border-blue-500': selectedDataSource === 'tcp'}"
        >
          <div>
            <label class="font-medium cursor-pointer">TCP Connection</label>
          </div>
          <p class="text-xs text-gray-400 mt-1">Connect to devices over network</p>
        </div>
        
        <div 
          @click="onDataSourceChanged('fake')"
          class="bg-gray-700 p-4 rounded-lg cursor-pointer transition-all duration-300 transform hover:scale-[1.02]"
          :class="{'border-2 border-blue-500': selectedDataSource === 'fake'}"
        >
          <div>
            <label class="font-medium cursor-pointer">Fake Data Generator</label>
          </div>
          <p class="text-xs text-gray-400 mt-1">Generate test signals for development</p>
        </div>
      </div>
    </div>
    
    <!-- Settings Container with Fixed Height -->
    <div class="mb-6">
      <h3 class="text-lg font-semibold mb-4">Source Settings</h3>
      <div class="bg-gray-700 rounded-lg min-h-[400px] relative overflow-hidden">
        <!-- Load the appropriate settings component based on selectedDataSource -->
        <SerialSettings v-show="selectedDataSource === 'serial'" 
                       :settings="serialSettings" 
                       class="p-4 space-y-4 absolute inset-0 transition-opacity duration-300 overflow-y-auto"
                       :class="{'opacity-100 z-10': selectedDataSource === 'serial', 'opacity-0 z-0': selectedDataSource !== 'serial'}" />
        
        <TCPSettings v-show="selectedDataSource === 'tcp'" 
                    :settings="tcpSettings" 
                    class="p-4 space-y-4 absolute inset-0 transition-opacity duration-300 overflow-y-auto"
                    :class="{'opacity-100 z-10': selectedDataSource === 'tcp', 'opacity-0 z-0': selectedDataSource !== 'tcp'}" />
        
        <FakeDataSettings v-show="selectedDataSource === 'fake'" 
                         :settings="fakeDataSettings" 
                         class="p-4 space-y-4 absolute inset-0 transition-opacity duration-300 overflow-y-auto"
                         :class="{'opacity-100 z-10': selectedDataSource === 'fake', 'opacity-0 z-0': selectedDataSource !== 'fake'}" />
      </div>
    </div>
  </div>
</template>

<script setup>
import SerialSettings from '../settings/SerialSettings.vue';
import TCPSettings from '../settings/TCPSettings.vue';
import FakeDataSettings from '../settings/FakeDataSettings.vue';

import { computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { connectionStatus } from '../../store/appState';

const props = defineProps({
  selectedDataSource: {
    type: String,
    required: true
  },
  serialSettings: {
    type: Object,
    required: true
  },
  tcpSettings: {
    type: Object,
    required: true
  },
  fakeDataSettings: {
    type: Object,
    required: true
  }
});

const emit = defineEmits(['data-source-changed']);

function onDataSourceChanged(source) {
  emit('data-source-changed', source);
}

// Connection status stored globally

// Button label based on status
const buttonLabel = computed(() => {
  switch (connectionStatus.value) {
    case 'connected': return 'Disconnect';
    case 'failed': return 'Reconnect';
    case 'connecting': return 'Connecting...';
    default: return 'Connect';
  }
});

// Button color classes based on status
const buttonClass = computed(() => {
  switch (connectionStatus.value) {
    case 'connected': return 'bg-red-600 hover:bg-red-700';
    case 'failed': return 'bg-yellow-500 hover:bg-yellow-600';
    case 'connecting': return 'bg-gray-500';
    default: return 'bg-blue-600 hover:bg-blue-700';
  }
});

// Disable while connecting
const isDisabled = computed(() => connectionStatus.value === 'connecting');

// Toggle connection/disconnection
function handleConnectionToggle() {
  if (connectionStatus.value === 'connected') {
    invoke('stop_data_acquisition')
      .then(() => { connectionStatus.value = 'disconnected'; })
      .catch(() => { connectionStatus.value = 'failed'; });
  } else {
    connectionStatus.value = 'connecting';
    let cmd;
    if (props.selectedDataSource === 'serial') {
      cmd = invoke('connect_serial', {
        port: props.serialSettings.port,
        baud_rate: props.serialSettings.baudRate,
        stop_bits: props.serialSettings.stopBits
      });
    } else if (props.selectedDataSource === 'fake') {
      cmd = invoke('start_fake_data', {
        config: {
          min_value: props.fakeDataSettings.minValue,
          max_value: props.fakeDataSettings.maxValue,
          frequency: props.fakeDataSettings.frequency,
          channel_count: props.fakeDataSettings.channelCount,
          waveform: props.fakeDataSettings.waveform
        }
      });
    } else if (props.selectedDataSource === 'tcp') {
      cmd = Promise.reject('TCP connect not implemented');
    } else {
      cmd = Promise.reject('Unknown data source');
    }
    cmd
      .then(() => { connectionStatus.value = 'connected'; })
      .catch((e) => {
        console.error(e);
        connectionStatus.value = 'failed';
      });
  }
}
</script>
