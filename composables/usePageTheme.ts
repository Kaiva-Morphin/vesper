import { useRoute, watchEffect } from '#imports';
import { useGlobalTheme } from '~/composables/useGlobalTheme';
import EventBus from '~/utils/eventBus';

export function usePageTheme(defaultTheme: Record<string, string>) {
  const route = useRoute();
  const globalTheme =  useGlobalTheme();//useNuxtApp().$globalTheme;
  //globalTheme.value = useGlobalTheme();
  const userTheme = useNuxtApp().$userTheme;
  onMounted(() => {
    watchEffect(() => {
      const theme = (route.meta.themeOverride as Record<string, string>) || userTheme.value;
      const root = document.documentElement;
      Object.entries(theme).forEach(([key, value]) => {
        root.style.setProperty(key, value);
        if (key == "--color-primary") {
          EventBus.$emit('setPrimaryColor', value);
        }
      });
      
    });
  });
}