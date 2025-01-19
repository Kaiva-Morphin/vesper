<script lang="ts" setup>
import { Separator } from 'radix-vue'
</script>


<template>
  <Separator class="h-full absolute w-px right-0 top-0 bg-outline vertical_res_separator mouse_res" type="vertical"/>
  <div class="h-fit w-full flex justify-end flex-col gap-[3px] sidebar relative p-[4px]">
    <SidebarItem icon="tabler:home" text="Home" link="/"/>
    <SidebarSeparator/>
    <SidebarItem icon="tabler:door-enter" text="Register" link="/register"/>
    <SidebarSeparator/>
    <SidebarItem icon="tabler:user" text="Profile" link="/profile"/>
    <SidebarItem icon="tabler:message" text="Messages" link="/msg"/>
    <SidebarItem icon="tabler:users" text="Friends" link="/friends"/>
    <SidebarSeparator/>
    <SidebarItem icon="tabler:server-spark" text="Services" link="/services"/>
    <SidebarSeparator/>
    <SidebarItem icon="tabler:play-card-star" text="Admin" link="/about"/>
    <SidebarItem icon="tabler:tag" text="Tags" link="/about"/>
    <SidebarItem icon="uil:info-circle" text="About" link="/about"/>
  </div>
</template>

<style>

.vertical_res_separator::after {
    position: absolute;
    display: block;
    border-radius: inherit;
    content: "";
    inset: calc(var(--hover-outline-size) / 2 * -1);
    width: calc(100% + var(--hover-outline-size) / 2);
    background:  linear-gradient(#ffffff00, #ffffff00) padding-box, radial-gradient(
        600px circle at var(--mouse-x) var(--mouse-y), 
        var(--color-primary),
        transparent var(--hover-outline-radius)
    ) border-box;
}

.horisontal_res_separator::after {
    position: absolute;
    display: block;
    border-radius: inherit;
    content: "";
    /*inset: calc(var(--hover-outline-size) / 2 * -1);*/
    margin-left: -1px;
    width: calc(100% + var(--hover-outline-size) / 2);
    height: calc(100% + var(--hover-outline-size) / 2);
    background:  linear-gradient(#ffffff00, #ffffff00) padding-box, radial-gradient(
        600px circle at var(--mouse-x) var(--mouse-y), 
        var(--color-primary),
        transparent var(--hover-outline-radius)
    ) border-box;
}
</style>