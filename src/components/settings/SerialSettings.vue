<template>
  <div>
    <h3 class="text-lg font-semibold">Serial Connection Settings</h3>
    
    <div class="grid grid-cols-2 gap-4">
      <div>
        <label class="block text-sm mb-1">Port</label>
        <select class="w-full bg-gray-800 p-2 rounded" v-model="settings.port">
          <option value="" disabled>Select a port</option>
          <option v-for="p in settings.availablePorts" :key="p" :value="p">{{ p }}</option>
          <option value="COM1">COM1</option>
          <option value="COM2">COM2</option>
          <option value="/dev/tty1">/dev/tty1</option>
        </select>
      </div>
      
      <div>
        <label class="block text-sm mb-1">Baud Rate</label>
        <select class="w-full bg-gray-800 p-2 rounded" v-model.number="settings.baudRate">
          <option value="9600">9600</option>
          <option value="19200">19200</option>
          <option value="38400">38400</option>
          <option value="57600">57600</option>
          <option value="115200">115200</option>
        </select>
      </div>
    </div>
    
    <div>
      <label class="block text-sm mb-1">Stop Bits</label>
      <select class="w-full bg-gray-800 p-2 rounded" v-model.number="settings.stopBits">
        <option :value="1">1</option>
        <option :value="2">2</option>
      </select>
    </div>
    
    <div class="mt-6">
      <h4 class="text-md font-medium mb-2">Send Data</h4>
      <div class="space-y-2">
        <input type="text" class="w-full bg-gray-800 p-2 rounded" v-model="settings.sendData" placeholder="Data to send" />
        <div class="flex gap-2">
          <input type="text" class="w-1/3 bg-gray-800 p-2 rounded" v-model="settings.sendEndFlag" placeholder="End character" />
          <button class="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-md transition-colors duration-300 flex-grow">
            Send
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>

const props = defineProps({
  settings: {
    type: Object,
    required: true
  }
});
</script>
