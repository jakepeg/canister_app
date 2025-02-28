<script lang="ts">
  import { cn } from '$lib/utils';
  import { NavigationMenu as NavigationMenuPrimitive } from 'bits-ui';

  let { ref = $bindable(null), class: className, children, ...restProps }: NavigationMenuPrimitive.ListProps = $props();
</script>

<NavigationMenuPrimitive.List
  bind:ref
  class={cn('group flex flex-1 list-none items-center justify-center space-x-1', className)}
  {...restProps}
>
  {@render children?.()}
</NavigationMenuPrimitive.List>
