import { Readable, Subscriber, Unsubscriber, writable, Writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/tauri";

export class LocalFilesStore<T> implements Readable<T> {
  private internalStore: Writable<T>;
  subscribe: (this: void, run: Subscriber<T>, invalidate?: (value?: T) => void) => Unsubscriber;

  private constructor(initialValue: T) {
    this.internalStore = writable(initialValue);
    this.subscribe = this.internalStore.subscribe;
  }

  static createFromInitial<N>(initialValue: N): LocalFilesStore<N> {
    return new LocalFilesStore(initialValue);
  }

  static createFromRust<N>(): LocalFilesStore<N> {
    console.log(invoke("get_all_data"))
    // return new LocalFilesStore();
  }

  static createFromRustAsync<N>(): LocalFilesStore<N> {
    return new LocalFilesStore(initialValue);
  }

  refresh() {

  }

  async refreshAsync() {

  }
}
