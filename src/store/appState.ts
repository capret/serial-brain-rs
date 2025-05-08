import { ref, reactive, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// Connection status
export const connectionStatus = ref<string>('disconnected');
// Separate connection running state from visualization running state
export const isConnected = ref<boolean>(false);
// Visualization running state - this controls whether the chart is actively plotting
export const isRunning = ref<boolean>(false);
export const chartDataBuffer = reactive<number[][]>([]);

// Function to fetch the connection state from the backend
export async function fetchConnectionState(): Promise<void> {
  try {
    const state = await invoke<any>('get_signal_config_state');
    
    // Update the connection status
    connectionStatus.value = state.connectionStatus;
    // Update the connection state
    isConnected.value = state.isRunning;
    // Do NOT update isRunning here - that's controlled by the visualization component
    
    console.log('Connection state from backend:', state);
  } catch (error) {
    console.error('Error fetching connection state:', error);
  }
}

// Chart configuration
export const windowSize = ref<number>(5000); // Display window size (samples)

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

// Global recording settings to persist across views
export const recordingDirectory = ref<string>('');

// Load maxRecordingDuration from localStorage or use default
const savedDuration = localStorage.getItem('maxRecordingDuration');
export const maxRecordingDuration = ref<number>(savedDuration ? parseInt(savedDuration) : 30);

// Save maxRecordingDuration to localStorage when it changes
maxRecordingDuration.value = Math.max(1, maxRecordingDuration.value); // Ensure at least 1 minute
watch(maxRecordingDuration, (newValue: number) => {
  localStorage.setItem('maxRecordingDuration', newValue.toString());
}, { immediate: true });

export const recordingFormat = ref<string>('csv'); // Default format
export const autoStartRecording = ref<boolean>(false); // Default auto-start setting
