<!-- <template>
  <div id="background" class="w-full h-full absolute -z-10"/>
    <NuxtPage /> <a class="bg-primary text-white font-serif">ABASczxjhfasuis</a>
    <div id="main" class="absolute flex flex-nowrap w-full h-full justify-center overflow-scroll no-scrollbar" v-locomotivel>

    <div id="sidebar_allocator" class="min-w-sidebar h-full bg-primary rounded-default flex-shrink-0">

    </div>

    <div id="content" class="max-w-content w-content h-[2000px] bg-secondary rounded-default flex-shrink-[1]">

    </div>

    <div id="unused_allocator" class="max-w-sidebar h-full bg-accent rounded-default flex-1 ">

    </div>
    </div>
    
</template> -->
<template>
  <NuxtLayout>
    <NuxtPage />
  </NuxtLayout>
</template>




<script lang="ts" setup>
//import { useColorTheme } from '~/composables/useColorTheme';
import { useInit } from '~/composables/init';

const CustomTheme = {
  primary: '#805ad5',
  secondary: '#d53f8c',
  accent: '#ecc94b',
  neutral: '#f6e05e',
};

useInit();

//useColorTheme(CustomTheme);


import { onMounted } from 'vue';
onMounted(() => {
  var glob_x = 0, glob_y = 0;
  var target_x = 0, target_y = 0;
  var interpolated_x = 0, interpolated_y = 0;

  document.onmousemove = e => {
    target_x = e.clientX;
    target_y = e.clientY;
  };

  function interpolate() {
    interpolated_x += (target_x - interpolated_x) * 0.1;
    interpolated_y += (target_y - interpolated_y) * 0.1;

    if (Math.abs(interpolated_x - glob_x) > 0.1 || Math.abs(interpolated_y - glob_y) > 0.1) {
      glob_x = interpolated_x;
      glob_y = interpolated_y;

      for(const outline_res of document.documentElement.getElementsByClassName("mouse_res")) {
        const rect = outline_res.getBoundingClientRect(),
              rel_x = glob_x - rect.left,
              rel_y = glob_y - rect.top;

        outline_res.style.setProperty("--mouse-x", `${rel_x}px`);
        outline_res.style.setProperty("--mouse-y", `${rel_y}px`);
      }
    }

    requestAnimationFrame(interpolate);
  }

  interpolate();
});
</script>


<style>

</style>