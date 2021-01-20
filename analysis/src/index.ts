import { ChartConfiguration } from 'chart.js';
import { CanvasRenderService } from 'chartjs-node-canvas';
import { writeFileSync } from 'fs';
import { dirname, join } from 'path';
import { Run } from './run';

// tslint:disable:no-console

interface Data {
  producer: string;
  consumer: string;
}

type Test = Data[][];

interface AveragedRun {
  loss: number;
  overhead: number;
}

const graphRender = new CanvasRenderService(1000, 400);
const logRoot = join(__dirname, '..', '..', 'logs');

export function getStats(file: string): AveragedRun[] {
  const fileDir = dirname(file);
  // tslint:disable-next-line: no-var-requires
  const runFile: Test = require(file);

  // normalise all paths
  runFile.forEach((run) => {
    run.forEach((data) => {
      data.producer = join(fileDir, data.producer);
      data.consumer = join(fileDir, data.consumer);
    });
  });

  // map each run of the test into an AveragedRun.
  const avgs: AveragedRun[] = runFile.map((run) => {
    const runs = run.map((d) => new Run(d.producer, d.consumer));
    const loss = runs.reduce((acc, cur) => acc + cur.payloadLoss(), 0) / runs.length;
    const overhead = runs.reduce((acc, cur) => acc + cur.overhead(), 0) / runs.length;
    return { loss, overhead };
  });
  return avgs;
}

function createGraph(figNum: number, protocol: string, avg64: AveragedRun[]): void {
  const chartConfig: ChartConfiguration = {
    type: 'line',
    data: {
      labels: ['Normal', 'Acceptable', 'Degraded', 'Horrible'],
      datasets: [
        {
          label: 'Overhead',
          data: avg64.map((a) => a.overhead),
          fill: false,
          borderColor: '#f05316'
        },
        {
          label: 'Payload bytes received',
          data: avg64.map((a) => 1 - a.loss),
          fill: false,
          borderColor: '#1838ed'
        }
      ]
    },
    options: {
      title: {
        display: true,
        position: 'bottom',
        fontColor: 'black',
        text: `Figure ${figNum}: ${protocol.toUpperCase()} results.`
      },
      legend: {
        labels: {
          fontColor: 'black'
        }
      },
      scales: {
        xAxes: [{
          ticks: {
            fontColor: 'black',
          }
        }],
        yAxes: [{
          ticks: {
            fontColor: 'black',
            beginAtZero: true,
            callback: (v: number) => (v * 100).toFixed(0) + '%',
          }
        }]
      }
    }
  }

  console.log(`[${protocol}] rendering graph.`);
  graphRender.renderToBuffer(chartConfig, 'image/png').then((buffer) => {
    writeFileSync(join(process.cwd(), `${protocol}-graph.png`), buffer);
    console.log(`[${protocol}] done`);
  });
}

const protocols = ['tcp', 'lrdp'];
for (let i = 0; i < protocols.length; i++) {
  const protocol = protocols[i];
  console.log(`[${protocol}] reading logs`)
  const avg64 = getStats(join(logRoot, 'iot', protocol, 'run.json'));
  createGraph(i + 1, protocol, avg64);
}
