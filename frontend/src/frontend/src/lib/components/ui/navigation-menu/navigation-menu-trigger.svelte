<script lang="ts">
  import { navigationMenuTriggerStyle } from '$lib/components/navigation-menu';
  import { cn } from '$lib/utils';
  import { NavigationMenu as NavigationMenuPrimitive } from 'bits-ui';
  import ChevronDown from 'lucide-svelte/icons/chevron-down';

  let {
    ref = $bindable(null),
    class: className,
    children,
    ...restProps
  }: NavigationMenuPrimitive.TriggerProps = $props();
</script>

<NavigationMenuPrimitive.Trigger bind:ref class={cn(navigationMenuTriggerStyle, className)} {...restProps}>
  {@render children?.()}

  <ChevronDown
    class="relative top-[1px] ml-1 h-3 w-3 transition duration-300 group-data-[state=open]:rotate-180"
    aria-hidden="true"
  />
</NavigationMenuPrimitive.Trigger>
