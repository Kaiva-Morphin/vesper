<template>
  <NuxtLink :to="link" class="rounded-default unselectabe sidebar_item outline_res_content p-[6px] flex flex-row content-center gap-[6px] overflow-hidden"> 
    <Icon :name="icon" class="m-[1px] text-xl" style="color: var(--color-text)"/>
    <span class="text-text">{{ text }}</span>
  </NuxtLink>
</template>

<script lang="ts" setup>
import { defineProps } from 'vue';
const props = defineProps({
  icon: { type: String, required: true },
  text: { type: String, required: true },
  link: { type: String, required: true }
});
</script>

<style>

.sidebar_item {
    background-color: var(--color-card-background);
    transition: background-color 0.2s;
}

.sidebar_item:hover {
    background-color: var(--color-card-hover);
    transition: background-color 0.2s;
}

.sidebar_item:hover > .outline_res_content {
    background-color: var(--color-card-hover);
    transition: background-color 0.2s;
    color: #FF00FFFF;
}

.sidebar_rounding {
  border: 1px solid var(--color-outline);
  border-radius: var(--border-radius) 0 0 var(--border-radius);
}
</style>