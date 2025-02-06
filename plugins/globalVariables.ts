import { useNuxtApp, useRoute, watchEffect, onMounted } from '#imports';

export default defineNuxtPlugin(async (nuxtApp) => {
  const userTheme = ref<Record<string, string>>({});
  /*
  const route = useRoute();
  try {
    const response = await fetch('/variables.css');
    const cssText = await response.text();

    // Parse CSS variables
    const themeVars: Record<string, string> = {};
    const varRegex = /(--[\w-]+):\s*([^;]+)/g;

    let match;
    while ((match = varRegex.exec(cssText)) !== null) {
      themeVars[match[1]] = match[2].trim();
    }

    console.log("🎨 Theme Loaded!", themeVars);
    userTheme.value = themeVars; // ✅ Now it's available globally

  } catch (error) {
    console.error("Failed to load theme variables:", error);
  }*/
  nuxtApp.provide('userTheme', userTheme);
  nuxtApp.provide('sidebarFullY', ref(true));
  nuxtApp.provide('sidebarLeft', ref(false));
  nuxtApp.provide('sidebarMinimize', ref(false));
});
