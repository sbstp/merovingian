<template>
  <div id="app" class="h-100">
    <div class="d-flex flex-row text-center h-100">
      <div id="navbar" class="bg-dark h-100 pt-3">
        <div class="brand">
          <h2 class="text-light">Merovingian {{$route.path}}</h2>
        </div>
        <hr class="bg-light">
        <div class="item pl-4 py-2 text-left border-bottom bg-info">
          <router-link to="/" class="text-light">Home</router-link>
        </div>
        <div class="item pl-4 py-2 text-left border-bottom">
          <router-link to="/about" class="text-light">Import</router-link>
        </div>
        <div class="item pl-4 py-2 text-left">
          <router-link to="/settings" class="text-light">Settings</router-link>
        </div>
      </div>
      <div class="container">
        <router-view/>
      </div>
    </div>
  </div>
</template>

<style lang="less">
#navbar {
  width: 250px;
}
</style>
