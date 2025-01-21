<template>
  <div id="sidebar_allocator" class="sticky h-screen z-10 top-0 left-0 min-w-[48px] flex-auto">
    <div id="sidebar" class="flex flex-col sidebar z-10 absolute no-scrollbar top-0 left-0 h-screen overflow-scroll">
        <SidebarItem text="Home" link="/" icon="ci:house-01"></SidebarItem>
        <div class="sidebar_separator"></div>
        <SidebarItem text="Dashboard" link="/dashboard" icon="ci:triangle"></SidebarItem>
        <SidebarItem text="Service_a" link="/service_a" icon="ci:square"></SidebarItem>
        <SidebarItem text="Service_b" link="/service_b" icon="ci:triangle"></SidebarItem>
        <SidebarItem text="Service_c" link="/service_c" icon="ci:circle"></SidebarItem>
        <div class="sidebar_separator"></div>
        <SidebarItem text="File sender" link="/file_sender" icon="ci:file-upload"></SidebarItem>
        <div class="h-full"></div>
        <div class="sidebar_separator"></div>
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
    background-color: var(--color-primary-muted);
}


.sidebar {
    color: var(--color-text-muted);
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
    outline-width: var(--default-outline-size);
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