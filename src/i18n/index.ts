import { createI18n } from 'vue-i18n';
import en from './en.json';
import zh from './zh.json';

// Try to get saved language from localStorage or default to 'en'
const savedLanguage = localStorage.getItem('language') || 'en';

// Create the i18n instance
export const i18n = createI18n({
  legacy: false, // Use Composition API
  locale: savedLanguage, // Default language is English
  fallbackLocale: 'en', // Fallback to English if translation is missing
  messages: {
    en,
    zh
  }
});

// Export a function to change the language
export function setLanguage(lang: 'en' | 'zh') {
  // Use the correct way to set locale in Composition API mode
  i18n.global.locale.value = lang;
  // Save the language preference in localStorage
  localStorage.setItem('language', lang);
  // Force reload components to apply the new language
  console.log(`Language changed to ${lang}`);
}

// Export a function to get the current language
export function getLanguage(): 'en' | 'zh' {
  // Access the locale value correctly for Composition API mode
  return i18n.global.locale.value as 'en' | 'zh';
}
