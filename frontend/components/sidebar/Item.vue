<template>
  <NuxtLink :to="link" :class="['sidebar_button', { 'selected_button': isSelected }]"> 
    <Icon :name="icon" class="overflow-visible sidebar_icon"/>
    <span class="text-opacity-0 absolute ml-2 mt-[5px] sidebar_label">{{ text }}</span>
  </NuxtLink>
</template>

<script lang="ts" setup>
import { defineProps } from 'vue';
const props = defineProps({
  icon: { type: String, required: true },
  text: { type: String, required: true },
  link: { type: String, required: true }
});
const route = useRoute();
const isSelected = computed(() => route.path === props.link);
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
}

.sidebar_rounding {
  border: 1px solid var(--color-outline);
  border-radius: var(--border-radius) 0 0 var(--border-radius);
} 

.sidebar_label {
  font-size: 14px;
}

.minimized_sidebar > * > .sidebar_label {
    transition-duration: var( --appear-animation-speed);
    opacity: 0.0;
    overflow: clip;
    transition-property: opacity;
}

.minimized_sidebar:hover > * > .sidebar_label {
    transition-duration: var( --appear-animation-speed);
    opacity: 1.0;
    transition-property: opacity;
}

.sidebar_icon {
    font-size: 26px;
    /*padding-left: 8px;*/
}

.sidebar_button {
    -webkit-touch-callout: none;
    -webkit-user-select: none;
    -khtml-user-select: none;
    -moz-user-select: none;
    -ms-user-select: none;
    user-select: none;
    cursor: pointer;
    font-size: 0.75rem;
    line-height: 1rem;
    padding-left: 7px;
    text-wrap: nowrap;
    opacity: 1;
    width: 100%;
    height: 40px;
    min-height: 40px;
    text-align: start;
    /* margin-top: 1rem; */
    border-radius: 0.75rem;
    outline-style: solid;
    background: linear-gradient(70deg, rgba(255, 0, 179, 0), rgba(255, 0, 255, 0));
    padding-top: 7px;
    overflow: hidden;
    outline-width: var(--default-border-size);
    outline-width: 0;
    transform: translate(0px, 0px);
    outline-color: color-mix(in srgb, var(--color-primary) 0%, transparent);
    /*transform: translate(0px, -5px) scale(1.05, 1.05);*/
    /*linear-gradient(70deg, 
        color-mix(in srgb, rgb(var(--color-left1)) 30%, transparent),
        color-mix(in srgb, rgb(var(--color-right1)) 30%, transparent)
    );*/
    transition-duration: var(--button-animation-speed);
    transition-property: transform;
    animation-duration: var(--button-animation-speed);
    color: var(--color-text-muted);
}

.sidebar_button:hover {
    /*transform: translate(0px, -5px) scale(1.05, 1.05);*/
    outline-style: solid;
    outline-color: color-mix(in srgb, var(--color-primary) 0%, transparent);
    /*background: linear-gradient(70deg, rgba(255, 0, 179, 0.2), rgba(255, 0, 255, 0.2));*/
    transition-property: transform;
    background: linear-gradient(70deg, 
      color-mix(in srgb, var(--color-primary-right) 20%, transparent),
      color-mix(in srgb, var(--color-primary) 20%, transparent)
    );
    color: var(--color-text);
}

.sidebar_button:active {
  color: var(--color-text);
  outline-color: color-mix(in srgb, var(--color-primary) 0%, transparent);
  transform: scale(0.98, 0.98);
  transition-property: transform;
  background: linear-gradient(70deg, 
    color-mix(in srgb, var(--color-primary-right) 32%, transparent),
    color-mix(in srgb, var(--color-primary) 32%, transparent)
  );
  transition-duration: var(--button-animation-speed);
  animation-duration: var(--button-animation-speed);
  color: var(--color-text);
}

.selected_button {
  background: linear-gradient(70deg, 
      color-mix(in srgb, var(--color-primary-right) 20%, transparent),
      color-mix(in srgb, var(--color-primary) 20%, transparent)
    );
    outline-color: color-mix(in srgb, var(--color-primary-highlighted) 40%, transparent);
    /*box-shadow:
    0px -2px 12px 0px color-mix(in srgb, rgb(var(--color-left0)) 50%, transparent),
    0px 2px 12px 0px color-mix(in srgb, rgb(var(--color-right0)) 50%, transparent);*/
    transition-duration: var(--button-animation-speed);
    animation-duration: var(--button-animation-speed);
    color: var(--color-text);
}

.selected_button:hover {
    background: linear-gradient(70deg, 
      color-mix(in srgb, var(--color-primary-right) 32%, transparent),
      color-mix(in srgb, var(--color-primary) 32%, transparent)
    );
    /* border-image: linear-gradient(70deg, 
      color-mix(in srgb, var(--color-primary-right) 90%, transparent),
      color-mix(in srgb, var(--color-primary-left) 90%, transparent)
    ); */
    outline-color: color-mix(in srgb, var(--color-primary-highlighted-extra) 40%, transparent);
    transition-duration: var(--button-animation-speed);
    animation-duration: var(--button-animation-speed);
}


</style>