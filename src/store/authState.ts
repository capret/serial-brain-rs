import { ref } from 'vue';

// Auth state
export const isAuthenticated = ref(false);
export const currentUser = ref<{ email: string } | null>(null);

// Login function
export function login(userData: { email: string }) {
  currentUser.value = userData;
  isAuthenticated.value = true;
  
  // Store in localStorage for persistence across refreshes
  localStorage.setItem('auth_user', JSON.stringify(userData));
  localStorage.setItem('auth_status', 'true');
}

// Logout function
export function logout() {
  currentUser.value = null;
  isAuthenticated.value = false;
  
  // Clear from localStorage
  localStorage.removeItem('auth_user');
  localStorage.removeItem('auth_status');
}

// Initialize auth state from localStorage (if available)
export function initAuthState() {
  const storedAuthStatus = localStorage.getItem('auth_status');
  const storedUser = localStorage.getItem('auth_user');
  
  if (storedAuthStatus === 'true' && storedUser) {
    try {
      currentUser.value = JSON.parse(storedUser);
      isAuthenticated.value = true;
    } catch (e) {
      console.error('Failed to parse stored user data:', e);
      logout(); // Clear invalid data
    }
  }
}
