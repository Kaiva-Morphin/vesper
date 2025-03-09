<template>
  <Icon name="tabler:cube-3d-sphere" class="w-16 h-16 rotate text-[var(--color-primary-muted)]"></Icon>
</template>

<script lang="ts" setup>

</script>

<style>
@keyframes animate_rotate {
  0% {transform: rotate3d(0.1, 1, 0, 180deg) translate(0,-5px);}
  25% {transform: rotate3d(0.1, 1, 0, 0deg) translate(0,5px);}
  50% {transform: rotate3d(-0.1, 1, 0, 180deg) translate(0,-5px);}
  75% {transform: rotate3d(0.1, 1, 0, 0deg) translate(0,5px);}
  100% {transform: rotate3d(0.1, 1, 0, 180deg) translate(0,-5px);}
}
.rotate {
  perspective: 200px;
  /*transform: rotate3d(2, -1, -1, var(a));*/
  animation: animate_rotate 2.5s ease-in-out infinite;/*animation: animate_rotate 5s linear 2s infinite alternate;*/
}

</style>