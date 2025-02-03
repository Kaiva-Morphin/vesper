<template>
  <CardCentered class="flex w-full h-full justify-center items-center">

      

      <VesperFullLogoAdapt class="m-8 w-48"/>
      <ContentTextInput placeholder=" email/login"></ContentTextInput>
      <ContentTextInput placeholder=" password"></ContentTextInput>
      <ContentBorderButton disabled>Log in</ContentBorderButton>
      <div class="h-fit w-full flex justify-between flex-row">
        <div><NuxtLink to="/forgot_password" class="text-[var(--color-text-muted)] text-center w-full">Forgot password?</NuxtLink></div>
        <div><NuxtLink to="/register" class="text-[var(--color-text-muted)] text-center w-full">Sign up</NuxtLink></div>
      </div>

      <ContentLabelSeparator><span class="mx-2">OR</span></ContentLabelSeparator>
      <ContentBorderButton class="flex justify-center gap-1"><Icon name="prime:google" class="text-[22px] align-middle"/><span class="align-middle">Google</span></ContentBorderButton>
      <ContentBorderButton class="flex justify-center gap-1"><Icon name="prime:discord" class="text-[22px] align-middle"/><span class="align-middle">Discord</span></ContentBorderButton>
  </CardCentered>
</template>

<script setup>
await preloadComponents('VesperFullLogo')
await preloadComponents('VesperFullLogoAdapt')
definePageMeta({
  themeOverride: {
    '--color-primary': '#e74c3c',
  },
});
</script>