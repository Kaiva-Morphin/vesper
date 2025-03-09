<template>
    <div class="cf-turnstile h-[65px] z-10 turnstile p-0 m-0 flex justify-center" data-sitekey="0x4AAAAAAA_vG-s5H9SVWJfg">
      <Loading id="loading" class="absolute -z-10"/>
    </div>
</template>

<style>
.turnstile {
  outline-color: var(--color-primary);
  outline-style: dashed;
  outline-width: var(--default-border-size);
  color: var(--color-primary);
}
</style> 

<script setup>
import { onMounted } from "vue";
onMounted(() => {
    const script = document.createElement("script");
    script.src = "https://challenges.cloudflare.com/turnstile/v0/api.js?onload=onloadTurnstileCallback";
    script.defer = true;
    script.z = 10;
    document.head.appendChild(script);
});
</script>