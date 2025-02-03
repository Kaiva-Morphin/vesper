import { onMounted } from '#imports';
import { useStorage } from '@vueuse/core'

export function useGlobalTheme() {
  const storedTheme = useStorage('userTheme', {});
  const userTheme = useNuxtApp().$userTheme as { value: Record<string, string> };
  if (Object.keys(storedTheme.value).length === 0) {
    onMounted(async () => {
      try {
        const response = await fetch('/variables.css');
        const cssText = await response.text();
        
        const themeVars: Record<string, string> = {};
        const varRegex = /--color-[\w-]+:\s*([^;]+)/g;
  
        let match;
        while ((match = varRegex.exec(cssText)) !== null) {
          const [fullMatch, value] = match;
          const varName = fullMatch.split(':')[0].trim();
          themeVars[varName] = value.trim();
        }
        userTheme.value = themeVars;
        storedTheme.value = userTheme.value;
      } catch (error) {
        console.error("Failed to load theme variables:", error);
      }
    });
  } else {
    userTheme.value = storedTheme.value;
  }
  return userTheme;
}