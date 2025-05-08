use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::DefaultMemoryImpl;
use std::cell::RefCell;

const UPGRADES: MemoryId = MemoryId::new(0);
const FILE_CONTENTS: MemoryId = MemoryId::new(1);
// Assuming MemoryId(2) might be used for recipient_file_contents or similar
const USER_CANISTERS: MemoryId = MemoryId::new(3); // Add new MemoryId

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

// Add function to get memory for the user canisters map
pub fn get_user_canisters_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(USER_CANISTERS))
}
