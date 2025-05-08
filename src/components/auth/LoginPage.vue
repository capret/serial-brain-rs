<template>
  <div id="main" class="flex flex-col h-screen w-screen overflow-hidden">
    <div class="bg-gradient-to-br from-gray-900 to-gray-800 px-6 rounded-lg shadow-2xl max-[800px]:px-0 font-sans text-white flex flex-col flex-grow overflow-hidden" style="border-radius: 0.5rem;">
      <!-- Include AppHeader at the top -->
      <AppHeader />
      
      <div class="flex items-center justify-center flex-grow px-4">
        <div class="w-full max-w-md bg-gray-800 rounded-lg shadow-xl p-8 border border-gray-700">
          <!-- Logo/Header -->
          <div class="text-center mb-8">
            <h1 class="text-4xl font-bold text-blue-400 mb-2">Signal Brain</h1>
            <p class="text-gray-400">Signal Processing & Visualization</p>
          </div>

          <!-- Form Tabs -->
          <div class="flex border-b border-gray-700 mb-6">
            <button 
              @click="activeTab = 'login'" 
              :class="[
                'py-2 px-4 font-medium transition-colors duration-200 focus:outline-none',
                activeTab === 'login' ? 'text-blue-400 border-b-2 border-blue-400' : 'text-gray-400 hover:text-gray-300'
              ]"
            >
              Login
            </button>
            <button 
              @click="activeTab = 'register'" 
              :class="[
                'py-2 px-4 font-medium transition-colors duration-200 focus:outline-none',
                activeTab === 'register' ? 'text-blue-400 border-b-2 border-blue-400' : 'text-gray-400 hover:text-gray-300'
              ]"
            >
              Register
            </button>
          </div>

          <!-- Login Form -->
          <form v-if="activeTab === 'login'" @submit.prevent="handleLogin" class="space-y-6">
            <div>
              <label for="email" class="block text-sm font-medium text-gray-300 mb-2">Email Address</label>
              <input 
                id="email" 
                v-model="loginForm.email" 
                type="email" 
                required 
                class="w-full px-4 py-3 rounded-md bg-gray-700 border border-gray-600 text-white focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
                placeholder="your@email.com"
              />
            </div>
            
            <div>
              <div class="flex items-center justify-between mb-2">
                <label for="password" class="block text-sm font-medium text-gray-300">Password</label>
                <button 
                  type="button" 
                  @click="activeTab = 'forgot'" 
                  class="text-sm text-blue-400 hover:text-blue-300 focus:outline-none"
                >
                  Forgot password?
                </button>
              </div>
              <input 
                id="password" 
                v-model="loginForm.password" 
                type="password" 
                required 
                class="w-full px-4 py-3 rounded-md bg-gray-700 border border-gray-600 text-white focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
                placeholder="••••••••"
              />
            </div>

            <div class="flex items-center">
              <input 
                id="remember-me" 
                v-model="loginForm.rememberMe" 
                type="checkbox" 
                class="h-4 w-4 rounded border-gray-600 bg-gray-700 text-blue-500 focus:ring-blue-500 focus:ring-offset-gray-800"
              />
              <label for="remember-me" class="ml-2 block text-sm text-gray-300">Remember me</label>
            </div>

            <div v-if="errorMessage" class="bg-red-500 bg-opacity-20 border border-red-500 text-red-300 px-4 py-3 rounded-md text-sm">
              {{ errorMessage }}
            </div>

            <button 
              type="submit" 
              class="w-full bg-blue-600 hover:bg-blue-700 text-white font-semibold py-3 px-4 rounded-md transition-colors duration-300 transform hover:scale-105"
              :disabled="isLoading"
            >
              <span v-if="isLoading" class="flex items-center justify-center">
                <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                Signing in...
              </span>
              <span v-else>Sign in</span>
            </button>
          </form>

          <!-- Register Form -->
          <form v-if="activeTab === 'register'" @submit.prevent="handleRegister" class="space-y-6">
            <div>
              <label for="reg-email" class="block text-sm font-medium text-gray-300 mb-2">Email Address</label>
              <input 
                id="reg-email" 
                v-model="registerForm.email" 
                type="email" 
                required 
                class="w-full px-4 py-3 rounded-md bg-gray-700 border border-gray-600 text-white focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
                placeholder="your@email.com"
              />
            </div>
            
            <div>
              <label for="reg-password" class="block text-sm font-medium text-gray-300 mb-2">Password</label>
              <input 
                id="reg-password" 
                v-model="registerForm.password" 
                type="password" 
                required 
                class="w-full px-4 py-3 rounded-md bg-gray-700 border border-gray-600 text-white focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
                placeholder="••••••••"
              />
            </div>
            
            <div>
              <label for="reg-confirm-password" class="block text-sm font-medium text-gray-300 mb-2">Confirm Password</label>
              <input 
                id="reg-confirm-password" 
                v-model="registerForm.confirmPassword" 
                type="password" 
                required 
                class="w-full px-4 py-3 rounded-md bg-gray-700 border border-gray-600 text-white focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
                placeholder="••••••••"
              />
            </div>

            <div v-if="errorMessage" class="bg-red-500 bg-opacity-20 border border-red-500 text-red-300 px-4 py-3 rounded-md text-sm">
              {{ errorMessage }}
            </div>

            <button 
              type="submit" 
              class="w-full bg-blue-600 hover:bg-blue-700 text-white font-semibold py-3 px-4 rounded-md transition-colors duration-300 transform hover:scale-105"
              :disabled="isLoading"
            >
              <span v-if="isLoading" class="flex items-center justify-center">
                <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                Creating account...
              </span>
              <span v-else>Create Account</span>
            </button>
          </form>

          <!-- Forgot Password Form -->
          <form v-if="activeTab === 'forgot'" @submit.prevent="handleForgotPassword" class="space-y-6">
            <div>
              <label for="forgot-email" class="block text-sm font-medium text-gray-300 mb-2">Email Address</label>
              <input 
                id="forgot-email" 
                v-model="forgotForm.email" 
                type="email" 
                required 
                class="w-full px-4 py-3 rounded-md bg-gray-700 border border-gray-600 text-white focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
                placeholder="your@email.com"
              />
            </div>

            <div v-if="errorMessage" class="bg-red-500 bg-opacity-20 border border-red-500 text-red-300 px-4 py-3 rounded-md text-sm">
              {{ errorMessage }}
            </div>

            <div v-if="successMessage" class="bg-green-500 bg-opacity-20 border border-green-500 text-green-300 px-4 py-3 rounded-md text-sm">
              {{ successMessage }}
            </div>

            <button 
              type="submit" 
              class="w-full bg-blue-600 hover:bg-blue-700 text-white font-semibold py-3 px-4 rounded-md transition-colors duration-300 transform hover:scale-105"
              :disabled="isLoading"
            >
              <span v-if="isLoading" class="flex items-center justify-center">
                <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                Sending...
              </span>
              <span v-else>Reset Password</span>
            </button>

            <div class="text-center">
              <button 
                type="button" 
                @click="activeTab = 'login'" 
                class="text-sm text-blue-400 hover:text-blue-300 focus:outline-none"
              >
                Back to login
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import AppHeader from '../AppHeader.vue';

// Add style to document body directly for consistent styling
onMounted(() => {
  // Apply styles to ensure no margins
  const mainContainer = document.getElementById('main');
  if (mainContainer) {
    Object.assign(mainContainer.style, {
      margin: '0',
      padding: '0',
      width: '100vw',
      height: '100vh',
      touchAction: 'pan-x pan-y'
    });
  }
});


// Props and emits
const emit = defineEmits(['login-success']);

// Form state
const activeTab = ref('login');
const isLoading = ref(false);
const errorMessage = ref('');
const successMessage = ref('');

// Login form
const loginForm = ref({
  email: '',
  password: '',
  rememberMe: false
});

// Register form
const registerForm = ref({
  email: '',
  password: '',
  confirmPassword: ''
});

// Forgot password form
const forgotForm = ref({
  email: ''
});

// Login handler
async function handleLogin() {
  isLoading.value = true;
  errorMessage.value = '';
  
  try {
    // Simulate API call
    console.log('Login attempt with:', loginForm.value);
    
    // Dummy validation
    if (loginForm.value.email && loginForm.value.password) {
      // Simulate delay
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      // Emit success event to parent
      emit('login-success', { 
        email: loginForm.value.email,
        // You could return a token or user object here
      });
    } else {
      throw new Error('Please enter both email and password');
    }
  } catch (error) {
    console.error('Login error:', error);
    errorMessage.value = error instanceof Error ? error.message : 'Failed to login. Please try again.';
  } finally {
    isLoading.value = false;
  }
}

// Register handler
async function handleRegister() {
  isLoading.value = true;
  errorMessage.value = '';
  
  try {
    console.log('Register attempt with:', registerForm.value);
    
    // Dummy validation
    if (!registerForm.value.email || !registerForm.value.password) {
      throw new Error('Please fill in all fields');
    }
    
    if (registerForm.value.password !== registerForm.value.confirmPassword) {
      throw new Error('Passwords do not match');
    }
    
    // Simulate API call delay
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Switch to login tab after successful registration
    activeTab.value = 'login';
    loginForm.value.email = registerForm.value.email;
    loginForm.value.password = '';
    
    // Reset register form
    registerForm.value = {
      email: '',
      password: '',
      confirmPassword: ''
    };
    
  } catch (error) {
    console.error('Registration error:', error);
    errorMessage.value = error instanceof Error ? error.message : 'Failed to register. Please try again.';
  } finally {
    isLoading.value = false;
  }
}

// Forgot password handler
async function handleForgotPassword() {
  isLoading.value = true;
  errorMessage.value = '';
  successMessage.value = '';
  
  try {
    console.log('Password reset requested for:', forgotForm.value.email);
    
    // Dummy validation
    if (!forgotForm.value.email) {
      throw new Error('Please enter your email address');
    }
    
    // Simulate API call delay
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Show success message
    successMessage.value = 'If an account exists with this email, password reset instructions have been sent.';
    
    // Reset form
    forgotForm.value.email = '';
    
  } catch (error) {
    console.error('Password reset error:', error);
    errorMessage.value = error instanceof Error ? error.message : 'Failed to request password reset. Please try again.';
  } finally {
    isLoading.value = false;
  }
}
</script>

<style scoped>
/* Optional animations */
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-10px); }
  to { opacity: 1; transform: translateY(0); }
}

form {
  animation: fadeIn 0.3s ease-out;
}
</style>
