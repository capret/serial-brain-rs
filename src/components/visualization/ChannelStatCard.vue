<template>
  <div :class="['p-3 rounded-lg w-30', props.visible ? 'bg-gray-700' : 'bg-gray-900']" @click="onToggleVisibility">
    <div>
      <div class="flex items-center gap-2">
        <label class="relative cursor-pointer" @click.stop>
          <span class="w-3 h-3 rounded-full inline-block" :style="{ backgroundColor: internalColor }"></span>
          <input type="color"
                 class="absolute top-0 left-0 w-3 h-3 opacity-0"
                 :value="internalColor"
                 @input="onColorChange" />
        </label>
        <p class="text-sm text-gray-400 m-0">{{ channelTitle }}</p>
      </div>
      <div class="pl-5">
        <p class="font-semibold m-0">{{ currentValue }}</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
const props = defineProps<{
  channelTitle: string;
  currentValue: number | string;
  color: string;
  visible: boolean;
}>();
const emit = defineEmits<{
  (event: 'color-change', color: string): void;
  (event: 'toggle-visibility'): void;
}>();
const internalColor = ref<string>(props.color);
watch(() => props.color, newVal => {
  internalColor.value = newVal;
});
function onColorChange(e: Event) {
  const newColor = (e.target as HTMLInputElement).value;
  internalColor.value = newColor;
  emit('color-change', newColor);
}
function onToggleVisibility() {
  emit('toggle-visibility');
}
</script>

<style scoped>
/* No custom styles needed */
</style>
