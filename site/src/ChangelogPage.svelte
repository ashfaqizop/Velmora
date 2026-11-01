<script lang="ts">
  import { onMount } from "svelte";
  import Backdrop from "./components/Backdrop.svelte";
  import Nav from "./components/Nav.svelte";
  import Changelog from "./components/Changelog.svelte";
  import Ad from "./components/Ad.svelte";
  import Footer from "./components/Footer.svelte";
  import { latest } from "./lib/latest.svelte";

  onMount(() => latest.load());
</script>

<Backdrop />
<Nav />
<main id="top">
  <Changelog />
  <Ad kind="banner" />
</main>
<Footer />
