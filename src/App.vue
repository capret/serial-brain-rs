<template>
  <div id="webcrumbs">
    <div class="app-container">
      <!-- Header section - placed first in DOM order -->
      <AppHeader />
      
      <!-- Main content area with sidebar and visualization -->
      <div class="main-content">
        <Sidebar />
        <SignalVisualization />
      </div>
    </div>
  </div>
</template>

<script setup>
import AppHeader from './components/AppHeader.vue';
import Sidebar from './components/sidebar/Sidebar.vue';
import SignalVisualization from './components/visualization/SignalVisualization.vue';
</script>

<style>
@import './assets/main.css';

#webcrumbs {
  height: 100vh;
  display: flex;
  flex-direction: column;
  margin: 0;
  padding: 0;
}

.app-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: white;
  overflow: hidden;
  margin: 0;
  padding: 0;
}

.main-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}

html, body {
  margin: 0;
  padding: 0;
  overflow: hidden;
}
</style>