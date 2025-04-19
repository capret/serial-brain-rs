<template>

</template>

<script setup>
import { ref, onMounted, onBeforeUnmount, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { WebglPlot, WebglLine, ColorRGBA } from "webgl-plot";


const newData = await invoke("get_recent_data");


</script>
