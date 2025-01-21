<script setup>
import { CollapsibleContent, CollapsibleRoot, CollapsibleTrigger} from 'radix-vue'
const open = ref(true)
</script>

<template>
  <CardWide>
      <ContentHeader>Collapsible💀</ContentHeader>
      <ContentMainSeparator/>
      <div>Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.</div>
      <ContentContainer>
        <ContentHeader>Container</ContentHeader>
        <ContentSeparator/>
        <div>Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.</div>
      </ContentContainer>
      <ContentContainer>
        <ContentHeader>Container</ContentHeader>
        <ContentSeparator/>
        <div>Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.</div>
      </ContentContainer>
      <ContentContainer>
        <ContentHeader>Container</ContentHeader>
        <ContentSeparator/>
        <div>Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.</div>
      </ContentContainer>
  </CardWide>

  <CardWide>
    <div>Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.</div>
    <CollapsibleRoot v-model:open="open">
      <CollapsibleTrigger><div>zxc</div></CollapsibleTrigger>
      <CollapsibleContent class="CollapsibleContent">
        <div>zxc</div>
        <div>zxc</div>
        <div>zxc</div>
      </CollapsibleContent>
    </CollapsibleRoot>
  </CardWide>
  <CardWide>
    <div>Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.</div>
    <CollapsibleRoot v-model:open="open">
      <CollapsibleTrigger><div>zxc</div></CollapsibleTrigger>
      <CollapsibleContent class="CollapsibleContent">
        <div>zxc</div>
        <div>zxc</div>
        <div>zxc</div>
      </CollapsibleContent>
    </CollapsibleRoot>
  </CardWide>
  <CardWide>
    <div>Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.</div>
    <CollapsibleRoot v-model:open="open">
      <CollapsibleTrigger><div>zxc</div></CollapsibleTrigger>
      <CollapsibleContent class="CollapsibleContent">
        <div>zxc</div>
        <div>zxc</div>
        <div>zxc</div>
      </CollapsibleContent>
    </CollapsibleRoot>
  </CardWide>
  <CardWide>
    <div>Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.</div>
    <CollapsibleRoot v-model:open="open">
      <CollapsibleTrigger><div>zxc</div></CollapsibleTrigger>
      <CollapsibleContent class="CollapsibleContent">
        <div>zxc</div>
        <div>zxc</div>
        <div>zxc</div>
      </CollapsibleContent>
    </CollapsibleRoot>
  </CardWide>
</template>



<style>



.CollapsibleContent {
  overflow: hidden;
}
.CollapsibleContent[data-state="open"] {
  animation: slideDown 300ms ease-out;
}
.CollapsibleContent[data-state="closed"] {
  animation: slideUp 300ms ease-out;
}

@keyframes slideDown {
  from {
    height: 0;
  }
  to {
    height: var(--radix-collapsible-content-height);
  }
}

@keyframes slideUp {
  from {
    height: var(--radix-collapsible-content-height);
  }
  to {
    height: 0;
  }
}
</style>