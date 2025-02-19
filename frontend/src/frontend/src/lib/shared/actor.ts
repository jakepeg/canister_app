import type { createActor } from "../../../../declarations/backend";

type BaseActorType = ReturnType<typeof createActor>;

// Additional methods we need
interface CustomActorMethods {
  initiate_upload(params: { name: string; file_type: string }): Promise<bigint>;
  create_file(params: { name: string; file_type: string }): Promise<bigint>;
  // Add any other custom methods here
}

// Combined type using intersection
export type ActorType = BaseActorType & CustomActorMethods;
