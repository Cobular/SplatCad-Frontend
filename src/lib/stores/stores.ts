import { derived, writable } from "svelte/store";
import { user } from "./auth";

export const tasks = writable([]);

export const user_tasks = derived([tasks, user], ([$tasks, $user]) => {
  let logged_in_user_tasks = [];

  if ($user && $user.sub) {
    logged_in_user_tasks = $tasks.filter((task) => task.user === $user.sub);
  }

  return logged_in_user_tasks;
});