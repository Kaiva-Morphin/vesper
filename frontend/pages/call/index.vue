<template>
  <Column>
    <CardWide>
      <ContentHeader>Make a room</ContentHeader>
      <ContentSeparator/>
      <div>You can host your own room!</div>
      <ContentContainer>
        <ContentHeader>Join a Room</ContentHeader>
        <ContentSeparator/>
        <div>
          <label for="roomName">Room Name: </label>
          <ContentTextInput type="text" id="roomName" v-model="roomName"/>
          <ContentBorderButton>Make Room</ContentBorderButton>
        </div>
        <div>
          <ContentBorderButton @click="joinRoom">Join Room</ContentBorderButton>
        </div>
      </ContentContainer>
    </CardWide>
    <CardWide>
      <ContentHeader>Public Rooms</ContentHeader>
      <ContentSeparator/>
      <div>Here you can join a public room</div>
      <ContentContainer>
        <ContentHeader>Join a Room</ContentHeader>
        <ContentSeparator/>
        <div>
          <label for="roomName">Room Name: </label>
          <ContentTextInput type="text" id="roomName" v-model="roomName"/>
        </div>
        <div>
          <ContentBorderButton @click="joinRoom">Join Room</ContentBorderButton>
        </div>
      </ContentContainer>
    </CardWide>
    <CardWide>
      <Loading class="bg-[var(--color-primary-muted)]"/>
    </CardWide>
  </Column>
</template>

<script lang="ts" setup>
function joinRoom() {
  return;
}
function roomName() {
  return;
}
</script>

<style>

</style>