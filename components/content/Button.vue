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
  padding: 10px 20px;
  border: none;
  border-radius: 5px;
  background-color: #007bff;
  color: white;
  cursor: pointer;
  transition: background-color 0.3s;
}

.custom-button:hover {
  background-color: #0056b3;
}

.custom-button:disabled {
  background-color: #cccccc;
  cursor: not-allowed;
}
</style>