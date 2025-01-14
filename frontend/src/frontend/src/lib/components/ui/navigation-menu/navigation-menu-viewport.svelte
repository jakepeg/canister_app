<script lang="ts">
  import { cn } from '$lib/utils';
  import { NavigationMenu as NavigationMenuPrimitive } from 'bits-ui';

  let {
    ref = $bindable(null),
    class: className,
    children,
    ...restProps
  }: NavigationMenuPrimitive.ViewportProps = $props();
</script>

<div class={cn('absolute left-0 top-full flex justify-center')}>
  <NavigationMenuPrimitive.Viewport
    bind:ref
    class={cn(
      'origin-top-center relative mt-1.5 h-[var(--bits-navigation-menu-viewport-height)] w-full overflow-hidden rounded-md border bg-popover text-popover-foreground shadow data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-90 md:w-[var(--bits-navigation-menu-viewport-width)]',
      className,
    )}
    {...restProps}
  >
    {@render children?.()}
  </NavigationMenuPrimitive.Viewport>
</div>
