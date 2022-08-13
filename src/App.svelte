<script lang="ts">
  import Router from "svelte-spa-router";

  import Home from "./routes/Home.svelte";
  import Callback from "./routes/Callback.svelte";

  import { auth0Store } from "./lib/stores/auth";
  import { parseHashHandler } from "./lib/auth/auth_service";

  import "carbon-components-svelte/css/g90.css";
  import { onMount } from "svelte";

  const routes = {
    // Exact path
    "/": Home,

    // Wildcard parameter
    "/callback*": Callback,

    "/#/callback*": Callback,

    // Catch-all
    // This is optional, but if present it must be the last
    "*": Home,
  };

  const auth0Client = $auth0Store;

  onMount(() => {
    auth0Client.parseHash(parseHashHandler(auth0Client));
  });
</script>

<body>
  <Router {routes} />
</body>
