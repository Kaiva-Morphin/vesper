<template>
    <div class="cf-turnstile h-[65px] capcha" data-sitekey="0x4AAAAAAA_vG-s5H9SVWJfg"></div>
</template>

<style>
.capcha {
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
    
    document.head.appendChild(script);
});
</script>