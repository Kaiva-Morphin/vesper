<template>
  <div id="sidebar_allocator" className="sticky h-screen z-10 top-0 left-0 min-w-[48px] flex-auto">
    <div id="sidebar" className="flex flex-col sidebar z-10 absolute no-scrollbar top-0 left-0 h-screen overflow-scroll">
        <SidebarItem text="Home" link="/" icon="ci-House_01"></SidebarItem>
        <SidebarItem text="Dashboard" link="/dashboard" icon="ci-Chart_Line"></SidebarItem>
        <SidebarItem text="Service_a" link="/service_a" icon="ci-Triangle"></SidebarItem>
        <SidebarItem text="Service_b" link="/service_b" icon="ci-Square"></SidebarItem>
        <SidebarItem text="Service_c" link="/service_c" icon="ci-Circle"></SidebarItem>
        
        <div className="h-full"></div>
        <div className="separator"></div>
        <SidebarItem text="File sender" link="/filesender" icon="ci-File_Upload"></SidebarItem>
        <SidebarItem text="Users" link="/users" icon="ci-Users"></SidebarItem>
        <SidebarItem text="User" link="/user" icon="ci-User_03"></SidebarItem>
      </div>
  </div>
</template>

<script lang="ts" setup>

</script>

<style>

</style>