<template>
  <!--<div
    id="main" class="relative flex flex-nowrap w-full h-full justify-center overflow-clip no-scrollbar text-text antialiased no-scrollbar scroll-smooth f-jet bg-dark1 -z-20"
  >
    <Sidebar/>
    <div id="content" class="m-[var(--main-margin)] max-w-content w-content h-fit rounded-default flex-shrink-[1] -z-10 relative">
      <slot/>
    </div>
    <div class="sticky h-screen z-10 top-0 left-0 flex-auto"></div>
  </div>-->
  <div id="main" class="absolute flex flex-nowrap w-full h-full justify-center overflow-y-scroll no-scrollbar text-text antialiased -z-20 bg-dark1 f-jet scroll-smooth">
    <BackgroundShaderNoise/>
    <SidebarDynamic/>
    <div id="content" class="flex p-4 m-0 w-full max-w-[960px] h-full flex-col gap-4">
      <slot />
    </div>
    <div id="unused_allocator" class="sticky h-screen z-10 top-0 left-0 flex-auto"></div>
  </div>
</template>


