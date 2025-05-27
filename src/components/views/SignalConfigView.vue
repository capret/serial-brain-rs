<template>
  <div class="bg-gray-800 bg-opacity-60 rounded-lg p-6 flex flex-col">
    <div class="flex flex-wrap justify-between items-start mb-6 gap-4">
      <div class="mb-2">
        <h2 class="text-3xl font-bold text-blue-400">{{ $t('signal.title') }}</h2>
      </div>
      <div class="flex w-full md:w-auto">
        <button
          @click="handleConnectionToggle"
          :disabled="isDisabled"
          :class="[
            buttonClass,
            'px-6 py-3 rounded-md font-semibold flex items-center justify-center gap-2 transition-all duration-300 transform hover:scale-105 shadow-lg w-full md:w-auto',
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
      <h3 class="text-lg font-semibold mb-4">{{ $t('signal.source') }}</h3>
      <div class="flex flex-wrap gap-4">
        <div 
          @click="onDataSourceChanged('serial')"
          class="bg-gray-700 p-4 rounded-lg cursor-pointer transition-all duration-300 transform hover:scale-[1.02] w-full md:w-[calc(50%-0.5rem)] lg:w-[calc(33.333%-0.75rem)]"
          :class="{'border-2 border-blue-500': selectedDataSource === 'serial'}"
        >
          <div>
            <label class="font-medium cursor-pointer">{{ $t('signal.serialConnection') }}</label>
          </div>
          <p class="text-xs text-gray-400 mt-1">{{ $t('signal.serialDescription') }}</p>
        </div>
        
        <div 
          @click="onDataSourceChanged('tcp')"
          class="bg-gray-700 p-4 rounded-lg cursor-pointer transition-all duration-300 transform hover:scale-[1.02] w-full md:w-[calc(50%-0.5rem)] lg:w-[calc(33.333%-0.75rem)]"
          :class="{'border-2 border-blue-500': selectedDataSource === 'tcp'}"
        >
          <div>
            <label class="font-medium cursor-pointer">{{ $t('signal.tcpConnection') }}</label>
          </div>
          <p class="text-xs text-gray-400 mt-1">{{ $t('signal.tcpDescription') }}</p>
        </div>
        
        <div 
          @click="onDataSourceChanged('fake')"
          class="bg-gray-700 p-4 rounded-lg cursor-pointer transition-all duration-300 transform hover:scale-[1.02] w-full md:w-[calc(50%-0.5rem)] lg:w-[calc(33.333%-0.75rem)]"
          :class="{'border-2 border-blue-500': selectedDataSource === 'fake'}"
        >
          <div>
            <label class="font-medium cursor-pointer">{{ $t('signal.fakeDataGenerator') }}</label>
          </div>
          <p class="text-xs text-gray-400 mt-1">{{ $t('signal.fakeDescription') }}</p>
        </div>
      </div>
    </div>
    
    <!-- Source Settings Section -->
    <div class="mb-6 flex flex-col flex-grow">
      <h3 class="text-lg font-semibold mb-4">{{ $t('signal.settings') }}</h3>
      
      <!-- Load the appropriate settings component based on selectedDataSource -->
      <div class="bg-gray-700 p-4 rounded-lg">
        <SerialSettings v-if="selectedDataSource === 'serial'" 
                      :settings="serialSettings" 
                      class="transition-opacity duration-300" />
        
        <TCPSettings v-if="selectedDataSource === 'tcp'" 
                    :settings="tcpSettings" 
                    class="transition-opacity duration-300" />
        
        <FakeDataSettings v-if="selectedDataSource === 'fake'" 
                        :settings="fakeDataSettings" 
                        class="transition-opacity duration-300" />
      </div>
    </div>
  </div>
</template>

<script setup>
import SerialSettings from '../settings/SerialSettings.vue';
import TCPSettings from '../settings/TCPSettings.vue';
import FakeDataSettings from '../settings/FakeDataSettings.vue';

import { computed, ref, onMounted, onUnmounted, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { connectionStatus, isRunning, tcpSettings, serialSettings, fakeDataSettings, fetchConnectionState } from '../../store/appState.ts';

// Initialize i18n
const { t } = useI18n();

const props = defineProps({
  selectedDataSource: {
    type: String,
    required: true
  }
});

const emit = defineEmits(['data-source-changed']);

function onDataSourceChanged(source) {
  emit('data-source-changed', source);
}

// Connection status stored globally

// Fetch the connection state from the backend when the component is mounted
onMounted(async () => {
  try {
    // Fetch the initial connection state from the backend
    await fetchConnectionState();
    
    console.log('Initial connection state loaded:', { 
      status: connectionStatus.value, 
      isRunning: isRunning.value 
    });
  } catch (error) {
    console.error('Error loading initial connection state:', error);
  }
});

// Button label based on status
const buttonLabel = computed(() => {
  switch (connectionStatus.value) {
    case 'connected': return t('signal.disconnect');
    case 'failed': return t('signal.reconnect');
    case 'connecting': return t('signal.connecting');
    default: return t('signal.connect');
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

// Reconnection timer reference
const reconnectTimer = ref(null);

// Function to connect to TCP socket
function connectToTcpSocket() {
  return invoke('connect_socket', {
    host: tcpSettings.host,
    port: tcpSettings.port
  });
}

// Function to attempt reconnection
async function attemptReconnect() {
  if (!tcpSettings.autoReconnect || connectionStatus.value === 'connected') {
    stopReconnecting();
    return;
  }
  
  tcpSettings.isReconnecting = true;
  tcpSettings.reconnectAttempts++;
  
  try {
    await connectToTcpSocket();
    connectionStatus.value = 'connected';
    tcpSettings.isConnected = true;
    tcpSettings.lastError = '';
    stopReconnecting();
  } catch (error) {
    tcpSettings.lastError = error.toString();
    connectionStatus.value = 'failed';
    tcpSettings.isConnected = false;
    
    // Schedule next attempt if auto-reconnect is still enabled
    if (tcpSettings.autoReconnect) {
      reconnectTimer.value = setTimeout(attemptReconnect, tcpSettings.reconnectInterval);
    }
  }
}

// Function to stop reconnection attempts
function stopReconnecting() {
  tcpSettings.isReconnecting = false;
  if (reconnectTimer.value) {
    clearTimeout(reconnectTimer.value);
    reconnectTimer.value = null;
  }
}

// Watch for changes in the auto-reconnect setting
watch(() => tcpSettings.autoReconnect, (newValue) => {
  // Only stop reconnecting if the setting is turned off
  if (!newValue) {
    stopReconnecting();
  }
  // Do NOT automatically trigger reconnection when the setting is turned on
  // Only reconnect when the user manually clicks the connect button
});

// Clean up on component unmount
onUnmounted(() => {
  stopReconnecting();
});

// Toggle connection/disconnection
async function handleConnectionToggle() {
  if (connectionStatus.value === 'connected') {
    invoke('stop_data_acquisition')
      .then(() => { 
        connectionStatus.value = 'disconnected'; 
        tcpSettings.isConnected = false;
        stopReconnecting();
      })
      .catch(() => { 
        connectionStatus.value = 'failed'; 
        tcpSettings.isConnected = false;
      });
  } else {
    connectionStatus.value = 'connecting';
    let cmd;
    
    if (props.selectedDataSource === 'serial') {
      cmd = invoke('connect_serial', {
        port: serialSettings.port,
        baudRate: serialSettings.baudRate,
        stopBits: serialSettings.stopBits,
        parity: serialSettings.parity,
        dataBits: serialSettings.dataBits
      });
    } else if (props.selectedDataSource === 'fake') {
      // First enable fake signal mode
      await invoke('toggle_fake_signal', {});
      
      cmd = invoke('start_fake_data', {
        config: {
          min_value: fakeDataSettings.minValue,
          max_value: fakeDataSettings.maxValue,
          frequency: fakeDataSettings.frequency,
          channel_count: fakeDataSettings.channelCount,
          waveform: fakeDataSettings.waveform
        }
      });
    } else if (props.selectedDataSource === 'tcp') {
      // Reset reconnection counters
      tcpSettings.reset();
      
      cmd = connectToTcpSocket();
      
      // If auto-reconnect is enabled, we'll handle reconnection attempts
      if (tcpSettings.autoReconnect) {
        cmd.catch((e) => {
          console.error(e);
          tcpSettings.lastError = e.toString();
          connectionStatus.value = 'failed';
          tcpSettings.isConnected = false;
          // Schedule first reconnection attempt
          reconnectTimer.value = setTimeout(attemptReconnect, tcpSettings.reconnectInterval);
        });
      }
    } else {
      cmd = Promise.reject('Unknown data source');
    }
    
    cmd
      .then(() => { 
        connectionStatus.value = 'connected'; 
        if (props.selectedDataSource === 'tcp') {
          tcpSettings.isConnected = true;
          tcpSettings.lastError = '';
        }
      })
      .catch((e) => {
        console.error(e);
        connectionStatus.value = 'failed';
        // TCP-specific error handling is done inside the TCP conditional block above
      });
  }
}
</script>
