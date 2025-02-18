<template>
  <div :class="['flex message relative w-full justify-center items-center', jumpy?'jump':'']">
    <span :class="['flex items-center gap-2', jumpy ? 'jump' : '']">
      <slot/>
    </span>
    <div class="absolute -bottom-[1px] message-pin w-2 h-2 -translate-x-1/2 translate-y-1/2 rotate-45"></div>
  </div>
</template>

<script lang="ts" setup>
import { defineProps } from 'vue';
const props = defineProps({
  jumpy: {
    type: Boolean,
    default: false
  }
});
</script>

<style scoped>
.jump {
    animation: jump 0.2s cubic-bezier(0, 0.55, 0.45, 1) infinite alternate;
}

@keyframes jump {
    0% {
        transform: translateY(2px);
    }
    100% {
        transform: translateY(-2px);
    }
}
.message {
  padding: 5px 5px;
  border: none;
  border-radius: var(--border-radius);
  outline-color: var(--color-primary);
  outline-style: solid;
  outline-width: var(--default-border-size); 
  background: inherit;
  color: var(--color-primary);
}
.message-pin {
  padding: 5px 5px;
  border: none;
  background: inherit;
  border-color: var(--color-primary);
  border-style: solid;
  border-width: 0 var(--default-border-size) var(--default-border-size) 0; 
  color: var(--color-primary);
}
</style> 