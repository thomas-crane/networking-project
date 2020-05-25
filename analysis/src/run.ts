import { readFileSync } from 'fs';
import { ProducerFile } from './producer-file';
import { ConsumerFile } from './consumer-file';

export class Run {
  readonly producerFile: ProducerFile;
  readonly consumerFile: ConsumerFile;

  constructor(producerFile: string, consumerFile: string) {
    const prodTxt = readFileSync(producerFile, { encoding: 'utf8' });
    const consTxt = readFileSync(consumerFile, { encoding: 'utf8' });

    this.producerFile = new ProducerFile(prodTxt);
    this.consumerFile = new ConsumerFile(consTxt);
  }

  /**
   * Returns the percentage of payload bytes which were lost between the sender
   * and the receiver.
   */
  payloadLoss(): number {
    const txBytes = this.producerFile.txPayloadBytes();
    return this.lostPayloadBytes() / txBytes;
  }

  /**
   * The percentage of the total traffic which was overhead.
   */
  overhead(): number {
    const overheadBytes = this.overheadBytes();
    const totalBytes = this.consumerFile.totalRXBytes() + this.producerFile.totalRXBytes();
    return overheadBytes / totalBytes;
  }

  /**
   * Returns the number of payload bytes which were lost between the sender the
   * the receiver.
   */
  private lostPayloadBytes(): number {
    const rxBytes = this.consumerFile.rxPayloadBytes();
    const txBytes = this.producerFile.txPayloadBytes();
    return txBytes - rxBytes;
  }

  /**
   * Returns the total number of bytes in both directions which are not payload
   * bytes.
   */
  private overheadBytes(): number {
    // for the consumer, we need to subtract the number of payload bytes we
    // received from the total number of bytes we received.
    const rxPayload = this.consumerFile.rxPayloadBytes();
    const rxBytes = this.consumerFile.totalRXBytes();
    const rxOverhead = rxBytes - rxPayload;

    // since the consumer doesn't send any payload back to the producer, the
    // total overhead in this direction is just the total number of bytes
    // received by the producer.
    const txOverhead = this.producerFile.totalRXBytes();

    return rxOverhead + txOverhead;
  }
}
