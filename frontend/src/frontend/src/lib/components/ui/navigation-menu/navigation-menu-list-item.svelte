<script lang="ts">
  import { NavigationMenuLink } from '$lib/components/navigation-menu';
  import { cn } from '$lib/utils';
  import type { WithElementRef } from 'bits-ui';
  import type { HTMLAttributes } from 'svelte/elements';

  let {
    ref = $bindable(),
    class: className,
    children,
    title,
    href,
    ...restProps
  }: WithElementRef<HTMLAttributes<HTMLAnchorElement>> & {
    href?: string;
  } = $props();
</script>

<li>
  <NavigationMenuLink>
    <a
      bind:this={ref}
      class={cn(
        'block select-none space-y-1 rounded-md p-3 leading-none no-underline outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground',
        className,
      )}
      {...restProps}
    >
      <div class="text-sm font-medium leading-none">{title}</div>

      <p class="line-clamp-2 text-sm leading-snug text-muted-foreground">
        {@render children?.()}
      </p>
    </a>
  </NavigationMenuLink>
</li>
