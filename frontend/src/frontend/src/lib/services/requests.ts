import type { ActorType } from "$lib/shared/actor";
import { formatUploadDate, formatUploadDateShort } from "$lib/shared/dates";
import { enumIs } from "$lib/shared/enums";
import { unreachable } from "$lib/shared/unreachable";
import { get, writable } from "svelte/store";

export type Request = {
  name: string;
  group_name: string;
  formattedDate: string;
  formattedDateShort: string;
  file_alias: string;
  group_alias: string;
  access: string;
};

export type RequestsState =
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
      requests: Request[];
      reloading: boolean;
    };

function createRequestsStore() {
  const { subscribe, set } = writable<RequestsState>({
    state: "idle",
  });

  return {
    subscribe,
    set,
    setLoaded: (requests: Request[], reloading: boolean) => {
      set({
        state: "loaded",
        requests: requests,
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

export const requestsStore = createRequestsStore();

export class RequestsService {
  constructor(private actor: ActorType) {}

  async init() {
    requestsStore.setLoading();
    try {
      console.log("Loading requests");
      const requests = await this.loadRequests();
      console.log("Requests: ", requests);

      requestsStore.setLoaded(requests, false);
    } catch (e: unknown) {
      requestsStore.setError("Failed to load files");
    }
  }

  async reload() {
    const store = get(requestsStore);
    if (store.state === "loading") {
      return;
    } else if (store.state === "loaded") {
      requestsStore.setLoaded(store.requests, true);
    } else if (store.state === "error" || store.state === "idle") {
      requestsStore.setLoading();
    } else {
      unreachable(store);
    }
    try {
      const files = await this.loadRequests();
      requestsStore.setLoaded(files, false);
    } catch (e: unknown) {
      requestsStore.setError("Failed to load files");
    }
  }

  private async loadRequests(): Promise<Request[]> {
    const requests = await this.actor.get_requests();

    const uploadedFiles: Request[] = [];

    for (const file of requests) {
      if ('File' in file.item_type && file.size == null) {
        uploadedFiles.push({
          name: file.name,
          group_name: "",
          access: "Only You",
          formattedDate: formatUploadDate(file.modified_at),
          formattedDateShort: formatUploadDateShort(file.modified_at),
          file_alias: "",
          group_alias: file.parent_id?.[0]?.toString() ?? "",
        });
      }
    }

    return uploadedFiles;
  }
}
