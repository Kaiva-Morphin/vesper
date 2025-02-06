<template>
  <div id="sidebar_allocator" :class="['sticky h-screen z-10 top-0 left-0 flex-auto', useNuxtApp().$sidebarLeft.value ? 'min-w-[48px]' : 'min-w-[12rem]']"> <!-- min-w-[48px] {{useNuxtApp().$sidebarLeft ? {'min-w-[48px]'} : {'min-w-[12rem]'}}}-->
    <div id="sidebar" 
    
    :class="['flex flex-col z-10 absolute no-scrollbar top-0 right-0 h-screen overflow-scroll', useNuxtApp().$sidebarLeft.value ? 'sidebar' : 'static_sidebar']"> <!-- sidebar {{useNuxtApp().$sidebarLeft ? 'sidebar' : 'static_sidebar'}}-->
        <VesperFullLogoAdapt class="m-2"/>
        <div class="sidebar_separator"></div>
        <SidebarItem text="Home" link="/" icon="ci:house-01"></SidebarItem>
        <div class="sidebar_separator"></div>
        <SidebarItem text="Calls" link="/call" icon="ci:phone"></SidebarItem>
        <SidebarItem text="Dashboard" link="/dashboard" icon="ci:triangle"></SidebarItem>
        <SidebarItem text="Service_a" link="/service_a" icon="ci:square"></SidebarItem>
        <SidebarItem text="Service_b" link="/service_b" icon="ci:triangle"></SidebarItem>
        <div class="sidebar_separator"></div>
        <SidebarItem text="Services" link="/services" icon="ci:main-component"></SidebarItem>
        <div class="sidebar_separator"></div>
        <SidebarItem text="File sender" link="/file_sender" icon="ci:file-upload"></SidebarItem>
        <div v-if="useNuxtApp().$sidebarFullY.value" class="h-full"></div>
        <div class="sidebar_separator"></div>
        <SidebarItem text="Login" link="/login" icon="ci:user-add"></SidebarItem>
        <SidebarItem text="User" link="/user" icon="ci:user"></SidebarItem>
        <SidebarItem text="Users" link="/users" icon="ci:users"></SidebarItem>
        <SidebarItem text="Settings" link="/settings" icon="ci:settings"></SidebarItem>
      </div>
  </div>
</template>

<script lang="ts" setup>
</script>

<style>

.sidebar_separator {
    margin-top: 0;
    margin-bottom: 0;
    border-radius: 1px;
    height: 1px;
    min-height: 1px;
    background-color: var(--color-primary-muted);
}


.sidebar {
    left: 0px;
    width: 64px;
    height: 100%;
    padding-inline: 12px;
    padding-top: 16px;
    padding-bottom: 16px;
    gap: 8px;
    transition-duration: var( --appear-animation-speed);
    backdrop-filter: blur(0px);
    -webkit-backdrop-filter: blur(0px);
    background: transparent;
    transition-property: width;
    transition: width var( --appear-animation-speed), 
    background var( --appear-animation-speed), 
    backdrop-filter var( --appear-animation-speed),
    -webkit-backdrop-filter var( --appear-animation-speed),
    outline-width var( --appear-animation-speed),
    box-shadow var( --appear-animation-speed);
}

.sidebar:hover { 
    width: 12rem;
    height: 100%;
    outline-width: var(--default-border-size);
    outline-color: var(--color-primary-muted);
    outline-style: solid;
    background: color-mix(in srgb, var(--color-card-background) 40%, transparent);
    backdrop-filter: blur(5px);
    -webkit-backdrop-filter: blur(5px);

    transition: width var( --appear-animation-speed), 
    background var( --appear-animation-speed), 
    backdrop-filter var( --appear-animation-speed),
    -webkit-backdrop-filter var( --appear-animation-speed),
    outline-width var( --appear-animation-speed),
    box-shadow var( --appear-animation-speed);

    box-shadow:
        3px 0px 16px 0px  color-mix(in srgb, var(--color-primary-muted) 100%, transparent);
}

.static_sidebar {
    width: 12rem;
    right: 0px;
    height: 100%;
    padding-inline: 12px;
    padding-top: 16px;
    padding-bottom: 16px;
    gap: 8px;
    background: color-mix(in srgb, var(--color-card-background) 0%, transparent);
    backdrop-filter: blur(0px);
    -webkit-backdrop-filter: blur(0px);
}



/* .sidebar > * > .sidebar_label {
    transition-duration: var( --appear-animation-speed);
    opacity: 0.0;
    overflow: clip;
    transition-property: opacity;
}

.sidebar:hover > * > .sidebar_label {
    transition-duration: var( --appear-animation-speed);
    opacity: 1.0;
    transition-property: opacity;
}

.sidebar_icon {
    font-size: 24px;
    padding-left: 8px;
} */
</style>