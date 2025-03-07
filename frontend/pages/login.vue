<template>
  <CardCentered class="flex w-full h-full justify-center items-center">
    <VesperFullLogoAdapt class="m-8 w-[236px]"/>
      <ContentTextInput placeholder="email/login"></ContentTextInput>
      <ContentPasswordInput placeholder="password"></ContentPasswordInput>
      <Capcha/>
      <ContentBorderButton>Log in</ContentBorderButton>
      <div class="h-fit w-full flex justify-between flex-row">
        <div><ContentLink to="/forgot_password">Forgot password?</ContentLink></div>
        <div><ContentLink to="/register">Sign up</ContentLink></div>
      </div>
      <ContentLabelSeparator>OR</ContentLabelSeparator>
      <ContentBorderButton class="flex justify-center gap-1"><Icon name="prime:google" class="text-[22px] align-middle"/><span class="align-middle">Google</span></ContentBorderButton>
      <ContentBorderButton class="flex justify-center gap-1"><Icon name="prime:discord" class="text-[22px] align-middle"/><span class="align-middle">Discord</span></ContentBorderButton>
  </CardCentered>
</template>

<script setup>
await preloadComponents('VesperFullLogo')
await preloadComponents('VesperFullLogoAdapt')
/*definePageMeta({
  themeOverride: {
    '--color-primary': '#e74c3c',
  },
});*/
</script>