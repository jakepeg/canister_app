import { NavigationMenu as NavigationMenuPrimitive } from 'bits-ui';

const NavigationMenuItem = NavigationMenuPrimitive.Item;
const NavigationMenuLink = NavigationMenuPrimitive.Link;

export { default as NavigationMenuContent } from './navigation-menu-content.svelte';
export { default as NavigationMenuIndicator } from './navigation-menu-indicator.svelte';
export { default as NavigationMenuListItem } from './navigation-menu-list-item.svelte';
export { default as NavigationMenuList } from './navigation-menu-list.svelte';
export { default as NavigationMenuTrigger } from './navigation-menu-trigger.svelte';
export { default as NavigationMenuViewport } from './navigation-menu-viewport.svelte';
export { default as NavigationMenu } from './navigation-menu.svelte';
export { NavigationMenuItem, NavigationMenuLink };

export const navigationMenuTriggerStyle =
  'group inline-flex h-9 w-max items-center justify-center rounded-md bg-background px-4 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground focus:outline-none disabled:pointer-events-none disabled:opacity-50 data-[active]:bg-accent/50 data-[state=open]:bg-accent/50';
