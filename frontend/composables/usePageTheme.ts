import { useNuxtApp, useRoute, watchEffect, onMounted } from '#imports';

export async function usePageTheme() {
  const userTheme = useNuxtApp().$userTheme as Ref<Record<string, string>>;
  //const route = useRoute();

  //if (!process.client) return;  // ✅ Prevent execution on the server

  /*async function loadTheme() {
    try {
      const response = await fetch('/variables.css');
      if (!response.ok) throw new Error('Failed to fetch theme file');
      
      const cssText = await response.text();
      
      const themeVars: Record<string, string> = {};
      const varRegex = /--color-[\w-]+:\s*([^;]+)/g;
      
      let match;
      while ((match = varRegex.exec(cssText)) !== null) {
        const [fullMatch, value] = match;
        const varName = fullMatch.split(':')[0].trim();
        themeVars[varName] = value.trim();
      }

      console.log("Loaded theme:", themeVars);
      userTheme.value = themeVars;
    } catch (error) {
      console.error("Failed to load theme variables:", error);
    }
  }

  await loadTheme();*/
}
