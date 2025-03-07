use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::DefaultMemoryImpl;
use std::cell::RefCell;

const UPGRADES: MemoryId = MemoryId::new(0);

const FILE_CONTENTS: MemoryId = MemoryId::new(1);

// Add a new memory ID for recipient file contents
const RECIPIENT_FILE_CONTENTS: MemoryId = MemoryId::new(2);

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    // The memory manager is used for simulating multiple memories. Given a `MemoryId` it can
    // return a memory that can be used by stable structures.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

pub fn get_upgrades_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(UPGRADES))
}

pub fn get_file_contents_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(FILE_CONTENTS))
}

// Add a function to get this memory
pub fn get_recipient_file_contents_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(RECIPIENT_FILE_CONTENTS))
}
