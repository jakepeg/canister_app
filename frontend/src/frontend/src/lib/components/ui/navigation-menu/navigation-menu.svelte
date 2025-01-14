<script lang="ts">
  import { NavigationMenuViewport } from '$lib/components/navigation-menu';
  import { cn } from '$lib/utils';
  import { NavigationMenu as NavigationMenuPrimitive } from 'bits-ui';

  let { ref = $bindable(null), class: className, children, ...restProps }: NavigationMenuPrimitive.RootProps = $props();
</script>

<NavigationMenuPrimitive.Root
  bind:ref
  class={cn('relative z-10 flex max-w-max flex-1 items-center justify-center', className)}
  {...restProps}
>
  {@render children?.()}
  <NavigationMenuViewport />
</NavigationMenuPrimitive.Root>
