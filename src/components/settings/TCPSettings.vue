<template>
  <div>
    <h3 class="text-lg font-semibold">TCP Connection Settings</h3>
    
    <div class="grid grid-cols-2 gap-4">
      <div>
        <label class="block text-sm mb-1">Host</label>
        <input type="text" class="w-full bg-gray-800 p-2 rounded" v-model="tcpSettings.host" placeholder="0.0.0.0" />
      </div>
      
      <div>
        <label class="block text-sm mb-1">Port</label>
        <input type="number" class="w-full bg-gray-800 p-2 rounded" v-model.number="tcpSettings.port" placeholder="8234" />
      </div>
    </div>
    
    <div class="mb-4">
      <label class="block text-sm mb-1">Protocol</label>
      <select class="w-full bg-gray-800 p-2 rounded" v-model="tcpSettings.protocol">
        <option value="tcp">TCP</option>
        <option value="udp">UDP</option>
      </select>
    </div>
    
    <!-- Auto-reconnect removed as the system is always in listening mode -->

    
    <!-- Connection status and status messages -->
    <div class="mt-4 bg-gray-900 p-3 rounded">
      <h4 class="text-sm font-medium mb-2">Connection Status:</h4>
      
      <!-- Auto-reconnect status removed as the system is always in listening mode -->

      
      <!-- Connection state indicator -->
      <div class="flex items-center mb-2">
        <div class="w-3 h-3 rounded-full mr-2" 
          :class="{
            'bg-green-500': tcpSettings.isConnected && hasClientConnected,
            'bg-yellow-500': tcpSettings.isConnected && !hasClientConnected,
            'bg-red-500': !tcpSettings.isConnected
          }"></div>
        <span class="text-xs">
          <span v-if="tcpSettings.isConnected && hasClientConnected">Client connected</span>
          <span v-else-if="tcpSettings.isConnected && !hasClientConnected">Listening for connections</span>
          <span v-else>Not connected</span>
        </span>
      </div>
      
      <!-- Current Host:Port display -->
      <div class="text-xs mb-2" v-if="tcpSettings.isConnected">
        <span class="text-gray-400">Configured endpoint: </span>
        <span class="text-blue-400">{{ tcpSettings.host }}:{{ tcpSettings.port }}</span>
      </div>
      
      <!-- Socket status messages -->
      <div ref="messagesRef" class="text-xs h-32 overflow-y-auto bg-gray-800 p-2 rounded flex-1">
        <div v-for="(message, index) in socketMessages" :key="index" 
          :class="{
            'text-green-400': message.includes('Connected from') || message.includes('successful'),
            'text-yellow-400': message.includes('listening'),
            'text-red-400': message.includes('failed') || message.includes('disconnected'),
            'mb-1': true
          }">
          {{ message }}
        </div>
      </div>
      
      <!-- Error display -->
      <div v-if="tcpSettings.lastError" class="text-red-400 text-xs mt-2">
        {{ tcpSettings.lastError }}
      </div>
    </div>
  </div>
</template>

<script setup>
import { tcpSettings, socketMessages, hasClientConnected, addSocketMessage } from '../../store/appState';
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue';
import { listen } from '@tauri-apps/api/event';

// Set up listener for socket status events
let unlistenSocketStatus;

// Reference to messages container for auto-scrolling
const messagesRef = ref(null);

// Reset connection state when port changes
watch(() => tcpSettings.port, (newPort, oldPort) => {
  if (newPort !== oldPort && oldPort !== 0) {
    console.log(`Port changed from ${oldPort} to ${newPort}, resetting connection state`);
    tcpSettings.resetConnectionState();
    tcpSettings.lastPort = newPort;
    // No need to handle reconnection since the system is always listening
  }
});

// Reset connection state when host changes
watch(() => tcpSettings.host, (newHost, oldHost) => {
  if (newHost !== oldHost && oldHost !== '') {
    console.log(`Host changed from ${oldHost} to ${newHost}, resetting connection state`);
    tcpSettings.resetConnectionState();
  }
});

onMounted(async () => {
  // Set up socket status event listener
  unlistenSocketStatus = await listen('socket_status', (event) => {
    // Use the global function to add messages
    addSocketMessage(event.payload);
    
    // Auto-scroll to bottom when new content is added
    nextTick(() => {
      if (messagesRef.value) {
        messagesRef.value.scrollTop = messagesRef.value.scrollHeight;
      }
    });
  });
});

onUnmounted(() => {
  if (unlistenSocketStatus) {
    unlistenSocketStatus();
  }
});
</script>
