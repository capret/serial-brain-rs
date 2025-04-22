<template>
  <div class="relative w-28 h-12">
    <!-- Color rectangle, left-aligned -->
    <div class="absolute left-0 w-3 h-9 rounded-xl transition-all duration-200 ease-in-out hover:h-12 top-1/2 -translate-y-1/2">
      <input
        type="color"
        class="absolute inset-0 w-full h-full opacity-0 cursor-pointer rounded-xl"
        :value="internalColor"
        @input="onColorChange"
        @click.stop
      />
      <div
        class="w-full h-full rounded-xl"
        :style="{ backgroundColor: internalColor }"
      ></div>
    </div>

    <!-- Content rectangle, right-aligned and overlapping -->
    <div
      @click="onToggleVisibility"
      class="absolute top-0 right-0 w-24 h-12 rounded-lg flex flex-col justify-center px-3 cursor-pointer z-10 transition-transform duration-200 hover:scale-95"
      :class="props.visible ? 'bg-gray-700' : 'bg-gray-900'"
    >
      <p class="text-xs text-gray-400 m-0 truncate">{{ channelTitle }}</p>
      <p class="font-semibold m-0 truncate text-xs">{{ currentValue }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';

const props = defineProps<{ channelTitle: string; currentValue: number | string; color: string; visible: boolean; }>();
const emit = defineEmits<{ (event: 'color-change', color: string): void; (event: 'toggle-visibility'): void; }>();

const internalColor = ref<string>(props.color);

watch(() => props.color, newVal => { internalColor.value = newVal; });

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
/* Layout handled via Tailwind utility classes */
</style>
