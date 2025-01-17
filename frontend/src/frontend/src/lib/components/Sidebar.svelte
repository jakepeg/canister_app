<script lang="ts">
  import { page } from "$app/stores";
  import { authService, authStore } from "$lib/services/auth";
  import IconFile from "./icons/IconFile.svelte";
  import RequestsIcon from "./icons/RequestsIcon.svelte";
  import { uploadInProgress } from "$lib/services/upload";
  import * as Sidebar from "$lib/components/ui/sidebar";

  function logout() {
    if ($uploadInProgress) {
      if (
        !confirm("Uploading is in progress. Are you sure you want to logout?")
      )
        return;
    }
    authService.logout();
  }

  // Menu items.
  const items = [
    {
      title: "Files",
      url: "/",
      icon: IconFile,
    },
    {
      title: "Requests",
      url: "/requests",
      icon: RequestsIcon,
    },
  ];

  import type { ComponentProps } from "svelte";

  let {
    ref = $bindable(null),
    collapsible = "icon",
    ...restProps
  }: ComponentProps<typeof Sidebar.Root> = $props();
</script>

<Sidebar.Root bind:ref {collapsible} {...restProps}>
  <Sidebar.Content class=" py-20">
    <Sidebar.Group>
      <Sidebar.GroupLabel>Application</Sidebar.GroupLabel>
      <Sidebar.GroupContent>
        <Sidebar.Menu>
          {#each items as item (item.title)}
            <Sidebar.MenuItem>
              <Sidebar.MenuButton>
                {#snippet child({ props })}
                  <a href={item.url} {...props}>
                    <item.icon />
                    <span>{item.title}</span>
                  </a>
                {/snippet}
              </Sidebar.MenuButton>
            </Sidebar.MenuItem>
          {/each}
        </Sidebar.Menu>
      </Sidebar.GroupContent>
    </Sidebar.Group>
  </Sidebar.Content>
</Sidebar.Root>
