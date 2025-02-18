<template>
  <div class="placeholder">
    <slot/>
  </div>
</template>

<script lang="ts" setup>

</script>

<style>
.placeholder {
  padding: 5px 5px;
  border: none;
  border-radius: var(--border-radius);
  outline-color: var(--color-primary);
  outline-style: dashed;
  outline-width: var(--default-border-size); 
  color: var(--color-primary);
}
</style> 