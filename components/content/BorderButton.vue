<template>
  <button class="custom-button">
    <slot></slot>
  </button>
</template>

<script lang="ts" setup>
import { defineProps } from 'vue';

const props = defineProps({
  type: {
    type: String,
    default: 'button'
  },
  disabled: {
    type: Boolean,
    default: false
  },
});

</script>

<style scoped>
.custom-button {
  padding: 5px 5px;
  border: none;
  border-radius: var(--border-radius);
  outline-color: var(--color-primary);
  outline-style: solid;
  outline-width: var(--default-border-size); 
  color: var(--color-primary);
  cursor: pointer;
  outline-offset: 0px;
  transform: scale(1);
  transition: background-color 0.2s, color 0.2s, outline-offset 0.1s, outline-color 0.1s, transform 0.1s;
  background-color: #00000000
}

.custom-button:hover {
  outline-offset: 3px;
  background-color: var(--color-primary);
  color: var(--color-text-invert);
  transform: scale(1);
  transition: background-color 0.2s, color 0.2s, outline-offset 0.1s, outline-color 0.1s, transform 0.1s;
}

.custom-button:active {
    background: var(--color-primary-muted);
    color: var(--color-text-invert);
    outline-offset: -3px;
    outline-color: var(--color-primary-muted);
    transform: scale(0.97);
    transition: background-color 0.2s, color 0.2s, outline-offset 0.1s, outline-color 0.1s, transform 0.1s;
}

.custom-button:disabled {
  cursor: not-allowed;
}

</style>