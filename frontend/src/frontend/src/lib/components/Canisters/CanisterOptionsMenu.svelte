<script lang="ts">
    import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
    
    // export let open = false;
    
    function handleRename() {
        console.log('Rename clicked');
    }
    
    function handleBackup() {
        console.log('Backup clicked');
    }
    
    function handleControllers() {
        console.log('Controllers clicked');
    }
    
    function handleTopupCycles() {
        console.log('Topup Cycles clicked');
    }
    
    function handleMemory() {
        console.log('Memory clicked');
    }
    
    function handleDelete() {
        console.log('Delete clicked');
    }
</script>

<DropdownMenu.Content 
    class="w-[134px] bg-[#1F1F1F] border border-[#0B8CE9] rounded-[11px] p-1 z-50"
    sideOffset={5}
>
    <DropdownMenu.Item
        class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-[#2F2F2F] text-white font-inder"
        on:click={handleRename}
    >
        Rename
    </DropdownMenu.Item>
    
    <DropdownMenu.Item
        class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-[#2F2F2F] text-white font-inder"
        on:click={handleBackup}
    >
        Backup
    </DropdownMenu.Item>
    
    <DropdownMenu.Item
        class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-[#2F2F2F] text-white font-inder"
        on:click={handleControllers}
    >
        Controllers
    </DropdownMenu.Item>
    
    <DropdownMenu.Item
        class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-[#2F2F2F] text-white font-inder"
        on:click={handleTopupCycles}
    >
        Topup Cycles
    </DropdownMenu.Item>
    
    <DropdownMenu.Item
        class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-[#2F2F2F] text-white font-inder"
        on:click={handleMemory}
    >
        Memory
    </DropdownMenu.Item>
    
    <DropdownMenu.Separator class="h-px bg-[#0B8CE9] my-1" />
    
    <DropdownMenu.Item
        class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-[#2F2F2F] text-red-500 font-inder"
        on:click={handleDelete}
    >
        Delete
    </DropdownMenu.Item>
</DropdownMenu.Content>

