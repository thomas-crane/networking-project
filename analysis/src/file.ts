import { Entry } from './types';

/**
 * A file which contains some log entries.
 */
export abstract class File<T extends Entry> {
  /**
   * The entries in this file.
   */
  readonly abstract entries: T[];

  /**
   * The total number of bytes received in the window which is represented by
   * this file.
   */
  totalRXBytes(): number {
    if (this.entries.length === 0) {
      return this.entries[0].rxBytes;
    }
    const [first, last] = this.ends();
    return last.rxBytes - first.rxBytes;
  }

  /**
   * The total number of bytes transmitted in the window which is represented by
   * this file.
   */
  totalTXBytes(): number {
    if (this.entries.length === 0) {
      return this.entries[0].txBytes;
    }
    const [first, last] = this.ends();
    return last.txBytes - first.txBytes;
  }

  /**
   * Get the first and the last entry in the `entries`.
   */
  protected ends(): [T, T] {
    const first = this.entries[0];
    const last = this.entries[this.entries.length - 1];

    return [first, last];
  }
}
