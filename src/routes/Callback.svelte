<script lang="ts">
  import { onMount } from "svelte";

  import { createClient, parseHashHandler } from "../lib/auth/auth_service";
  import type { WebAuth } from "auth0-js";
  import { push } from "svelte-spa-router";
  import { isAuthenticated, user as userStore } from "../lib/stores/auth";
  import { link } from "svelte-spa-router"

  console.log("Loaded Callback page")

  onMount(() => {
    const auth0: WebAuth = createClient();

    auth0.parseHash(parseHashHandler(auth0));
    push('#/')
  });
</script>


<p>You should be redirected in a moment. If not, click <a href="/" use:link>here</a></p>