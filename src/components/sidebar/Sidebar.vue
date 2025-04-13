<template>
  <div class="sidebar-container border-r border-gray-200 bg-gray-50 flex flex-col h-full">
    <div class="p-4 overflow-y-auto flex-1">
      <h2 class="text-lg font-medium mb-4">Signal Configuration</h2>
      
      <!-- Data Source Selection Panel -->
      <DataSourcePanel ref="dataSourcePanel" @source-changed="onDataSourceChanged" />
      
      <!-- Unified Data Source Settings Panel with Auto-scaling -->
      <details class="panel auto-scale-panel" open>
        <summary class="panel-header">
          <span class="font-medium">Data Source Settings</span>
          <span class="material-symbols-outlined transition-transform group-open:rotate-180">expand_more</span>
        </summary>
        <div class="panel-body space-y-3">
          
          <!-- Serial Settings -->
          <div v-if="selectedDataSource === 'serial'" class="space-y-3">
            <div>
              <label class="form-label">Port</label>
              <select class="form-select" v-model="serialSettings.port">
                <option value="" disabled>Select a port</option>
                <option v-for="p in serialSettings.availablePorts" :key="p" :value="p">{{ p }}</option>
              </select>
            </div>
            <div>
              <label class="form-label">Baud Rate</label>
              <select class="form-select" v-model.number="serialSettings.baudRate">
                <option value="9600">9600</option>
                <option value="19200">19200</option>
                <option value="38400">38400</option>
                <option value="57600">57600</option>
                <option value="115200">115200</option>
              </select>
            </div>
            <div>
              <label class="form-label">Stop Bits</label>
              <select class="form-select" v-model.number="serialSettings.stopBits">
                <option :value="1">1</option>
                <option :value="2">2</option>
              </select>
            </div>
          </div>
          
          <!-- TCP Settings -->
          <div v-if="selectedDataSource === 'tcp'" class="space-y-3">
            <div>
              <label class="form-label">Host</label>
              <input type="text" placeholder="localhost" class="form-input" v-model="tcpSettings.host" />
            </div>
            <div>
              <label class="form-label">Port</label>
              <input type="number" placeholder="1234" class="form-input" v-model.number="tcpSettings.port" />
            </div>
            <div>
              <label class="form-label">Protocol</label>
              <select class="form-select" v-model="tcpSettings.protocol">
                <option value="tcp">TCP</option>
                <option value="udp">UDP</option>
              </select>
            </div>
          </div>
          
          <!-- Fake Data Settings -->
          <div v-if="selectedDataSource === 'fake'" class="space-y-3">
            <div>
              <label class="form-label">Min Value</label>
              <input type="number" v-model.number="fakeDataSettings.minValue" placeholder="-10" class="form-input" />
            </div>
            <div>
              <label class="form-label">Max Value</label>
              <input type="number" v-model.number="fakeDataSettings.maxValue" placeholder="10" class="form-input" />
            </div>
            <div>
              <label class="form-label">Frequency (Hz)</label>
              <input type="number" v-model.number="fakeDataSettings.frequency" placeholder="1" min="0.1" max="1000" step="0.1" class="form-input" />
            </div>
            <div>
              <label class="form-label">Number of Channels</label>
              <input type="number" v-model.number="fakeDataSettings.channelCount" placeholder="4" min="1" max="8" class="form-input" />
            </div>
            <div>
              <label class="form-label">Waveform</label>
              <select class="form-select" v-model="fakeDataSettings.waveform">
                <option value="sine">Sine</option>
                <option value="square">Square</option>
                <option value="triangle">Triangle</option>
                <option value="sawtooth">Sawtooth</option>
                <option value="random">Random</option>
              </select>
            </div>
          </div>
        </div>
      </details>
      
      <!-- Send Data Panel (Only for Serial) -->
      <details v-if="selectedDataSource === 'serial'" class="panel mt-3">
        <summary class="panel-header">
          <span class="font-medium">Send Data</span>
          <span class="material-symbols-outlined transition-transform group-open:rotate-180">expand_more</span>
        </summary>
        <div class="panel-body space-y-3">
          <div>
            <label class="form-label">Data to Send</label>
            <input type="text" class="form-input" v-model="serialSettings.sendData" placeholder="Data to send" />
          </div>
          <div>
            <label class="form-label">End Character</label>
            <input type="text" class="form-input" v-model="serialSettings.sendEndFlag" placeholder="e.g., \n or other delimiter" />
          </div>
          <button class="btn btn-primary w-full" @click="sendSerialData">Send</button>
        </div>
      </details>
      
      <SignalFiltersPanel />
    </div>
    <div class="p-4 border-t border-gray-200">
      <button 
        class="btn btn-primary w-full flex items-center justify-center"
        @click="startDataAcquisition"
      >
        <span class="material-symbols-outlined mr-2">{{ isAcquiring ? 'stop' : 'play_arrow' }}</span> 
        {{ isAcquiring ? 'Stop Data Acquisition' : 'Start Data Acquisition' }}
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import DataSourcePanel from './DataSourcePanel.vue';
import SignalFiltersPanel from './SignalFiltersPanel.vue';

// The currently selected data source type
const selectedDataSource = ref('serial');
const isAcquiring = ref(false);
const dataSourcePanel = ref(null);

// Serial settings
const serialSettings = ref({
  port: '',
  baudRate: 9600,
  stopBits: 1,
  availablePorts: [],
  sendData: '',
  sendEndFlag: '\n'
});

// TCP settings
const tcpSettings = ref({
  host: 'localhost',
  port: 1234,
  protocol: 'tcp'
});

// Fake data settings
const fakeDataSettings = ref({
  minValue: -100,
  maxValue: 100,
  frequency: 500,
  channelCount: 8,
  waveform: 'sine'
});

// Called when data source type is changed
function onDataSourceChanged(source) {
  selectedDataSource.value = source;
}

// Fetch available serial ports
async function fetchSerialPorts() {
  try {
    const ports = await invoke('get_available_ports');
    serialSettings.value.availablePorts = ports;
    if (ports.length > 0 && !serialSettings.value.port) {
      serialSettings.value.port = ports[0];
    }
  } catch (error) {
    console.error("Error fetching serial ports:", error);
  }
}

// Connect to serial port
async function connectToSerialPort() {
  try {
    await invoke('connect_serial', {
      port: serialSettings.value.port,
      baudRate: serialSettings.value.baudRate,
      stopBits: serialSettings.value.stopBits,
    });
    console.log("Serial port connected");
    return true;
  } catch (error) {
    console.error("Error connecting to serial port:", error);
    return false;
  }
}

// Send data over serial connection
async function sendSerialData() {
  const message = serialSettings.value.sendData + serialSettings.value.sendEndFlag;
  try {
    await invoke('send_serial', { message });
    console.log("Message sent:", message);
  } catch (error) {
    console.error("Error sending message:", error);
  }
}

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
  try {
    switch (selectedDataSource.value) {
      case 'serial':
        // First connect to the serial port
        const connected = await connectToSerialPort();
        if (!connected) return;
        
        // Then start acquisition
        await invoke('start_serial_acquisition');
        break;
      
      case 'tcp':
        // TCP connection logic
        const tcpConfig = {
          host: tcpSettings.value.host,
          port: tcpSettings.value.port,
          protocol: tcpSettings.value.protocol
        };
        await invoke('start_tcp_acquisition', { config: tcpConfig });
        break;
      
      case 'fake':
        // Get fake data configuration
        const fakeConfig = {
          min_value: fakeDataSettings.value.minValue,
          max_value: fakeDataSettings.value.maxValue,
          frequency: fakeDataSettings.value.frequency,
          channel_count: fakeDataSettings.value.channelCount,
          waveform: fakeDataSettings.value.waveform
        };
        await invoke('start_fake_data', { config: fakeConfig });
        break;
      
      default:
        console.error('Unknown data source:', selectedDataSource.value);
        return;
    }
    
    isAcquiring.value = true;
    console.log(`Started data acquisition from source: ${selectedDataSource.value}`);
  } catch (error) {
    console.error('Error starting data acquisition:', error);
  }
}

// Initialize
onMounted(async () => {
  // Fetch available serial ports on component mount
  await fetchSerialPorts();
  
  // If DataSourcePanel has a specific source selected on mount, use that
  if (dataSourcePanel.value) {
    selectedDataSource.value = dataSourcePanel.value.getSelectedSource();
  }
});

</script>

<style scoped>
.auto-scale-panel {
  transition: all 0.3s ease;
  width: 250px;
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

.w-full {
  width: 100%;
}

.mr-2 {
  margin-right: 0.5rem;
}

.overflow-y-auto {
  overflow-y: auto;
}

.bg-red-500 {
  background-color: rgb(239 68 68);
}

.hover\:bg-red-600:hover {
  background-color: rgb(220 38 38);
}

.bg-green-500 {
  background-color: rgb(34 197 94);
}

.hover\:bg-green-600:hover {
  background-color: rgb(22 163 74);
}

.items-center {
  align-items: center;
}

.justify-center {
  justify-content: center;
}

.mb-4 {
  margin-bottom: 1rem;
}

.w-full {
  width: 100%;
}


</style>
