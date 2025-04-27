<template>
  <div class="flex flex-col h-full">
    <h3 class="text-lg font-semibold">Serial Connection Settings</h3>
    
    <div class="flex flex-col gap-4">
      <div class="flex flex-col gap-4 w-full">
        <!-- Serial settings form -->
        <div class="flex flex-wrap gap-4 w-full items-start">
          <div class="flex flex-col">
            <label class="block text-sm mb-1">Port</label>
            <div class="flex gap-2">
              <select class="w-[8rem] bg-gray-800 p-2 rounded" v-model="settings.port">
                <option value="" disabled>Select a port</option>
                <option v-for="p in settings.availablePorts" :key="p" :value="p">{{ p }}</option>
              </select>
              <button type="button" @click="refreshPorts" class="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded">Refresh</button>
            </div>
          </div>
          <div class="flex flex-col">
            <label class="block text-sm mb-1">Baud Rate</label>
            <select class="w-[8rem] bg-gray-800 p-2 rounded" v-model.number="settings.baudRate">
              <option value="9600">9600</option>
              <option value="19200">19200</option>
              <option value="38400">38400</option>
              <option value="57600">57600</option>
              <option value="115200">115200</option>
              <option value="460800">460800</option>
            </select>
          </div>
          <div class="flex flex-col">
            <label class="block text-sm mb-1">Stop Bits</label>
            <select class="w-[8rem] bg-gray-800 p-2 rounded" v-model.number="settings.stopBits">
              <option :value="1">1</option>
              <option :value="2">2</option>
            </select>
          </div>
          <div class="flex flex-col">
            <label class="block text-sm mb-1">Parity</label>
            <select class="w-[8rem] bg-gray-800 p-2 rounded" v-model="settings.parity">
              <option value="none">None</option>
              <option value="odd">Odd</option>
              <option value="even">Even</option>
            </select>
          </div>
          <div class="flex flex-col">
            <label class="block text-sm mb-1">Data Bits</label>
            <select class="w-[8rem] bg-gray-800 p-2 rounded" v-model.number="settings.dataBits">
              <option :value="5">5</option>
              <option :value="6">6</option>
              <option :value="7">7</option>
              <option :value="8">8</option>
            </select>
          </div>
        </div>
        <div class="mt-4 w-full flex flex-col">
          <h4 class="text-md font-medium mb-2">Send Data</h4>
          <div class="space-y-2">
            <input type="text" class="w-full bg-gray-800 p-2 rounded" v-model="settings.sendData" placeholder="Data to send" />
            <div class="flex gap-2">
              <input type="text" class="w-1/3 bg-gray-800 p-2 rounded" v-model="settings.sendEndFlag" placeholder="End character" />
              <button class="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-md transition-colors duration-300 flex-grow">Send</button>
            </div>
          </div>
        </div>
      </div>
      <div class="bg-gray-900 p-3 rounded text-white flex flex-col w-full h-[250px] mt-2">
        <h4 class="text-sm font-medium mb-2">Serial Connection Status:</h4>
        
        <!-- Connection state indicator -->
        <div class="flex items-center mb-2">
          <div class="w-3 h-3 rounded-full mr-2" 
            :class="{
              'bg-green-500': settings.isConnected,
              'bg-red-500': !settings.isConnected
            }"></div>
          <span class="text-xs">
            <span v-if="settings.isConnected">Connected to {{ settings.port }}</span>
            <span v-else>Not connected</span>
          </span>
        </div>
        
        <!-- Current Port display -->
        <div class="text-xs mb-2" v-if="settings.isConnected">
          <span class="text-gray-400">Configuration: </span>
          <span class="text-blue-400">{{ settings.port }}, {{ settings.baudRate }} baud, {{ settings.dataBits }}{{ settings.parity.charAt(0).toUpperCase() }}{{ settings.stopBits }}</span>
        </div>
        
        <!-- Serial status messages -->
        <div ref="infoRef" class="text-xs overflow-y-auto bg-gray-800 p-2 rounded flex-1">
          <div v-for="(message, index) in settings.serialInfoBuffer" :key="index" 
            :class="{
              'text-green-400': message.includes('Connected') || message.includes('successful'),
              'text-yellow-400': message.includes('waiting') || message.includes('opened'),
              'text-red-400': message.includes('failed') || message.includes('disconnected') || message.includes('error'),
              'mb-1': true
            }">
            {{ message }}
          </div>
          <div v-if="!settings.serialInfoBuffer.length" class="text-gray-400">No serial info.</div>
        </div>
        
        <!-- Error display -->
        <div v-if="settings.lastError" class="text-red-400 text-xs mt-2">
          {{ settings.lastError }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

const props = defineProps({
  settings: {
    type: Object,
    required: true
  }
});

// Reference to the serial info container for auto-scrolling
const infoRef = ref<HTMLElement | null>(null);

function refreshPorts() {
  invoke('get_available_ports')
    .then((ports) => { props.settings.availablePorts = ports; })
    .catch((e) => console.error('Failed to refresh ports', e));
}

onMounted(() => {
  refreshPorts();
  // listen for text info from backend
  const cleanup = listen<string>('serial_info', event => {
    // Only add non-empty messages
    if (event.payload && event.payload.trim().length > 0) {
      props.settings.serialInfoBuffer.push(event.payload);
      // Limit the buffer size to prevent memory issues
      if (props.settings.serialInfoBuffer.length > 1000) {
        props.settings.serialInfoBuffer.splice(0, props.settings.serialInfoBuffer.length - 1000);
      }
      // Auto-scroll to bottom when new content is added
      nextTick(() => {
        if (infoRef.value) {
          infoRef.value.scrollTop = infoRef.value.scrollHeight;
        }
      });
    }
  });
  
  // Cleanup listener when component is unmounted
  onUnmounted(() => {
    cleanup.then(unlisten => unlisten());
  });
});
</script>
