<template>
  <!-- <NuxtImg src="/vesper-monochrome.png" class="custom-filter" style="image-rendering: pixelated"/> -->
  <div style="position: relative;">
    <NuxtImg src="/vesper-monochrome.png" class="w-full h-full relative undraggable unselectable" style="image-rendering: pixelated"></NuxtImg>
    <div class='w-full h-full bg-red-300 absolute z-10 top-0 custom-filter undraggable unselectable'></div>
  </div>
</template>
<style>
.custom-filter{
  mask: url('/vesper-monochrome.png');
  image-rendering: pixelated;
  background-color: color-mix(in srgb, var(--color-primary) 90%, #000);
  background-size: cover;
  mask-repeat: no-repeat;
  mix-blend-mode: overlay;
  /*filter: sepia(0) saturate(0.76) hue-rotate(8deg);*/
  /*opacity: 70%;*/
  mask-size: 100%;
}
</style>