import type { ActorType } from "$lib/shared/actor";
import { formatUploadDate, formatUploadDateShort } from "$lib/shared/dates";
import { enumIs } from "$lib/shared/enums";
import { flatten } from "$lib/shared/flatten";
import { unreachable } from "$lib/shared/unreachable";
import { get, writable } from "svelte/store";
import type {
  public_item_metadata,
  item_id,
} from "../../../../declarations/backend/backend.did";

export type UploadedFile = {
  name: string;
  access: string;
  uploadedAt: string;
  uploadedAtShort: string;
  file_id: bigint;
  metadata: public_item_metadata;
  parentId?: bigint; // For hierarchical structure
  isFolder: boolean; // To distinguish between files and folders
};

export type FilesState =
  | {
      state: "idle";
    }
  | {
      state: "loading";
      currentFolderId?: item_id; // Track what folder is being loaded
    }
  | {
      state: "error";
      error: string;
    }
  | {
      state: "loaded";
      // files: UploadedFile[]; // This should probably store all items fetched so far, keyed by parentId or a flat list
      // For simplicity now, let's assume `files` represents the content of the currently loaded folder.
      // A more robust solution would be a cache of folders.
      files: UploadedFile[];
      currentFolderId?: item_id;
      reloading: boolean;
    };

function createFilesStore() {
  const { subscribe, set } = writable<FilesState>({
    state: "idle",
  });

  return {
    subscribe,
    set,
    setLoaded: (
      files: UploadedFile[],
      reloading: boolean,
      folderId?: item_id,
    ) => {
      set({
        state: "loaded",
        files,
        reloading,
        currentFolderId: folderId,
      });
    },
    setLoading: (folderId?: item_id) => {
      set({
        state: "loading",
        currentFolderId: folderId,
      });
    },
    setError: (error: string) => {
      set({
        state: "error",
        error,
      });
    },
    reset: () => set({ state: "idle" }),
  };
}

export const filesStore = createFilesStore();

export class FilesService {
  constructor(private actor: ActorType) {}

  // `init` should load the root folder's content.
  async init() {
    // filesStore.setLoading(); // Let loadFolderContents handle this
    await this.loadFolderContents(undefined); // Load root
  }

  // `reload` should reload the content of the *currently viewed* folder.
  async reload(folderIdToReload?: item_id) {
    const store = get(filesStore);
    let effectiveFolderId = folderIdToReload;

    if (store.state === "loading") {
      // If already loading this specific folder, do nothing
      if (store.currentFolderId === effectiveFolderId) return;
    } else if (store.state === "loaded") {
      effectiveFolderId =
        folderIdToReload === undefined
          ? store.currentFolderId
          : folderIdToReload;
      // filesStore.setLoaded(store.files, true, effectiveFolderId); // Mark as reloading
    }
    // else if (store.state === "error" || store.state === "idle") {
    // filesStore.setLoading(effectiveFolderId);
    // } else {
    // unreachable(store);
    // }

    await this.loadFolderContents(effectiveFolderId);
  }

  // New method to load contents of a specific folder
  async loadFolderContents(folderId?: item_id) {
    filesStore.setLoading(folderId);
    try {
      console.log(`Loading contents for folderId: ${folderId}`);
      const result = await this.actor.list_folder_contents(
        folderId ? [folderId] : [],
      );

      if (enumIs(result, "Err")) {
        console.error(`Failed to load folder contents: ${result.Err}`);
        filesStore.setError(`Failed to load files: ${result.Err}`);
        return;
      }

      const itemsFromBackend = result.Ok;
      console.log(
        "Items from backend for folder",
        folderId,
        ": ",
        itemsFromBackend,
      );

      const processedFiles: UploadedFile[] = itemsFromBackend.map((item) => {
        const isFolder = "Folder" in item.item_type;
        return {
          name: item.name,
          access: "Owner", // Placeholder: Needs proper logic for shared items
          uploadedAt: formatUploadDate(item.modified_at),
          uploadedAtShort: formatUploadDateShort(item.modified_at),
          file_id: item.id, // This matches what FileList.svelte expects
          metadata: item,
          parentId: item.parent_id?.[0],
          isFolder: isFolder,
        };
      });

      filesStore.setLoaded(processedFiles, false, folderId);
    } catch (e: unknown) {
      console.error("Error in loadFolderContents:", e);
      filesStore.setError(
        `Failed to load files. ${e instanceof Error ? e.message : String(e)}`,
      );
    }
  }

  // This function might be deprecated or used for a different view (e.g., "Shared with me")
  // For now, it's not directly used by the FileList.svelte logic for folder navigation.
  private async _DEPRECATED_loadAllMyAccessibleItems(): Promise<
    UploadedFile[]
  > {
    // This old logic combined owned root, shared, and pending.
    // This is not suitable for folder-by-folder navigation.
    const itemsSharedWithMe = await this.actor.get_items_shared_with_me();
    const pendingRequests = await this.actor.get_requests(); // My pending items

    // Get contents of root folder owned by me
    const rootItemsResult = await this.actor.list_folder_contents([]);
    if (enumIs(rootItemsResult, "Err")) {
      throw new Error(`Failed to load root items: ${rootItemsResult.Err}`);
    }
    const ownedRootItems = rootItemsResult.Ok;

    // Combine all items: This logic needs to be careful about duplicates
    // e.g. an item in root shared with me, would appear in both ownedRootItems (if I am owner) and itemsSharedWithMe.
    // A proper approach would be to fetch based on the current folder view.
    const allItemsSet = new Map<bigint, public_item_metadata>();

    [...ownedRootItems, ...itemsSharedWithMe, ...pendingRequests].forEach(
      (item) => {
        if (!allItemsSet.has(item.id)) {
          allItemsSet.set(item.id, item);
        }
      },
    );

    const allItems = Array.from(allItemsSet.values());
    console.log("All accessible items (old logic): ", allItems);

    const uploadedFiles: UploadedFile[] = [];
    for (const item of allItems) {
      const isFolder = "Folder" in item.item_type;
      uploadedFiles.push({
        name: item.name,
        access: "Only You", // Default access, needs update
        uploadedAt: formatUploadDate(item.modified_at),
        uploadedAtShort: formatUploadDateShort(item.modified_at),
        file_id: item.id,
        metadata: item,
        parentId: item.parent_id?.[0],
        isFolder: isFolder,
      });
    }
    return uploadedFiles;
  }
}
