import { WebAuth } from "auth0-js";
import { writable, derived } from "svelte/store";

export interface User {
  sub?: string;
  authToken?: string;
}

export const isAuthenticated = writable(false);
export const user = writable<User>({});
export const popupOpen = writable(false);
export const error = writable();

