import type { ActorType } from "$lib/shared/actor";
import { formatUploadDate, formatUploadDateShort } from "$lib/shared/dates";
import { enumIs } from "$lib/shared/enums";
import { flatten } from "$lib/shared/flatten";
import { unreachable } from "$lib/shared/unreachable";
import { get, writable } from "svelte/store";
import type { public_item_metadata } from "../../../../declarations/backend/backend.did";

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
    }
  | {
      state: "error";
      error: string;
    }
  | {
      state: "loaded";
      files: UploadedFile[];
      reloading: boolean;
    };

function createFilesStore() {
  const { subscribe, set } = writable<FilesState>({
    state: "idle",
  });

  return {
    subscribe,
    set,
    setLoaded: (files: UploadedFile[], reloading: boolean) => {
      set({
        state: "loaded",
        files,
        reloading,
      });
    },
    setLoading: () => {
      set({
        state: "loading",
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

  async init() {
    filesStore.setLoading();
    try {
      console.log("Loading files");
      const files = await this.loadFiles();
      console.log("Loaded files: ", files);

      filesStore.setLoaded(files, false);
    } catch (e: unknown) {
      filesStore.setError("Failed to load files");
    }
  }

  async reload() {
    const store = get(filesStore);
    if (store.state === "loading") {
      return;
    } else if (store.state === "loaded") {
      filesStore.setLoaded(store.files, true);
    } else if (store.state === "error" || store.state === "idle") {
      filesStore.setLoading();
    } else {
      unreachable(store);
    }
    try {
      const files = await this.loadFiles();
      filesStore.setLoaded(files, false);
    } catch (e: unknown) {
      filesStore.setError("Failed to load files");
    }
  }

  private async loadFiles(): Promise<UploadedFile[]> {
    // Get both shared items and pending requests
    const items = flatten(
      await Promise.all([
        this.actor.get_items_shared_with_me(),
        this.actor.get_requests(),
      ]),
    );

    // Get contents of root folder
    const rootItems = await this.actor.list_folder_contents([]);
    if (enumIs(rootItems, "Err")) {
      throw new Error(`Failed to load root items: ${rootItems.Err}`);
    }
    
    // Combine all items
    const allItems = [...items, ...rootItems.Ok];
    console.log("All items: ", allItems);

    const uploadedFiles: UploadedFile[] = [];

    for (const item of allItems) {
      const isFolder = 'Folder' in item.item_type;
      
      // Convert both files and folders to UploadedFile type
      uploadedFiles.push({
        name: item.name,
        access: "Only You", // Default access, can be updated based on sharing info
        uploadedAt: formatUploadDate(item.modified_at),
        uploadedAtShort: formatUploadDateShort(item.modified_at),
        file_id: item.id,
        metadata: item,
        parentId: item.parent_id?.[0], // Optional parent folder ID
        isFolder: isFolder
      });
    }

    return uploadedFiles;
  }
}
