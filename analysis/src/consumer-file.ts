import { File } from './file';
import { Entry } from './types';

export interface ConsumerEntry extends Entry {
  /**
   * The number of packets received at the time of this entry.
   */
  readonly packetsReceived: number;
  /**
   * The total payload bytes received at the time of this entry.
   */
  readonly totalReceivedBytes: number;
}

export class ConsumerFile extends File<ConsumerEntry> {
  readonly entries: ConsumerEntry[];

  constructor(csv: string) {
    super();
    this.entries = csv.split('\n')
      .filter((line) => line.length > 0)
      .map((line) => {
        const [timestamp, packetsReceived, totalReceivedBytes, _, rxBytes, txBytes] = line.split(',')
          .filter((s) => /\d+/.test(s))
          .map((num) => parseInt(num, 10));
        return { timestamp, packetsReceived, totalReceivedBytes, rxBytes, txBytes };
      });
  }

  /**
   * The total number of payload bytes received in the window which this file
   * represents.
   */
  rxPayloadBytes(): number {
    const [first, last] = this.ends();
    return last.totalReceivedBytes - first.totalReceivedBytes;
  }

  /**
   * The total overhead in bytes. This is just the total number of bytes
   * received minus the total payload bytes received.
   */
  overhead(): number {
    return this.totalRXBytes() - this.rxPayloadBytes();
  }

  /**
   * The total overhead in bytes divided by the number of packets received.
   */
  overheadPerPacket(): number {
    return this.overhead() / this.numPackets();
  }

  /**
   * The number of packets which were received in the window which this file
   * represents.
   */
  numPackets(): number {
    const [first, last] = this.ends();
    return last.packetsReceived - first.packetsReceived;
  }
}
