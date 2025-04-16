<template>
  <div id="main" class="flex flex-col h-screen w-screen overflow-hidden">
    <div
      class="bg-gradient-to-br from-gray-900 to-gray-800 rounded-lg shadow-2xl p-8 font-sans text-white flex flex-col flex-grow overflow-hidden">
      <header class="flex justify-between items-center mb-8" id="titlebar">
        <h1 class="text-3xl font-bold tracking-tight">
          Serial<span class="text-blue-400">Brain</span>
        </h1>
        <div class="flex items-center gap-3 z-10">
          <button
            class="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-md 
                flex items-center gap-2 transition-all duration-300 transform hover:scale-105">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
              stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="feather feather-refresh-cw">
              <polyline points="23 4 23 10 17 10"></polyline>
              <polyline points="1 20 1 14 7 14"></polyline>
              <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
            </svg>
            Refresh
          </button>
          <button
            class="bg-gray-700 hover:bg-gray-600 px-4 py-2 rounded-md flex items-center gap-2 transition-all duration-300 transform hover:scale-105">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
              stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="feather feather-settings">
              <circle cx="12" cy="12" r="3"></circle>
              <path
                d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z">
              </path>
            </svg>
            Settings
          </button>
        </div>
        <div>
          <div class="flex items-center gap-2 ml-2">
            <button id="titlebar-minimize"
              class="titlebar-button text-gray-400 hover:text-white p-1 transition-colors duration-200">
              <svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 24 24" fill="none"
                stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="5" y1="12" x2="19" y2="12"></line>
              </svg>
            </button>
            <button id="titlebar-maximize"
              class="titlebar-button text-gray-400 hover:text-white p-1 transition-colors duration-200">
              <svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 24 24" fill="none"
                stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
              </svg>
            </button>
            <button id="titlebar-close"
              class="titlebar-button text-gray-400 hover:text-white p-1 transition-colors duration-200">
              <svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 24 24" fill="none"
                stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </button>
          </div>
        </div>
      </header>
      <div class="flex gap-8 flex-grow h-full overflow-hidden">
        <div class="w-72 flex-shrink-0 bg-gray-800 bg-opacity-60 rounded-lg p-5 overflow-y-auto">
          <h2 class="text-xl font-semibold mb-4">Available Signal</h2>
          <div class="space-y-3">
            <div
              class="bg-blue-600 bg-opacity-20 border-l-4 border-blue-500 p-4 rounded-r-lg cursor-pointer hover:bg-blue-600 hover:bg-opacity-30 transition-all duration-300 transform hover:translate-x-1">
              <h3 class="font-bold">Fake Reader</h3>
              <p class="text-sm text-gray-300">Signal Simulation</p>
            </div>
            <div
              class="bg-gray-700 p-4 rounded-lg cursor-pointer hover:bg-gray-600 transition-all duration-300 transform hover:translate-x-1">
              <h3 class="font-bold">Serial Reader</h3>
              <p class="text-sm text-gray-300">USB Connection</p>
            </div>
            <div
              class="bg-gray-700 p-4 rounded-lg cursor-pointer hover:bg-gray-600 transition-all duration-300 transform hover:translate-x-1">
              <h3 class="font-bold">Socket Reader</h3>
              <p class="text-sm text-gray-300">TCP Connection</p>
            </div>
          </div>
          
          <!-- Toggle Controls -->
          <div class="mt-8">
            <h2 class="text-xl font-semibold mb-4">View Controls</h2>
            <div class="space-y-3">
              <div
                @click="activeView = 'visualization'"
                :class="[`p-4 rounded-lg cursor-pointer transition-all duration-300 transform hover:translate-x-1`,
                  activeView === 'visualization' 
                    ? 'bg-blue-600 bg-opacity-20 border-l-4 border-blue-500 rounded-r-lg' 
                    : 'bg-gray-700 hover:bg-gray-600']">
                <div class="flex items-center gap-2">
                  <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" 
                    stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"></polyline>
                  </svg>
                  <h3 class="font-bold">Visualization</h3>
                </div>
              </div>
              
              <div
                @click="activeView = 'signal'"
                :class="[`p-4 rounded-lg cursor-pointer transition-all duration-300 transform hover:translate-x-1`,
                  activeView === 'signal' 
                    ? 'bg-blue-600 bg-opacity-20 border-l-4 border-blue-500 rounded-r-lg' 
                    : 'bg-gray-700 hover:bg-gray-600']">
                <div class="flex items-center gap-2">
                  <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" 
                    stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M22 12h-4l-3 9L9 3l-3 9H2"></path>
                  </svg>
                  <h3 class="font-bold">Signal</h3>
                </div>
              </div>
              
              <div
                @click="activeView = 'filters'"
                :class="[`p-4 rounded-lg cursor-pointer transition-all duration-300 transform hover:translate-x-1`,
                  activeView === 'filters' 
                    ? 'bg-blue-600 bg-opacity-20 border-l-4 border-blue-500 rounded-r-lg' 
                    : 'bg-gray-700 hover:bg-gray-600']">
                <div class="flex items-center gap-2">
                  <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" 
                    stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"></polygon>
                  </svg>
                  <h3 class="font-bold">Filters</h3>
                </div>
              </div>
              
              <div
                @click="activeView = 'folder'"
                :class="[`p-4 rounded-lg cursor-pointer transition-all duration-300 transform hover:translate-x-1`,
                  activeView === 'folder' 
                    ? 'bg-blue-600 bg-opacity-20 border-l-4 border-blue-500 rounded-r-lg' 
                    : 'bg-gray-700 hover:bg-gray-600']">
                <div class="flex items-center gap-2">
                  <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" 
                    stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
                  </svg>
                  <h3 class="font-bold">Recording</h3>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div class="flex-grow space-y-6 overflow-y-auto h-full pr-2 max-h-full">
          <!-- Visualization View -->
          <div v-if="activeView === 'visualization'" class="bg-gray-800 bg-opacity-60 rounded-lg p-6">
            <div class="flex justify-between items-start mb-6">
              <div>
                <h2 class="text-3xl font-bold text-blue-400">Visualization</h2>
              </div>
              <div class="flex gap-3">
                <button
                  class="bg-blue-600 hover:bg-blue-700 px-6 py-3 rounded-md font-semibold flex items-center gap-2 transition-all duration-300 transform hover:scale-105 shadow-lg">
                  <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24"
                    fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                    class="feather feather-play">
                    <polygon points="5 3 19 12 5 21 5 3"></polygon>
                  </svg>
                  Launch
                </button>
                <details class="relative inline-block">
                  <summary
                    class="bg-gray-700 hover:bg-gray-600 px-3 py-3 rounded-md cursor-pointer transition-all duration-300 list-none flex items-center justify-center">
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
                      stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <circle cx="12" cy="12" r="1"></circle>
                      <circle cx="19" cy="12" r="1"></circle>
                      <circle cx="5" cy="12" r="1"></circle>
                    </svg>
                  </summary>
                  <div
                    class="absolute right-0 mt-2 w-48 bg-gray-700 rounded-md shadow-xl z-20 overflow-hidden text-sm border border-gray-600">
                    <a href="#" class="block px-4 py-2 hover:bg-gray-600 transition-colors duration-200">
                      Export Data
                    </a>
                    <a href="#" class="block px-4 py-2 hover:bg-gray-600 transition-colors duration-200">
                      Report Issue
                    </a>
                  </div>
                </details>
              </div>
            </div>
            <div class="mb-6">
              <h3 class="text-lg font-semibold mb-2">Display Window:</h3>
              <SignalVisualization />
            </div>
          </div>
          
          <!-- Signal Source View -->
          <div v-if="activeView === 'signal'" class="bg-gray-800 bg-opacity-60 rounded-lg p-6">
            <div class="flex justify-between items-start mb-6">
              <div>
                <h2 class="text-3xl font-bold text-blue-400">Signal Configuration</h2>
              </div>
              <div class="flex gap-3">
                <button
                  class="bg-blue-600 hover:bg-blue-700 px-6 py-3 rounded-md font-semibold flex items-center gap-2 transition-all duration-300 transform hover:scale-105 shadow-lg">
                  <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
                    stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polygon points="5 3 19 12 5 21 5 3"></polygon>
                  </svg>
                  Start Acquisition
                </button>
              </div>
            </div>
            
            <!-- Data Source Selection Panel -->
            <div class="mb-6">
              <h3 class="text-lg font-semibold mb-4">Select Data Source</h3>
              <div class="grid grid-cols-3 gap-4">
                <div 
                  @click="selectedDataSource = 'serial'"
                  class="bg-gray-700 p-4 rounded-lg cursor-pointer transition-all duration-300 transform hover:scale-[1.02]"
                  :class="{'border-2 border-blue-500': selectedDataSource === 'serial'}"
                >
                  <div>
                    <label class="font-medium cursor-pointer">Serial Connection</label>
                  </div>
                  <p class="text-xs text-gray-400 mt-1">Connect to devices via USB serial port</p>
                </div>
                
                <div 
                  @click="selectedDataSource = 'tcp'"
                  class="bg-gray-700 p-4 rounded-lg cursor-pointer transition-all duration-300 transform hover:scale-[1.02]"
                  :class="{'border-2 border-blue-500': selectedDataSource === 'tcp'}"
                >
                  <div>
                    <label class="font-medium cursor-pointer">TCP Connection</label>
                  </div>
                  <p class="text-xs text-gray-400 mt-1">Connect to devices over network</p>
                </div>
                
                <div 
                  @click="selectedDataSource = 'fake'"
                  class="bg-gray-700 p-4 rounded-lg cursor-pointer transition-all duration-300 transform hover:scale-[1.02]"
                  :class="{'border-2 border-blue-500': selectedDataSource === 'fake'}"
                >
                  <div>
                    <label class="font-medium cursor-pointer">Fake Data Generator</label>
                  </div>
                  <p class="text-xs text-gray-400 mt-1">Generate test signals for development</p>
                </div>
              </div>
            </div>
            
            <!-- Settings Container with Fixed Height -->
            <div class="mb-6">
              <h3 class="text-lg font-semibold mb-4">Source Settings</h3>
              <div class="bg-gray-700 rounded-lg min-h-[400px] relative overflow-hidden">
                <!-- Serial Settings -->
                <div v-show="selectedDataSource === 'serial'" class="p-4 space-y-4 absolute inset-0 transition-opacity duration-300 overflow-y-auto"
                  :class="{'opacity-100 z-10': selectedDataSource === 'serial', 'opacity-0 z-0': selectedDataSource !== 'serial'}">
                  <h3 class="text-lg font-semibold">Serial Connection Settings</h3>
                  
                  <div class="grid grid-cols-2 gap-4">
                    <div>
                      <label class="block text-sm mb-1">Port</label>
                      <select class="w-full bg-gray-800 p-2 rounded" v-model="serialSettings.port">
                        <option value="" disabled>Select a port</option>
                        <option v-for="p in serialSettings.availablePorts" :key="p" :value="p">{{ p }}</option>
                        <option value="COM1">COM1</option>
                        <option value="COM2">COM2</option>
                        <option value="/dev/tty1">/dev/tty1</option>
                      </select>
                    </div>
                    
                    <div>
                      <label class="block text-sm mb-1">Baud Rate</label>
                      <select class="w-full bg-gray-800 p-2 rounded" v-model.number="serialSettings.baudRate">
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
                    <select class="w-full bg-gray-800 p-2 rounded" v-model.number="serialSettings.stopBits">
                      <option :value="1">1</option>
                      <option :value="2">2</option>
                    </select>
                  </div>
                  
                  <div class="mt-6">
                    <h4 class="text-md font-medium mb-2">Send Data</h4>
                    <div class="space-y-2">
                      <input type="text" class="w-full bg-gray-800 p-2 rounded" v-model="serialSettings.sendData" placeholder="Data to send" />
                      <div class="flex gap-2">
                        <input type="text" class="w-1/3 bg-gray-800 p-2 rounded" v-model="serialSettings.sendEndFlag" placeholder="End character" />
                        <button class="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-md transition-colors duration-300 flex-grow">
                          Send
                        </button>
                      </div>
                    </div>
                  </div>
                </div>
                
                <!-- TCP Settings -->
                <div v-show="selectedDataSource === 'tcp'" class="p-4 space-y-4 absolute inset-0 transition-opacity duration-300 overflow-y-auto"
                  :class="{'opacity-100 z-10': selectedDataSource === 'tcp', 'opacity-0 z-0': selectedDataSource !== 'tcp'}">
                  <h3 class="text-lg font-semibold">TCP Connection Settings</h3>
                  
                  <div class="grid grid-cols-2 gap-4">
                    <div>
                      <label class="block text-sm mb-1">Host</label>
                      <input type="text" class="w-full bg-gray-800 p-2 rounded" v-model="tcpSettings.host" placeholder="localhost" />
                    </div>
                    
                    <div>
                      <label class="block text-sm mb-1">Port</label>
                      <input type="number" class="w-full bg-gray-800 p-2 rounded" v-model.number="tcpSettings.port" placeholder="1234" />
                    </div>
                  </div>
                  
                  <div>
                    <label class="block text-sm mb-1">Protocol</label>
                    <select class="w-full bg-gray-800 p-2 rounded" v-model="tcpSettings.protocol">
                      <option value="tcp">TCP</option>
                      <option value="udp">UDP</option>
                    </select>
                  </div>
                </div>
                
                <!-- Fake Data Settings -->
                <div v-show="selectedDataSource === 'fake'" class="p-4 space-y-4 absolute inset-0 transition-opacity duration-300 overflow-y-auto"
                  :class="{'opacity-100 z-10': selectedDataSource === 'fake', 'opacity-0 z-0': selectedDataSource !== 'fake'}">
                  <h3 class="text-lg font-semibold">Fake Data Generator Settings</h3>
                  
                  <div class="grid grid-cols-2 gap-4">
                    <div>
                      <label class="block text-sm mb-1">Min Value</label>
                      <input type="number" class="w-full bg-gray-800 p-2 rounded" v-model.number="fakeDataSettings.minValue" placeholder="-100" />
                    </div>
                    
                    <div>
                      <label class="block text-sm mb-1">Max Value</label>
                      <input type="number" class="w-full bg-gray-800 p-2 rounded" v-model.number="fakeDataSettings.maxValue" placeholder="100" />
                    </div>
                  </div>
                  
                  <div>
                    <label class="block text-sm mb-1">Frequency (Hz)</label>
                    <input type="number" class="w-full bg-gray-800 p-2 rounded" v-model.number="fakeDataSettings.frequency" 
                      placeholder="500" min="0.1" max="1000" step="0.1" />
                    <div class="mt-1">
                      <input type="range" class="w-full" min="0.1" max="1000" step="0.1" v-model.number="fakeDataSettings.frequency" />
                    </div>
                  </div>
                  
                  <div class="grid grid-cols-2 gap-4">
                    <div>
                      <label class="block text-sm mb-1">Number of Channels</label>
                      <input type="number" class="w-full bg-gray-800 p-2 rounded" v-model.number="fakeDataSettings.channelCount" 
                        placeholder="8" min="1" max="8" />
                    </div>
                    
                    <div>
                      <label class="block text-sm mb-1">Waveform</label>
                      <div class="relative">
                        <select 
                          v-model="fakeDataSettings.waveform"
                          class="w-full bg-gray-800 p-2 rounded appearance-none cursor-pointer pr-8 custom-select"
                          style="background-color: rgb(31, 41, 55);"
                        >
                          <option value="sine">Sine</option>
                          <option value="square">Square</option>
                          <option value="triangle">Triangle</option>
                          <option value="sawtooth">Sawtooth</option>
                          <option value="random">Random</option>
                        </select>
                        <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-white">
                          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
                            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <polyline points="6 9 12 15 18 9"></polyline>
                          </svg>
                        </div>
                      </div>
                    </div>
                  </div>
                  
                  <div class="mt-2">
                    <div class="p-3 bg-gray-600 rounded">
                      <div class="flex items-center gap-2">
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" 
                          stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                          <circle cx="12" cy="12" r="10"></circle>
                          <line x1="12" y1="8" x2="12" y2="12"></line>
                          <line x1="12" y1="16" x2="12.01" y2="16"></line>
                        </svg>
                        <span class="text-sm">Fake data is purely for testing and development purposes.</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
          
          <!-- Filters View -->
          <div v-if="activeView === 'filters'" class="bg-gray-800 bg-opacity-60 rounded-lg p-6">
            <div class="flex justify-between items-start mb-6">
              <div>
                <h2 class="text-3xl font-bold text-blue-400">Filter Configuration</h2>
              </div>
              <div class="flex gap-3">
                <button
                  class="bg-blue-600 hover:bg-blue-700 px-6 py-3 rounded-md font-semibold flex items-center gap-2 transition-all duration-300 transform hover:scale-105 shadow-lg">
                  <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
                    stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"></path>
                    <polyline points="17 21 17 13 7 13 7 21"></polyline>
                    <polyline points="7 3 7 8 15 8"></polyline>
                  </svg>
                  Save Config
                </button>
              </div>
            </div>
            <div class="mb-6">
              <div class="grid grid-cols-2 gap-4">
                <div
                  class="bg-gray-700 hover:bg-gray-600 p-4 rounded-lg transition-all duration-300 transform hover:scale-[1.02] cursor-pointer">
                  <div class="flex justify-between items-start mb-2">
                    <h4 class="font-medium">Low Pass</h4>
                    <span class="text-xs bg-blue-500 px-2 py-1 rounded">Active</span>
                  </div>
                  <p class="text-sm text-gray-400 mb-2">Value: 50 Hz</p>
                  <div class="mt-3">
                    <input type="range" min="1" max="200" value="50" class="w-full"/>
                    <div class="flex justify-between text-xs text-gray-400 mt-1">
                      <span>1 Hz</span>
                      <span>200 Hz</span>
                    </div>
                  </div>
                </div>
                <div
                  class="bg-gray-700 hover:bg-gray-600 p-4 rounded-lg transition-all duration-300 transform hover:scale-[1.02] cursor-pointer">
                  <div class="flex justify-between items-start mb-2">
                    <h4 class="font-medium">Band Pass</h4>
                    <span class="text-xs bg-gray-500 px-2 py-1 rounded">Not Active</span>
                  </div>
                  <p class="text-sm text-gray-400 mb-2">Low: 20 Hz High: 100 Hz</p>
                  <div class="mt-3 space-y-3">
                    <div>
                      <label class="text-xs text-gray-400">Low cutoff</label>
                      <input type="range" min="1" max="50" value="20" class="w-full"/>
                    </div>
                    <div>
                      <label class="text-xs text-gray-400">High cutoff</label>
                      <input type="range" min="51" max="200" value="100" class="w-full"/>
                    </div>
                  </div>
                </div>
              </div>
              <div class="mt-6">
                <button class="bg-green-600 hover:bg-green-700 px-4 py-2 rounded-md font-medium mt-4">
                  + Add New Filter
                </button>
              </div>
            </div>
          </div>
          
          <!-- Recording Folder View -->
          <div v-if="activeView === 'folder'" class="bg-gray-800 bg-opacity-60 rounded-lg p-6">
            <div class="flex justify-between items-start mb-6">
              <div>
                <h2 class="text-3xl font-bold text-blue-400">Recording Setup</h2>
              </div>
              <div class="flex gap-3">
                <button
                  class="bg-blue-600 hover:bg-blue-700 px-6 py-3 rounded-md font-semibold flex items-center gap-2 transition-all duration-300 transform hover:scale-105 shadow-lg">
                  <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
                    stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polygon points="5 3 19 12 5 21 5 3"></polygon>
                  </svg>
                  Start Recording
                </button>
              </div>
            </div>
            <div class="space-y-6">
              <div>
                <h3 class="text-lg font-semibold mb-4">Storage Location</h3>
                <div class="mt-2">
                  <div class="flex items-center justify-between mb-2">
                    <p class="text-sm">Select folder to record data</p>
                    <span class="text-xs text-gray-400">Required</span>
                  </div>
                  <div class="flex">
                    <input type="text" placeholder="No folder selected" readonly=""
                      class="bg-gray-700 px-3 py-2 rounded-l-md flex-grow text-gray-300" />
                    <button
                      class="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-r-md transition-colors duration-300">
                      Browse
                    </button>
                  </div>
                </div>
              </div>
              
              <div>
                <h3 class="text-lg font-semibold mb-4">Recording Format</h3>
                <div class="grid grid-cols-3 gap-4">
                  <div class="bg-gray-700 p-4 rounded-lg border-2 border-blue-500">
                    <div class="flex items-center gap-2">
                      <input type="radio" name="format" checked id="csv" />
                      <label for="csv" class="font-medium">CSV</label>
                    </div>
                    <p class="text-xs text-gray-400 mt-1">Standard format compatible with most analysis tools</p>
                  </div>
                  <div class="bg-gray-700 p-4 rounded-lg">
                    <div class="flex items-center gap-2">
                      <input type="radio" name="format" id="binary" />
                      <label for="binary" class="font-medium">Binary</label>
                    </div>
                    <p class="text-xs text-gray-400 mt-1">Compact storage for large datasets</p>
                  </div>
                  <div class="bg-gray-700 p-4 rounded-lg">
                    <div class="flex items-center gap-2">
                      <input type="radio" name="format" id="json" />
                      <label for="json" class="font-medium">JSON</label>
                    </div>
                    <p class="text-xs text-gray-400 mt-1">Structured format with metadata</p>
                  </div>
                </div>
              </div>
              
              <div>
                <h3 class="text-lg font-semibold mb-4">Recording Options</h3>
                <div class="space-y-4">
                  <div class="flex items-center">
                    <input type="checkbox" id="autostart" class="mr-2" />
                    <label for="autostart">Auto-start recording when signal is detected</label>
                  </div>
                  <div>
                    <label class="block mb-2 text-sm">Maximum recording duration (minutes)</label>
                    <input type="number" value="30" min="1" class="bg-gray-700 px-3 py-2 rounded-md w-32" />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
// Add style to document body directly
document.body.style.margin = '0';
document.body.style.padding = '0';
// document.body.style.overflow = 'hidden';
document.documentElement.style.margin = '0';
document.documentElement.style.padding = '0';
// Import the Window API from Tauri
import { getCurrentWindow } from '@tauri-apps/api/window';
import { onMounted, ref } from 'vue';
// import AppHeader from './components/AppHeader.vue';
// import Sidebar from './components/sidebar/Sidebar.vue';
import SignalVisualization from './components/visualization/SignalVisualization.vue';
const appWindow = getCurrentWindow();

// State for tracking which content is shown in the right panel
const activeView = ref('visualization'); // Options: 'visualization', 'signal', 'filters', 'folder'

// Signal source state
const selectedDataSource = ref('fake'); // Options: 'serial', 'tcp', 'fake'

// Settings for different signal sources
const serialSettings = ref({
  port: '',
  baudRate: 115200,
  stopBits: 1,
  availablePorts: [],
  sendData: '',
  sendEndFlag: '\n'
});

const tcpSettings = ref({
  host: 'localhost',
  port: 1234,
  protocol: 'tcp'
});

const fakeDataSettings = ref({
  minValue: -100,
  maxValue: 100,
  frequency: 500,
  channelCount: 8,
  waveform: 'sine'
});

// Function to handle data source change
function onDataSourceChanged(source) {
  selectedDataSource.value = source;
  console.log(`Data source changed to: ${source}`);
}

onMounted(() => {
  console.log("Mounted! Checking DOM...");

  // Apply styles to ensure no margins
  const mainContainer = document.getElementById('main');
  if (mainContainer) {
    Object.assign(mainContainer.style, {
      margin: '0',
      padding: '0',
      width: '100vw',
      height: '100vh'
    });
  }

  // Set up the window controls
  document.getElementById('titlebar-minimize')?.addEventListener('click', () => {
    console.log('Minimize clicked');
    appWindow.minimize();
  });

  document.getElementById('titlebar-maximize')?.addEventListener('click', () => {
    console.log('Maximize clicked');
    appWindow.toggleMaximize();
  });

  document.getElementById('titlebar-close')?.addEventListener('click', () => {
    console.log('Close clicked');
    appWindow.close();
  });

  document.getElementById('titlebar')?.addEventListener('mousedown', (e) => {
    // If the target is a button, exit without dragging.
    if (e.target.closest('button')) return;

    if (e.buttons === 1) {
      if (e.detail === 2) {
        console.log('Toggle maximize clicked');
        appWindow.toggleMaximize();
      } else {
        console.log('Dragging');
        appWindow.startDragging();
      }
    }
  });
});
</script>

<style scoped>
.main {
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
  user-select: none;
}

/* Ensure scrollable containers work properly */
:deep(.overflow-y-auto) {
  scrollbar-width: thin;
  scrollbar-color: rgba(156, 163, 175, 0.5) transparent;
}

:deep(.overflow-y-auto::-webkit-scrollbar) {
  width: 6px;
}

:deep(.overflow-y-auto::-webkit-scrollbar-track) {
  background: transparent;
}

:deep(.overflow-y-auto::-webkit-scrollbar-thumb) {
  background-color: rgba(156, 163, 175, 0.5);
  border-radius: 3px;
}

/* Custom select styling */
.custom-select {
  color: white;
}

/* Cross-browser styling for the select element */
.custom-select {
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  text-indent: 0.01px;
  text-overflow: '';
}

/* Custom select hover effects */
.custom-select:hover {
  background-color: rgb(45, 55, 72) !important;
}
</style>

<!-- Global styles for option elements -->
<style>
option {
  background-color: rgb(55, 65, 81) !important; /* bg-gray-700 */
  color: white !important;
  padding: 8px 16px !important;
}

option:hover, option:focus {
  background-color: rgb(75, 85, 99) !important; /* bg-gray-600 */
}

option:checked {
  background-color: rgba(37, 99, 235, 0.2) !important; /* bg-blue-600 bg-opacity-20 */
  font-weight: 500 !important;
}
</style>
