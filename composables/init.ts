import { onMounted } from 'vue';

export const useInit = () => {
  onMounted(() => {
    document.body.classList.add('bg-background', 'no-scrollbar', 'text-text');
  });
}
