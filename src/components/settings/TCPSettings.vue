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
    
    <div class="flex items-center py-2 mb-2">
      <input 
        type="checkbox" 
        id="auto-reconnect" 
        class="mr-2 w-4 h-4" 
        v-model="tcpSettings.autoReconnect"
      />
      <label for="auto-reconnect" class="text-sm select-none">Keep trying to reconnect if connection fails</label>
    </div>
    
    <div v-if="tcpSettings.autoReconnect" class="mt-2 bg-gray-900 p-3 rounded text-xs">
      <div class="text-yellow-400 mb-1" v-if="tcpSettings.isReconnecting">
        Attempting to reconnect... ({{ tcpSettings.reconnectAttempts }} attempts)
      </div>
      <div v-else-if="tcpSettings.isConnected" class="text-green-400">
        Connected successfully
      </div>
      <div v-if="tcpSettings.lastError" class="text-red-400 text-xs mt-1">
        {{ tcpSettings.lastError }}
      </div>
    </div>
  </div>
</template>

<script setup>
import { tcpSettings } from '../../store/appState';
</script>
