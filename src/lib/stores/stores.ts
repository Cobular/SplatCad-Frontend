import { derived, writable } from "svelte/store";
import { user } from "./auth";

interface Project {
  id: number;
  name: string;
  description?: string;
  createdAt: string;
}


export const currentProjectId = writable<undefined | number>(undefined);
export const projects = writable<>([]);


export const currentProject = derived(currentProjectId, (id) => {