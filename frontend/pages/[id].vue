<template>
  <CardCentered>
    <ContentHeader>
      <h1>404 - Page Not Found</h1>
    </ContentHeader>
    <ContentMainSeparator/>
    <ContentContainer>
      <h1>/{{ $route.params.id }}</h1>
    </ContentContainer>
    <p>Sorry, the page you are looking for does not exist.</p>
  </CardCentered>
</template>

<script lang="ts" setup>
</script>

<style scoped>
.error-page {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
}

.error-page h1 {
  font-size: 3rem;
  margin-bottom: 1rem;
}

.error-page p {
  font-size: 1.5rem;
  margin-bottom: 2rem;
}

.error-page a {
  font-size: 1.2rem;
  color: var(--color-primary);
  text-decoration: underline;
}
</style>