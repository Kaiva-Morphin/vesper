import { watchEffect } from 'vue';

export function useColorTheme(theme: Record<string, string> | null) {
  watchEffect(() => {
    if (!theme) return;

    const root = document.documentElement;
    Object.entries(theme).forEach(([key, value]) => {
      root.style.setProperty(key, value);
    });
  });
}