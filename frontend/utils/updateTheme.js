import { useStorage } from '@vueuse/core'

export default (key, value) => {
  //const storedTheme = useStorage('userTheme', {});
  //const userTheme = useNuxtApp().$userTheme;
  //storedTheme.value[key] = value;
  //userTheme.value[key] = value;
  console.log(key, value);
  if (key == '--color-primary') {
    //console.log("pre")
    //EventBus.$emit("setPrimaryColor", value);
    //console.log("post")

  }
  return
}
