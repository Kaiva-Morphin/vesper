<template>
  <div class="relative w-full">
    <input
      :type="showPassword ? 'text' : 'password'"
      v-model="password"
      class="w-full custom_input unselectable"
      :placeholder="placeholder"
    />
    <button
      type="button"
      class="absolute right-1 top-1/2 transform -translate-y-1/2 text-[var(--color-primary)] w-8 h-8 flex items-center justify-center"
      @click="togglePassword"
    >
      <Icon :name="showPassword ? 'mdi-eye-off' : 'mdi-eye'" class="w-5 h-5" />
    </button>
  </div>
</template>

<script setup>
import { ref, defineProps } from 'vue';

const props = defineProps({
  placeholder: {
    type: String,
    default: 'Enter your password'
  }
});
const password = ref('');
const showPassword = ref(false);

const togglePassword = () => {
  showPassword.value = !showPassword.value;
};
</script>


<style>
.custom_input {
  padding: 5px 10px;
  border: none;
  border-radius: var(--border-radius);
  outline-color: var(--color-primary);
  outline-style: solid;
  outline-width: var(--default-border-size); 
  color: var(--color-primary);
  cursor: pointer;
  outline-offset: 0px;
  transition: background-color 0.2s, color 0.2s, outline-offset 0.1s;
  background-color: #00000000
}
.custom_input:focus {
  padding: 5px 10px;
  border: none;
  border-radius: var(--border-radius);
  outline-color: var(--color-primary);
  outline-style: solid;
  outline-width: var(--default-border-size); 
  color: var(--color-primary);
  cursor: pointer;
  outline-offset: 0px;
  transition: background-color 0.2s, color 0.2s, outline-offset 0.1s;
  background-color: #00000000
}
</style>