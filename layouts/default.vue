<template>
  <div id="main" class="absolute flex flex-nowrap w-full h-full justify-center overflow-clip no-scrollbar text-text">
    <div id="sidebar_allocator" class="min-w-sidebar top-0 h-full flex-shrink-0 sticky flex justify-end items-center overflow-scroll no-scrollbar">
        <Sidebar/>
    </div>
    <div id="content" class="m-[var(--main-margin)] max-w-content w-content h-fit rounded-default flex-shrink-[1] -z-10 relative">
      <slot />
    </div>
    <div id="unused_allocator" class="max-w-sidebar flex-1 ">
    </div>
  </div>
</template>
