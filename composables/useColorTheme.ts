import { onMounted } from 'vue';

export function useColorTheme(theme: Record<string, string>) {
  onMounted(() => {
    const root = document.documentElement;
    Object.entries(theme).forEach(([key, value]) => {
      root.style.setProperty(`--custom-color-${key}`, value);
    });
  });
}