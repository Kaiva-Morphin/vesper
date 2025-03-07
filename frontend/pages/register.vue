<template>
    <CardCentered class="flex w-full h-full justify-center items-center relative">
      <VesperFullLogoAdapt class="m-8 w-[236px]"/>
      <div class="relative w-full h-0 bg-inherit ">
        <div class="absolute w-full bottom-0 bg-inherit">
          <!-- <ContentMessage jumpy id="username_status">Username taken!</ContentMessage> -->
          <!-- <ContentMessage>Username Available!</ContentMessage> -->
          <!-- <ContentMessage>Checking<Icon name="eos-icons:bubble-loading"/></ContentMessage> -->
        </div>
      </div>
      <ContentTextInput placeholder="login"/>
      <!-- <ContentMessage jumpy>Password is incorrect!</ContentMessage> -->
      <ContentPasswordInput placeholder="password"/>
      <Capcha/>
      <ContentTextInput placeholder="email"></ContentTextInput>
      <div class="h-fit w-full flex justify-between flex-row gap-[10px]">
        <ContentBorderButton>Send code</ContentBorderButton>
        <ContentTextInput placeholder="code" class="w-16 grow"/>
      </div>
      <ContentCheckbox class="text-[var(--color-text-muted)]">I agree with <ContentLink to="/tos">tos</ContentLink></ContentCheckbox>
      <div class="relative w-full h-0 bg-inherit -mt-3">
        <div class="absolute w-full bottom-0 bg-inherit">

        </div>
      </div>
      <ContentBorderButton>Register</ContentBorderButton>
      <div class="h-fit w-full flex justify-between flex-row">
        <div><ContentLink to="/forgot_password">Forgot password?</ContentLink></div>
        <div><ContentLink to="/login">Log in</ContentLink></div>
      </div>
  </CardCentered>
</template>
