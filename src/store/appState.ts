import { ref, reactive } from 'vue';

export const connectionStatus = ref<string>('disconnected');
export const isRunning = ref<boolean>(false);
export const chartDataBuffer = reactive<number[][]>([]);
