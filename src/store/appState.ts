import { ref, reactive } from 'vue';

// Connection status
export const connectionStatus = ref<string>('disconnected');
export const isRunning = ref<boolean>(false);
export const chartDataBuffer = reactive<number[][]>([]);

// TCP Connection Settings
export const tcpSettings = reactive({
  // Connection settings
  host: '0.0.0.0',
  port: 8234,
  protocol: 'tcp',
  
  // Auto-reconnect settings
  autoReconnect: false,
  reconnectInterval: 1000, // 1 second
  isReconnecting: false,
  reconnectAttempts: 0,
  maxReconnectAttempts: 0, // 0 means infinite attempts
  
  // Connection status
  isConnected: false,
  lastError: '',
  
  // Methods
  reset() {
    this.isReconnecting = false;
    this.reconnectAttempts = 0;
    this.lastError = '';
  }
});

// Serial Port Settings
export const serialSettings = reactive({
  port: '',
  baudRate: 460800,
  dataBits: 8,
  stopBits: 1,
  parity: 'none'
});

// Fake Data Generator Settings
export const fakeDataSettings = reactive({
  minValue: -100,
  maxValue: 100,
  frequency: 500,
  channelCount: 8,
  waveform: 'sine'
});
