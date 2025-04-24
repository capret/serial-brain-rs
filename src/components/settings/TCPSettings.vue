<template>
  <div>
    <h3 class="text-lg font-semibold">TCP Connection Settings</h3>
    
    <div class="grid grid-cols-2 gap-4">
      <div>
        <label class="block text-sm mb-1">Host</label>
        <input type="text" class="w-full bg-gray-800 p-2 rounded" v-model="settings.host" placeholder="localhost" />
      </div>
      
      <div>
        <label class="block text-sm mb-1">Port</label>
        <input type="number" class="w-full bg-gray-800 p-2 rounded" v-model.number="settings.port" placeholder="1234" />
      </div>
    </div>
    
    <div>
      <label class="block text-sm mb-1">Protocol</label>
      <select class="w-full bg-gray-800 p-2 rounded" v-model="settings.protocol">
        <option value="tcp">TCP</option>
        <option value="udp">UDP</option>
      </select>
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
