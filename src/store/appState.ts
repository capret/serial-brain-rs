import { ref, reactive } from 'vue';

// Connection status
export const connectionStatus = ref<string>('disconnected');
export const isRunning = ref<boolean>(false);
export const chartDataBuffer = reactive<number[][]>([]);

// Socket status messages (for global access across pages)
export const socketMessages = reactive<string[]>([]);
export const hasClientConnected = ref<boolean>(false);

// Add a message to socket status log
export function addSocketMessage(message: string) {
  socketMessages.push(message);
  
  // Keep maximum of 50 messages
  if (socketMessages.length > 50) {
    socketMessages.shift();
  }
  
  // Update client connection status based on message content
  if (message.includes('Connected from')) {
    hasClientConnected.value = true;
  } else if (message.includes('Client disconnected')) {
    hasClientConnected.value = false;
  } else if (message.includes('Setup successful') && !message.includes('Connected from')) {
    // If setup is successful but no client is connected yet
    hasClientConnected.value = false;
  }
}

// Clear socket messages
export function clearSocketMessages() {
  socketMessages.length = 0;
  hasClientConnected.value = false;
}

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
  lastPort: 0,  // Track the last port used for connection
  
  // Methods
  reset() {
    this.isReconnecting = false;
    this.reconnectAttempts = 0;
    this.lastError = '';
  },
  
  // Reset connection state when changing ports
  resetConnectionState() {
    clearSocketMessages();
    hasClientConnected.value = false;
    this.isConnected = false;
  }
});

// Serial Port Settings
export const serialSettings = reactive({
  port: '',
  baudRate: 460800,
  dataBits: 8,
  stopBits: 1,
  parity: 'none',
  serialInfo: '',
  serialInfoBuffer: [],
  availablePorts: [],
  sendData: '',
  sendEndFlag: '\n'
});

// Fake Data Generator Settings
export const fakeDataSettings = reactive({
  minValue: -100,
  maxValue: 100,
  frequency: 500,
  channelCount: 8,
  waveform: 'sine'
});

// Global recording directory to persist selection across views
export const recordingDirectory = ref<string>('');
