<script lang="ts">
  import Router from "svelte-spa-router";

  import Home from "./routes/Home.svelte";
  import Callback from "./routes/Callback.svelte";

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
</script>

<body>
  <Router {routes} />
</body>
