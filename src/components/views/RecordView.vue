<template>
    <div class="bg-gray-800 bg-opacity-60 rounded-lg p-6">
        <h2 class="text-3xl font-bold text-blue-400 mb-6">
            {{ $t("recording.title") }}
        </h2>

        <!-- Status Cards -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
            <!-- Camera Status Card -->
            <div
                class="bg-gray-700 bg-opacity-60 rounded-lg p-4 border border-gray-600"
            >
                <div class="flex items-center justify-between mb-2">
                    <h3 class="text-lg font-semibold text-blue-300">
                        {{ $t("recording.cameraStatus") }}
                    </h3>
                    <div
                        class="w-3 h-3 rounded-full"
                        :class="
                            streamingActive
                                ? 'bg-green-500 animate-pulse'
                                : 'bg-red-500'
                        "
                    ></div>
                </div>
                <div class="flex items-center gap-2">
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="20"
                        height="20"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        :class="
                            streamingActive ? 'text-green-400' : 'text-gray-400'
                        "
                    >
                        <path
                            d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"
                        ></path>
                        <circle cx="12" cy="13" r="4"></circle>
                    </svg>
                    <span
                        class="text-sm"
                        :class="
                            streamingActive ? 'text-green-400' : 'text-gray-400'
                        "
                    >
                        {{
                            streamingActive
                                ? $t("recording.streaming")
                                : $t("recording.notStreaming")
                        }}
                    </span>
                </div>
            </div>

            <!-- Signal Connection Status Card -->
            <div
                class="bg-gray-700 bg-opacity-60 rounded-lg p-4 border border-gray-600"
            >
                <div class="flex items-center justify-between mb-2">
                    <h3 class="text-lg font-semibold text-blue-300">
                        {{ $t("recording.signalStatus") }}
                    </h3>
                    <div
                        class="w-3 h-3 rounded-full"
                        :class="
                            isConnected
                                ? 'bg-green-500 animate-pulse'
                                : 'bg-red-500'
                        "
                    ></div>
                </div>
                <div class="flex items-center gap-2">
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="20"
                        height="20"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        :class="
                            isConnected ? 'text-green-400' : 'text-gray-400'
                        "
                    >
                        <polyline
                            points="22 12 18 12 15 21 9 3 6 12 2 12"
                        ></polyline>
                    </svg>
                    <span
                        class="text-sm"
                        :class="
                            isConnected ? 'text-green-400' : 'text-gray-400'
                        "
                    >
                        {{
                            isConnected
                                ? $t("recording.connected")
                                : $t("recording.disconnected")
                        }}
                    </span>
                </div>
            </div>
        </div>

        <!-- Recording Controls -->
        <div
            class="bg-gray-700 bg-opacity-60 rounded-lg p-6 border border-gray-600"
        >
            <h3 class="text-xl font-semibold mb-4 text-blue-300">
                {{ $t("recording.controls") }}
            </h3>

            <!-- Recording Status Display -->
            <div
                v-if="isRecording"
                class="mb-4 p-4 bg-red-900 bg-opacity-30 rounded-lg border border-red-500"
            >
                <div class="flex items-center gap-2">
                    <div
                        class="w-3 h-3 bg-red-500 rounded-full animate-pulse"
                    ></div>
                    <span class="font-semibold text-red-300">{{
                        $t("recording.recordingInProgress")
                    }}</span>
                </div>
                <p v-if="recordingFilename" class="text-sm text-gray-300 mt-2">
                    {{ $t("recording.filename") }}: {{ recordingFilename }}
                </p>
            </div>

            <!-- Start/Stop Recording Button -->
            <button
                v-if="!isRecording"
                @click="startRecording()"
                :disabled="!isConnected"
                class="w-full bg-blue-600 hover:bg-blue-700 px-6 py-4 rounded-md font-semibold flex items-center justify-center gap-2 transition-all duration-300 transform hover:scale-105 shadow-lg"
                :class="{ 'opacity-50 cursor-not-allowed': !isConnected }"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="20"
                    height="20"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <circle cx="12" cy="12" r="10"></circle>
                    <circle cx="12" cy="12" r="3" fill="currentColor"></circle>
                </svg>
                {{ $t("recording.startRecording") }}
            </button>
            <button
                v-else
                @click="stopRecording()"
                class="w-full bg-red-600 hover:bg-red-700 px-6 py-4 rounded-md font-semibold flex items-center justify-center gap-2 transition-all duration-300 transform hover:scale-105 shadow-lg"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="20"
                    height="20"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <rect x="6" y="6" width="12" height="12"></rect>
                </svg>
                {{ $t("recording.stopRecording") }}
            </button>

            <!-- Recording Format Selector -->
            <RecordingFormatSelector
                v-model="recordingFormat"
                :disabled="isRecording"
                class="mt-6"
            />

            <!-- Recording Options -->
            <RecordingOptions
                v-model:autostart="autoStartRecording"
                v-model:max-recording-duration="maxRecordingDuration"
                :disabled="isRecording"
                class="mt-4"
            />
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import * as path from "@tauri-apps/api/path";
import { exists, mkdir, BaseDirectory } from "@tauri-apps/plugin-fs";
import {
    recordingDirectory,
    isConnected,
    maxRecordingDuration,
    recordingFormat,
    autoStartRecording,
    fetchConnectionState,
    streamingActive,
    toggleStreamingState,
} from "../../store/appState";
import { RecordingFormatSelector, RecordingOptions } from "../records";

// Initialize i18n
useI18n();

// State variables for recording
const isRecording = ref<boolean>(false);
const recordingFilename = ref<string>("");

// Track if the Android service is running
const isServiceRunning = ref<boolean>(false);

// Event listener unsubscribe functions
let recordingCompletedUnsubscribe: (() => void) | null = null;
let recordingFilenameChangedUnsubscribe: (() => void) | null = null;

// Listen for recording events from the backend
async function setupEventListeners() {
    // Clean up previous listeners if they exist
    if (recordingCompletedUnsubscribe) {
        recordingCompletedUnsubscribe();
        recordingCompletedUnsubscribe = null;
    }
    if (recordingFilenameChangedUnsubscribe) {
        recordingFilenameChangedUnsubscribe();
        recordingFilenameChangedUnsubscribe = null;
    }

    // Listen for when a recording completes due to reaching max duration
    recordingCompletedUnsubscribe = await listen(
        "recording-completed",
        async (event) => {
            console.log("Recording completed event received:", event);
            const payload = event.payload as {
                format: string;
                directory: string;
                maxDurationMinutes: number;
                shouldRestartRecording: boolean;
            };

            // Only restart if shouldRestartRecording is true
            if (payload.shouldRestartRecording) {
                console.log("Auto-starting new recording segment...");

                // Temporarily set recording state to false
                isRecording.value = false;

                // Start a new recording segment with the same parameters
                try {
                    setTimeout(async () => {
                        await startRecording(
                            payload.format,
                            payload.directory,
                            payload.maxDurationMinutes,
                            true,
                        );
                    }, 800);
                } catch (error) {
                    console.error("Failed to auto-restart recording:", error);
                    alert(`Failed to auto-restart recording: ${error}`);
                }
            }
        },
    );

    // Listen for recording filename changes
    recordingFilenameChangedUnsubscribe = await listen(
        "recording-filename-changed",
        async (event) => {
            const newFilename = event.payload as string;
            console.log("Recording filename changed to:", newFilename);
            recordingFilename.value = newFilename;
        },
    );
}

// Initialize component
onMounted(async () => {
    try {
        // Set up the recordings directory in AppData
        await selectDirectory();

        // Fetch the connection state from the backend
        try {
            console.log("Fetching connection status from backend...");
            await fetchConnectionState();
            console.log("Connection status:", isConnected.value);
        } catch (error) {
            console.warn(
                "Failed to get connection status from backend:",
                error,
            );
        }

        // Check if we're already recording when navigating to this page
        try {
            interface RecordingStatusResponse {
                isRecording: boolean;
                filename: string;
            }

            const recordingStatus = (await invoke("get_app_state", {
                category: "recording",
                key: "status",
            })) as RecordingStatusResponse;

            if (recordingStatus) {
                isRecording.value = recordingStatus.isRecording;
                recordingFilename.value = recordingStatus.filename;

                console.log(
                    "Recording status from backend:",
                    isRecording.value ? "Active" : "Inactive",
                );
                if (recordingFilename.value) {
                    console.log(
                        "Current recording filename:",
                        recordingFilename.value,
                    );
                }
            }
        } catch (error) {
            console.error("Failed to get recording status:", error);
        }

        // Set up event listeners
        await setupEventListeners();
    } catch (error) {
        console.error("Error initializing component:", error);
    }
});

// Clean up when component unmounts
onUnmounted(() => {
    // Remove event listeners
    if (recordingCompletedUnsubscribe) {
        recordingCompletedUnsubscribe();
        recordingCompletedUnsubscribe = null;
    }
    if (recordingFilenameChangedUnsubscribe) {
        recordingFilenameChangedUnsubscribe();
        recordingFilenameChangedUnsubscribe = null;
    }
});

// Use Document directory with signal_data folder for recordings
async function selectDirectory(): Promise<void> {
    try {
        // Get the document directory full path
        const documentDir = await path.documentDir();

        // Check if signal_data directory exists
        const folderExists = await exists("signal_data", {
            baseDir: BaseDirectory.Document,
        });

        // Create the directory if it doesn't exist
        if (!folderExists) {
            console.log("Creating signal_data directory in Documents");
            await mkdir("signal_data", {
                baseDir: BaseDirectory.Document,
            });
        }

        // Join the document dir with signal_data folder to get the full path
        const fullPath = await path.join(documentDir, "signal_data");

        // Set the recording directory path to the full absolute path
        recordingDirectory.value = fullPath;
        console.log("Using full path for recordings:", fullPath);
    } catch (error) {
        console.error("Error setting up signal_data directory:", error);
        alert(`Failed to set up recordings directory: ${error}`);
    }
}

async function startRecording(
    format?: string,
    directory?: string,
    duration?: number,
    isSegmentChange: boolean = false,
): Promise<void> {
    // Check if camera is streaming - if not, start streaming
    try {
        if (!streamingActive.value) {
            console.log("Camera not streaming, starting camera stream");

            // Get the default stream URL
            const streamUrlResponse = await invoke("get_app_state", {
                category: "stream",
                key: "default_stream_url",
            });
            const streamUrl = streamUrlResponse as string;
            if (streamUrl) {
                const success = await toggleStreamingState(true, streamUrl);
                if (success) {
                    console.log(
                        "Streaming started successfully with URL:",
                        streamUrl,
                    );
                } else {
                    console.warn(
                        "Failed to start streaming, proceeding with recording anyway",
                    );
                }
            } else {
                console.warn(
                    "No default stream URL available, proceeding without camera",
                );
            }
        } else {
            console.log("Camera already streaming, proceeding with recording");
        }
    } catch (error) {
        console.warn("Error checking streaming status:", error);
        // Continue with recording anyway
    }

    // Use provided parameters or defaults from UI controls
    // Explicitly unwrap refs to get plain values
    const recordFormat =
        format ||
        (typeof recordingFormat.value === "string"
            ? recordingFormat.value
            : "csv");
    const recordDir = directory || recordingDirectory.value;
    const recordDuration = duration || maxRecordingDuration.value;

    console.log("=== Recording Parameters ===");
    console.log("recordingFormat.value:", recordingFormat.value);
    console.log("recordFormat:", recordFormat);
    console.log("recordFormat type:", typeof recordFormat);
    console.log("recordFormat JSON:", JSON.stringify(recordFormat));
    console.log("recordDir:", recordDir);
    console.log("recordDuration:", recordDuration);
    console.log("autoStart:", autoStartRecording.value);

    if (!recordDir) {
        alert("Recording directory not set up");
        return;
    }

    // Validate format
    const validFormats = ["csv", "binary", "json"];
    if (!validFormats.includes(recordFormat)) {
        console.error(
            "Invalid format:",
            recordFormat,
            "Valid formats are:",
            validFormats,
        );
        alert(
            `Invalid recording format: ${recordFormat}. Valid formats are: ${validFormats.join(", ")}`,
        );
        return;
    }

    try {
        console.log("Starting recording with format:", recordFormat);
        console.log("Directory:", recordDir);
        console.log("Duration:", recordDuration);

        // Start both signal recording and video recording
        const actualFilename = (await invoke("start_recording", {
            format: String(recordFormat),
            directory: String(recordDir),
            maxDurationMinutes: Number(recordDuration),
            autoStart: Boolean(autoStartRecording.value),
        })) as string;

        console.log("Received filename from backend:", actualFilename);
        recordingFilename.value = actualFilename;

        // Start video recording with the same base name but mp4 extension
        try {
            const baseFilename = actualFilename.replace(/\.[^/.]+$/, "");

            console.log(
                `Starting video recording for ${isSegmentChange ? "segment change" : "new recording"}`,
            );
            console.log("Video base filename:", baseFilename);
            console.log("Video directory:", recordDir);

            await invoke("start_video_recording", {
                filename: baseFilename,
                directory: recordDir,
            });

            console.log("Video recording started successfully");
        } catch (error) {
            console.warn("Failed to start video recording:", error);
        }

        // Start Android service if not already running
        if (!isServiceRunning.value && !isSegmentChange) {
            try {
                await invoke(
                    "plugin:android-forward-service|start_forward_service",
                );
                console.log("Android foreground service started");
                isServiceRunning.value = true;
            } catch (e) {
                console.warn(
                    "Forward service not available or failed to start:",
                    e,
                );
            }
        }

        isRecording.value = true;
        console.log("Recording started - filename:", recordingFilename.value);
    } catch (error) {
        console.error("Error starting recording:", error);
        alert(`Failed to start recording: ${error}`);
    }
}

async function stopRecording(): Promise<void> {
    try {
        await invoke("stop_recording");

        // Stop Android foreground service
        if (isServiceRunning.value) {
            try {
                await invoke(
                    "plugin:android-forward-service|stop_forward_service",
                );
                console.log("Android foreground service stopped");
                isServiceRunning.value = false;
            } catch (e) {
                console.warn(
                    "Forward service not available or failed to stop:",
                    e,
                );
            }
        }

        // Stop video recording
        try {
            await invoke("stop_video_recording");
            console.log("Video recording stopped");

            // Also update the shared streaming state to reflect that streaming has stopped
            if (streamingActive.value) {
                await toggleStreamingState(false);
                console.log("Shared streaming state updated to inactive");
            }
        } catch (error) {
            console.warn("Error stopping video recording:", error);
        }

        isRecording.value = false;
        recordingFilename.value = "";
    } catch (error) {
        console.error("Error stopping recording:", error);
        alert(`Failed to stop recording: ${error}`);
    }
}
</script>
