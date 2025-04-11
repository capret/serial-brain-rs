<template>
  <details class="panel" open>
    <summary class="panel-header">
      <span class="font-medium">Serial Settings</span>
      <span class="material-symbols-outlined transition-transform group-open:rotate-180">expand_more</span>
    </summary>
    <div class="panel-body space-y-3 block">
      <div>
        <label class="form-label">Port</label>
        <select class="form-select" v-model="port">
          <option value="" disabled>Select a port</option>
          <option v-for="p in availablePorts" :key="p" :value="p">{{ p }}</option>
        </select>
      </div>
      <div>
        <label class="form-label">Baud Rate</label>
        <select class="form-select" v-model.number="baudRate">
          <option value="9600">9600</option>
          <option value="19200">19200</option>
          <option value="38400">38400</option>
          <option value="57600">57600</option>
          <option value="115200">115200</option>
        </select>
      </div>
      <div>
        <label class="form-label">Stop Bits</label>
        <select class="form-select" v-model.number="stopBits">
          <option :value="1">1</option>
          <option :value="2">2</option>
        </select>
      </div>
      <div>
        <button class="btn bg-primary text-white w-full py-2 rounded" @click="connect">Connect to Port</button>
      </div>
    </div>
  </details>

  <details class="panel mt-3">
    <summary class="panel-header">
      <span class="font-medium">Send Data</span>
      <span class="material-symbols-outlined transition-transform group-open:rotate-180">expand_more</span>
    </summary>
    <div class="panel-body space-y-3 block">
      <div>
        <label class="form-label">Data to Send</label>
        <input type="text" class="form-input" v-model="sendPart" placeholder="Data to send" />
      </div>
      <div>
        <label class="form-label">End Character</label>
        <input type="text" class="form-input" v-model="sendEndFlag" placeholder="e.g., \n or other delimiter" />
      </div>
      <button class="btn bg-success text-white w-full py-2 rounded" @click="sendData">Send</button>
    </div>
  </details>
</template>

<script setup>
import { ref, onMounted, defineExpose } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// Serial connection and application state.
const port = ref('');
const baudRate = ref(9600);
const stopBits = ref(1);
const availablePorts = ref([]);
const sendPart = ref('');
const sendEndFlag = ref('');

// Serial port commands.
async function connect() {
  try {
    await invoke('connect_serial', {
      port: port.value,
      baudRate: baudRate.value,
      stopBits: stopBits.value,
    });
    console.log("Serial port connected");
  } catch (error) {
    console.error("Error connecting to serial port:", error);
  }
}

async function sendData() {
  const message = sendPart.value + sendEndFlag.value;
  try {
    await invoke('send_serial', { message });
    console.log("Message sent:", message);
  } catch (error) {
    console.error("Error sending message:", error);
  }
}

async function fetchPorts() {
  try {
    const ports = await invoke('get_available_ports');
    availablePorts.value = ports;
    if (ports.length > 0 && !port.value) {
      port.value = ports[0];
    }
  } catch (error) {
    console.error("Error fetching serial ports:", error);
  }
}

// Set up listeners and fetch available ports
onMounted(async () => {
  await fetchPorts();
});

// Expose methods for parent components
defineExpose({
  connect,
  sendData,
  getSettings: () => ({
    port: port.value,
    baudRate: baudRate.value,
    stopBits: stopBits.value
  })
});
</script>

