<template>
  <div class="card_wide flex flex-col gap-2">
    <slot />
  </div>
</template>

<script lang="ts" setup>

</script>

<style>
.card_wide {
  padding: 0.75rem;
  width: 100%;
  height: fit-content;
  border: var(--default-outline-size) solid var(--color-border-1);
  border-radius: var(--border-radius-1);
  background-color: var(--color-background-1);
  position: relative;
  box-shadow: var(--shadow-offset) var(--shadow-radius) var(--color-shadow);
}
</style>