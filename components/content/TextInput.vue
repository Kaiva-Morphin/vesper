<template>
  <input class="custom_input">
  </input>
</template>

<style>
.custom_input {
  padding: 5px 5px;
  border: none;
  border-radius: var(--border-radius);
  outline-color: var(--color-primary);
  outline-style: solid;
  outline-width: var(--default-border-size); 
  color: var(--color-primary);
  cursor: pointer;
  outline-offset: 0px;
  transition: background-color 0.2s, color 0.2s, outline-offset 0.1s;
  background-color: #00000000
}
.custom_input:focus {
  padding: 5px 5px;
  border: none;
  border-radius: var(--border-radius);
  outline-color: var(--color-primary);
  outline-style: solid;
  outline-width: var(--default-border-size); 
  color: var(--color-primary);
  cursor: pointer;
  outline-offset: 0px;
  transition: background-color 0.2s, color 0.2s, outline-offset 0.1s;
  background-color: #00000000
}

</style>