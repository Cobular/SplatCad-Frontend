import { fetch, FetchOptions, Response } from "@tauri-apps/api/http";
import { get } from "svelte/store";
import { auth0Store, isAuthenticated, user } from "./stores/auth";

export function fetch_authed(
  url: string,
  options: FetchOptions
): Promise<Response<unknown>> {
  if (!get(isAuthenticated)) {
    return Promise.reject(new Error("Not authenticated"));
  }

  return fetch(url, {
    ...options,
    headers: {
      ...options.headers,
      Authorization: `${get(user).authToken}`,
    },
  });
}
