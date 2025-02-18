<template>
  <label class="container">
    <slot/>
    <input type="checkbox"></input>
    <span class="checkmark custom-button"></span>
  </label>
</template>

<script lang="ts" setup>

</script>

<style scoped>
.container {
  color: var(--color-primary);
  align-content: center;
  display: block;
  position: relative;
  padding-left: 40px;
  min-height: 32px;
  cursor: pointer;
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
  user-select: none;
}

.container input {
  position: absolute;
  opacity: 0;
  cursor: pointer;
  height: 32;
  width: 32;
}

.checkmark {
  position: absolute;
  top: 0;
  left: 0;
  height: 32px;
  width: 32px;
}






.container input ~ .checkmark:after {
  border: solid var(--color-primary);
  border-width: 0 3px 3px 0;
  transition: border 0.15s;
}

.container:hover input ~ .checkmark:after {
  border: solid black;
  border-width: 0 3px 3px 0;
  transition: border 0.15s;
}

.container:active input ~ .checkmark:after {
  border: solid black;
  border-width: 0 3px 3px 0;
  transition: border 0.15s;
}

.checkmark:after {
  content: "";
  position: absolute;
  display: none;
}

.container input:checked ~ .checkmark:after {
  display: block;
}

.container .checkmark:after {
  left: 11px;
  top: 6px;
  width: 10px;
  height: 16px;
  -webkit-transform: rotate(45deg);
  -ms-transform: rotate(45deg);
  transform: rotate(45deg);
}


.custom-button {
  padding: 5px 5px;
  border: none;
  border-radius: var(--border-radius);
  outline-color: var(--color-primary);
  outline-style: solid;
  outline-width: var(--default-border-size); 
  color: var(--color-primary);
  cursor: pointer;
  outline-offset: 0px;
  transform: scale(1);
  background-color: #00000000;
  transition: background-color 0.15s, color 0.15s, outline-offset 0.15s, outline-color 0.15s, transform 0.15s;
}

.container:hover .custom-button {
  outline-offset: 3px;
  background-color: var(--color-primary);
  color: var(--color-text-invert);
  transform: scale(1);
  transition: background-color 0.15s, color 0.15s, outline-offset 0.15s, outline-color 0.15s, transform 0.15s;
}

.container:active .custom-button {
    background: var(--color-primary-muted);
    color: var(--color-text-invert);
    outline-offset: -3px;
    outline-color: var(--color-primary-muted);
    transform: scale(0.97);
    transition: background-color 0.15s, color 0.15s, outline-offset 0.15s, outline-color 0.15s, transform 0.15s;
}

.custom-button:disabled {
  cursor: not-allowed;
}
</style>