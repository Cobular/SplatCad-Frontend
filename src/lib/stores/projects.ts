import { derived, Readable, writable } from "svelte/store";
import type {
  CloudProjectData,
  FileMapping,
  ProjectFileMapping,
  WholeProject,
} from "./projects/types";

export const selectedProjectId = writable<undefined | number>(undefined);

// projectMetadata is synced with and filled from the cloud
export const projectMetadata = writable<CloudProjectData[]>([]);

// localFiles is filled from Rust
export const localFiles = writable<ProjectFileMapping>({});

export const currentProject: Readable<WholeProject | undefined> = derived(
  [projectMetadata, localFiles, selectedProjectId],
  ([$projects, $localFiles, $currentProjectId]) => {
    const metadata: CloudProjectData | undefined = $projects[$currentProjectId];
    const files: FileMapping | undefined = $localFiles[$currentProjectId];
    if (metadata !== undefined && files !== undefined)
      return {
        metadata,
        localFiles: files,
      };
    else return undefined;
  }
);
