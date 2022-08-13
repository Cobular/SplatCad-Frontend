import type { WebAuth } from "auth0-js";
import { writable, derived, Readable, readable } from "svelte/store";
import { createClient } from "../auth/auth_service";

export interface User {
  sub?: string;
  authToken?: string;
  expireTime?: number;
}

export let auth0Store: Readable<WebAuth> = readable(createClient());

export const isAuthenticated = writable(false);
export const user = writable<User>({});
export const popupOpen = writable(false);
export const error = writable();

