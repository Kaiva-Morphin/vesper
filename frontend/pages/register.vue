<template>
    <CardCentered class="flex w-full h-full justify-center items-center">
      <VesperFullLogoAdapt class="m-8 w-48"/>
      <!-- <ContentMessage jumpy>Username taken!</ContentMessage>
      <ContentMessage>Username Available!</ContentMessage>
      <ContentMessage>Checking<Icon name="eos-icons:bubble-loading"/></ContentMessage> -->
      <ContentTextInput placeholder="login"></ContentTextInput>
      <!-- <ContentMessage jumpy>Password is incorrect!</ContentMessage> -->
      <ContentPasswordInput placeholder="password"></ContentPasswordInput>
      <ContentTextInput placeholder="email"></ContentTextInput>
      <div class="h-fit w-full flex justify-between flex-row gap-[10px]">
        <ContentTextInput placeholder="code" class="w-16 grow"></ContentTextInput>
        <ContentBorderButton>Send code</ContentBorderButton>
      </div>
      <ContentPlaceholder class="w-full flex justify-center items-center h-20">CAPCHA</ContentPlaceholder>
      <ContentCheckbox class="text-[var(--color-text-muted)]">Just check it</ContentCheckbox>
      <ContentBorderButton>Register</ContentBorderButton>
      <div class="h-fit w-full flex justify-between flex-row">
        <div><NuxtLink to="/forgot_password" class="text-[var(--color-text-muted)] text-center w-full">Forgot password?</NuxtLink></div>
        <div><NuxtLink to="/login" class="text-[var(--color-text-muted)] text-center w-full">Log in</NuxtLink></div>
      </div>
  </CardCentered>
</template>

<script lang="ts" setup>

</script>

<style>

</style>