import { reactive } from 'vue';

// Centralized channel config: colors and visibility
export const channelColors = reactive<string[]>([
  '#FF6384',
  '#36A2EB',
  '#FFCE56',
  '#4BC0C0',
  '#9966FF',
  '#FF9F40',
  '#E7E9ED',
  '#7CFFC4'
]);

// Persistent visibility state per channel
export const channelVisibility = reactive<boolean[]>(channelColors.map(() => true));
