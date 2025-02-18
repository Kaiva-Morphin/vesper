<template>
  <div class="content_container flex flex-col gap-2">
    <slot />
  </div>
</template>

<script lang="ts" setup>

</script>

<style>
.content_container {
  padding: 0.75rem;
  width: 100%;
  height: fit-content;
  border: var(--default-border-size) solid var(--color-border-2);
  border-radius: var(--border-radius-2);
  background-color: var(--color-background-2);
  box-shadow: var(--shadow-offset) var(--shadow-radius) var(--color-shadow);
}
</style>