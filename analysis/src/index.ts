import { join, dirname } from 'path';
import { cwd } from 'process';
import { Run } from './run';
import { CanvasRenderService } from 'chartjs-node-canvas';
import { ChartConfiguration } from 'chart.js';
import { writeFileSync } from 'fs';

const graphRender = new CanvasRenderService(800, 400);

interface Data {
  producer: string;
  consumer: string;
}

type RunX5 = [Data, Data, Data, Data, Data];
type Test = [RunX5, RunX5, RunX5, RunX5];

// tslint:disable:no-console

const file = process.argv.slice(2, 3)[0];
const fileDir = dirname(file);
// tslint:disable-next-line: no-var-requires
const runFile: Test = require(join(cwd(), file));

// normalise all paths
runFile.forEach((run) => {
  run.forEach((data) => {
    data.producer = join(fileDir, data.producer);
    data.consumer = join(fileDir, data.consumer);
  });
});

interface AveragedRun {
  loss: number;
  overhead: number;
}

// map each RunX5 into an AveragedRun.
const avgs: AveragedRun[] = runFile.map((run) => {
  const runs = run.map((d) => new Run(d.producer, d.consumer));
  const loss = runs.reduce((acc, cur) => acc + cur.payloadLoss(), 0) / runs.length;
  const overhead = runs.reduce((acc, cur) => acc + cur.overhead(), 0) / runs.length;
  return { loss, overhead };
});

const chartConfig: ChartConfiguration = {
  type: 'line',
  data: {
    labels: ['Normal', 'Acceptable', 'Degraded', 'Horrible'],
    datasets: [
      {
        label: 'Overhead',
        data: avgs.map((a) => a.overhead),
        fill: false,
        borderColor: '#fcad00'
      },
      {
        label: 'Loss',
        data: avgs.map((a) => 1 - a.loss),
        fill: false,
        borderColor: '#3d9be3'
      }
    ]
  },
  options: {
    scales: {
      yAxes: [{
        ticks: {
          beginAtZero: true,
        }
      }]
    }
  }
}

console.log(avgs);
console.log('rendering to graph.');
graphRender.renderToBuffer(chartConfig, 'image/png').then((buffer) => {
  writeFileSync(join(fileDir, 'graph.png'), buffer);
  console.log('done');
});
