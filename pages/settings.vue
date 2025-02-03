<template>
  <CardWide>
    <ContentHeader>Settings</ContentHeader>
    <ContentMainSeparator />
    <div>Here you can customize the site appearance</div>
    <ContentContainer>
      <ContentHeader>Adjust Settings</ContentHeader>
      <ContentSeparator />
      
      <div>
        <label for="bgMixAmount">Background Mix Amount:</label>
        <input type="range" id="bgMixAmount" v-model="bgMixAmount" min="0" max="100" @input="updateCSSVariable('--bg-mix-amount', bgMixAmount + '%')" />
        <span>{{ bgMixAmount }}%</span>
      </div>
      
      <div>
        <label for="colorPrimary">Choose Primary Color:</label>
        <input type="color" id="colorPrimary" v-model="colorPrimary" @input="updateCSSVariable('--color-primary', colorPrimary)" />
      </div>
      
      <div>
        <label for="sidebarToggle">Toggle Sidebar: </label>
        <input type="checkbox" id="sidebarToggle" v-model="isSidebarVisible" />
      </div>
      
      <div>
        <p>Selected Primary Color: <span :style="{ color: colorPrimary }">{{ colorPrimary }}</span></p>
        <p>Sidebar is <strong>{{ isSidebarVisible ? 'Visible' : 'Hidden' }}</strong></p>
      </div>
    </ContentContainer>
  </CardWide>
</template>

<script setup>
import { ref, watch } from 'vue';
import { onMounted } from 'vue';

const bgMixAmount = ref(3);
const colorPrimary = ref('#00C1FF');
const isSidebarVisible = ref(true);

const updateCSSVariable = (variable, value) => {
  document.documentElement.style.setProperty(variable, value);
  updateTheme(variable, value);
};
onMounted(() => {
// Initialize CSS variables
updateCSSVariable('--bg-mix-amount', bgMixAmount.value + '%');
updateCSSVariable('--color-primary', colorPrimary.value);

watch(bgMixAmount, (newValue) => {
  updateCSSVariable('--bg-mix-amount', newValue + '%');
});
/*

  //const storedTheme = useStorage('userTheme', {});
  //const userTheme = useNuxtApp().$userTheme;
  //storedTheme.value[key] = value;
  //userTheme.value[key] = value;


*/
watch(colorPrimary, (newValue) => {
  updateCSSVariable('--color-primary', newValue);
});});
</script>

<style>
/* Add any specific styles for the settings page here */
</style>