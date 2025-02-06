

<script setup>
//import { ref, watch } from 'vue';
//import { onMounted } from 'vue';

const b = "#0000FF";
const r = "#FF0000";
const color = ref("#FF90FF");
color.value = b;

const sidebarY = useNuxtApp().$sidebarFullY;
const sidebarLeft = useNuxtApp().$sidebarLeft;
const sidebarMinimized = useNuxtApp().$sidebarMinimized;
const userTheme = useNuxtApp().$userTheme; 

const setCSSVariable = (event, key) => {
  const value = event.target.value;
  document.documentElement.style.setProperty(key, value);
  userTheme.value['--color-primary'] = value;
}

/*onMounted(() => {
// Initialize CSS variables
updateCSSVariable('--bg-mix-amount', bgMixAmount.value + '%');
updateCSSVariable('--color-primary', colorPrimary.value);

watch(bgMixAmount, (newValue) => {
  updateCSSVariable('--bg-mix-amount', newValue + '%');
});
  //const storedTheme = useStorage('userTheme', {});
  //const userTheme = useNuxtApp().$userTheme;
  //storedTheme.value[key] = value;
  //userTheme.value[key] = value;
watch(colorPrimary, (newValue) => {
  updateCSSVariable('--color-primary', newValue);
});});*/
const swp = () => {
  if (color.value == r) {
    color.value = b;
  } else {
    color.value = r;
  }
}
</script>

<style>
.test {
  --color: v-bind(color);
}
.test {
  color: var(--color);
  transition: color 2s ease-in-out;
}
</style>

<template>
  <CardWide>
    <ContentHeader>Settings</ContentHeader>
    <div class="test">zxc</div>
    <button @click="swp()">press me!</button>
    <ContentMainSeparator />
    <div>Here you can customize the site appearance</div>
    <ContentContainer>
      <ContentHeader>Adjust Settings</ContentHeader>
      <ContentSeparator />
      
      <!-- <div>
        <label for="bgMixAmount">Background Mix Amount:</label>
        <input type="range" id="bgMixAmount" v-model="bgMixAmount" min="0" max="100" @input="updateCSSVariable('--bg-mix-amount', bgMixAmount + '%')" />
        <span>{{ bgMixAmount }}%</span>
      </div> -->
      
      <div>
        <label for="colorPrimary">Choose Primary Color:</label>
        <input type="color" @input="setCSSVariable($event, '--color-primary')" v-model="useNuxtApp().$userTheme.value['--color-primary']"/>
        <p>Selected Primary Color: <span :style="{ color: useNuxtApp().$userTheme.value['--color-primary'] }">{{ useNuxtApp().$userTheme.value['--color-primary'] }}</span></p>
      </div>
      
      <div>
        <label for="sidebarToggle">Sidebar full-height: </label>
        <input type="checkbox" id="sidebarToggle" v-model="sidebarY" />
      </div>
      <div>
        <label for="sidebarToggle">Sidebar left: </label>
        <input type="checkbox" id="sidebarToggle" v-model="sidebarLeft" />
      </div>  
      <div>
        <label for="sidebarToggle">Minimize sidebar: </label>
        <input type="checkbox" id="sidebarToggle" v-model="sidebarMinimized" />
      </div> 

      
      
      <div>
        <p>Sidebar is <strong>{{ sidebarY ? 'Full-height' : 'Fixed' }}</strong></p>
        <p>Sidebar is on <strong>{{ sidebarLeft ? 'Left' : 'Right' }}</strong></p>
        <p>Sidebar is <strong>{{ sidebarMinimized ? 'Minimized' : 'Full-sized' }}</strong></p>
      </div>
    </ContentContainer>
  </CardWide>
</template>

