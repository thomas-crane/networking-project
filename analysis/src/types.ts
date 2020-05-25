/**
 * An entry in a log file.
 */
export interface Entry extends Snapshot {
  /**
   * The timestamp of this entry.
   */
  timestamp: number;
}

/**
 * A snapshot of network statistics.
 */
export interface Snapshot {
  /**
   * The total number of bytes received on this interface.
   */
  readonly rxBytes: number;
  /**
   * The total number of bytes sent on this interface.
   */
  readonly txBytes: number;
}
