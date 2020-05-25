import { File } from './file';
import { Entry } from './types';

export interface ProducerEntry extends Entry {
  /**
   * The number of packets sent at the time of this entry.
   */
  readonly packetsSent: number;
  /**
   * The total payload bytes sent at the time of this entry.
   */
  readonly totalSentBytes: number;
}

export class ProducerFile extends File<ProducerEntry> {
  readonly entries: ProducerEntry[];

  constructor(csv: string) {
    super();
    this.entries = csv.split('\n')
      .filter((line) => line.length > 0)
      .map((line) => {
        const [timestamp, packetsSent, totalSentBytes, _, rxBytes, txBytes] = line.split(',')
          .filter((s) => /\d+/.test(s))
          .map((num) => parseInt(num, 10));
        return { timestamp, packetsSent, totalSentBytes, rxBytes, txBytes };
      });
  }

  /**
   * The total number of payload bytes transmitted in the window which this file
   * represents.
   */
  txPayloadBytes(): number {
    const [first, last] = this.ends();
    return last.totalSentBytes - first.totalSentBytes;
  }

  /**
   * The number of packets which were sent in the window which this file
   * represents.
   */
  numPackets(): number {
    const [first, last] = this.ends();
    return last.packetsSent - first.packetsSent;
  }
}
