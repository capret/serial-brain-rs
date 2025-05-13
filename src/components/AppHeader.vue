<template>
  <header class="flex justify-between items-center py-8 px-6 max-[800px]:px-4" id="titlebar">
    <h1 class="text-3xl font-bold tracking-tight">
      Serial<span class="text-blue-400">Brain</span>
    </h1>
    <div class="flex items-center gap-4">
      <!-- Language Switch Button -->
      <div class="relative language-menu-container">
        <button @click="toggleLanguageMenu" class="language-button flex items-center gap-1 text-gray-400 hover:text-white p-1 transition-colors duration-200">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="2" y1="12" x2="22" y2="12"></line>
            <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
          </svg>
          <span class="max-[800px]:hidden">{{ currentLanguage === 'en' ? 'Language' : '语言' }}</span>
        </button>
        <div v-show="showLanguageMenu" class="absolute z-50 mt-1 bg-gray-800 border border-gray-700 rounded-md shadow-lg overflow-hidden">
          <button @click="changeLanguage('en')" class="w-full px-4 py-2 text-left hover:bg-gray-700" :class="{'bg-gray-700': currentLanguage === 'en'}">English</button>
          <button @click="changeLanguage('zh')" class="w-full px-4 py-2 text-left hover:bg-gray-700" :class="{'bg-gray-700': currentLanguage === 'zh'}">中文</button>
        </div>
      </div>
      <!-- User info and logout button -->
      <div v-if="user" class="flex items-center gap-3 mr-2">
        <div class="flex items-center gap-2">
          <div class="w-8 h-8 rounded-full bg-blue-500 flex items-center justify-center text-white font-semibold">
            {{ user.email.charAt(0).toUpperCase() }}
          </div>
          <span class="text-sm text-gray-300 max-[800px]:hidden">{{ user.email }}</span>
        </div>
        <button 
          @click="$emit('logout')" 
          class="text-gray-400 hover:text-white p-1 transition-colors duration-200 flex items-center gap-1"
          title="Logout"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"></path>
            <polyline points="16 17 21 12 16 7"></polyline>
            <line x1="21" y1="12" x2="9" y2="12"></line>
          </svg>
          <span class="max-[800px]:hidden">{{ $t('app.logout') }}</span>
        </button>
      </div>
      
      <!-- Window controls -->
      <div class="flex items-center gap-2 ml-2 max-[800px]:hidden">
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
</template>

<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window';
import { onMounted, ref } from 'vue';
import { getLanguage, setLanguage } from '../i18n';

// Define props and emits
defineProps({
  user: {
    type: Object as () => { email: string } | null,
    default: null
  }
});

defineEmits(['logout']);


// Language state
const showLanguageMenu = ref(false);
const currentLanguage = ref(getLanguage());

// Force update when language changes
const updateLanguageDisplay = () => {
  currentLanguage.value = getLanguage();
  console.log('Current language updated:', currentLanguage.value);
};

// Toggle language menu visibility
function toggleLanguageMenu() {
  showLanguageMenu.value = !showLanguageMenu.value;
}

// Change language function
function changeLanguage(lang: 'en' | 'zh') {
  console.log(`Changing language to ${lang}`);
  // Update language in i18n and localStorage
  setLanguage(lang);
  // Update local state
  currentLanguage.value = lang;
  // Close the menu
  showLanguageMenu.value = false;
  
  // Force DOM update
  setTimeout(() => {
    updateLanguageDisplay();
  }, 100);
}

// Close language menu when clicking outside
function handleClickOutside(event: MouseEvent) {
  // Only process if menu is open
  if (!showLanguageMenu.value) return;
  
  // Get the clicked element
  const target = event.target as HTMLElement;
  
  // Check if click is outside the language menu area
  const languageMenuArea = document.querySelector('.relative');
  if (languageMenuArea && !languageMenuArea.contains(target)) {
    showLanguageMenu.value = false;
  }
}

onMounted(() => {
  // Initialize the current language
  updateLanguageDisplay();
  
  // Add click event listener to document to close language menu when clicking outside
  document.addEventListener('click', handleClickOutside);
  const appWindow = getCurrentWindow();
  console.log("AppHeader mounted")
  document.getElementById('titlebar-minimize')?.addEventListener('click', () => {
    console.log("Minimize")
    appWindow.minimize();
  });
  document.getElementById('titlebar-maximize')?.addEventListener('click', () => {
    console.log("Maximize")
    appWindow.toggleMaximize();
  });
  document.getElementById('titlebar-close')?.addEventListener('click', () => {
    console.log("Close")
    appWindow.close();
  });
  document.getElementById('titlebar')?.addEventListener('mousedown', (e) => {
    // Type assertion for TypeScript
    const target = e.target as HTMLElement;
    if (target.closest('button')) return;
    if (e.buttons === 1) {
      if (e.detail === 2) {
        appWindow.toggleMaximize();
      } else {
        appWindow.startDragging();
      }
    }
  });
  
  // Add a keydown event handler to close dropdown on Escape key
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && showLanguageMenu.value) {
      showLanguageMenu.value = false;
    }
  });
});
</script>
