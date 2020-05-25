import { join } from 'path';
import { cwd } from 'process';
import { Run } from './run';

interface TestRun {
  /**
   * The independent variable.
   */
  independent: string;
  /**
   * The data in the run.
   */
  data: {
    /**
     * The value of the independent variable for this data.
     */
    value: string;
    /**
     * The path to the consumer file for this data.
     */
    producer: string;
    /**
     * The path to the producer file for this data.
     */
    consumer: string;
  }[],
}

// tslint:disable:no-console

const runFile: TestRun = process.argv.slice(2, 3)
  .map((f) => join(cwd(), f))
  .map(require)
  .shift();

const [xs, losses, overheads] = runFile.data
  .map((d, i) => {
    const run = new Run(d.producer, d.consumer);
    return [i, 1 - run.payloadLoss(), run.overhead()];
  })
  .reduce(([xs, ls, os], [x, l, o]) => [[...xs, x], [...ls, l], [...os, o]], [[], [], []] as [number[], number[], number[]]);
