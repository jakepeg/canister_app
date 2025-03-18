<script>
  import PlaceholderLogo from "../icons/PlaceholderLogo.svelte";
  import { authService } from "$lib/services/auth";
  import LogoIcon from "../icons/LogoIcon.svelte";
</script>

<div class="pt-10 pb-4 text-center flex flex-col items-center gap-4 mt-6">
  <PlaceholderLogo />
  <h2>Please log in</h2>

  <button
    class="gap-4 btn btn-accent flex items-center"
    on:click={() => authService.login()}
  >
    <LogoIcon />
    Login
  </button>
</div>
