<template>
  <div class="bg-gray-800 bg-opacity-60 rounded-lg p-6">
    <div class="flex justify-between items-start mb-6">
      <div>
        <h2 class="text-3xl font-bold text-blue-400">Signal Configuration</h2>
      </div>
      <div class="flex gap-3">
        <button
          class="bg-blue-600 hover:bg-blue-700 px-6 py-3 rounded-md font-semibold flex items-center gap-2 transition-all duration-300 transform hover:scale-105 shadow-lg">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polygon points="5 3 19 12 5 21 5 3"></polygon>
          </svg>
          Start Acquisition
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
</script>
