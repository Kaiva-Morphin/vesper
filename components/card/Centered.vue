<template>
  <div class="absolute w-full h-fit top-[50vh] -translate-y-[50%] flex justify-center items-center">
    <div class="outline_res w-fit bg-red-500 h-fit text-text rounded-default mouse_res">
      <div class="outline_res_content h-fit p-3">
        <slot />
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>

</script>

<style>

</style>