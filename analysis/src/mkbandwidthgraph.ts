import { join } from 'path';
import { readFileSync, writeFileSync } from 'fs';
import { ConsumerFile } from './consumer-file';
import { ProducerFile } from './producer-file';
import { ChartConfiguration } from 'chart.js';
import { CanvasRenderService } from 'chartjs-node-canvas';

const logRoot = join(__dirname, '..', '..', 'logs');

function getStats(protocol: string): [ConsumerFile[], ProducerFile[]] {
  const fileBase = join(logRoot, 'iot', protocol, 'normal');
  const consumers = [];
  const producers = [];
  for (let i = 1; i <= 5; i++) {
    const consumer = new ConsumerFile(readFileSync(join(fileBase, `${i}-consumer.txt`), { encoding: 'utf8' }));
    const producer = new ProducerFile(readFileSync(join(fileBase, `${i}-producer.txt`), { encoding: 'utf8' }));
    consumers.push(consumer);
    producers.push(producer);
  }

  return [consumers, producers];
}

function getCumulativeBandwidth(cons: ConsumerFile[], prod: ProducerFile[]): number[] {
  const avgConsumerTX = cons[0].entries.map((_, i) => {
    // get each value relative to its start value.
    let value = 0;
    let divValue = 0;
    for (const consumer of cons) {
      if (consumer.entries[i] !== undefined) {
        value += consumer.entries[i].txBytes - consumer.entries[0].txBytes;
        divValue += 1;
      }
    }
    return value / divValue;
  });
  const avgProducerTX = cons[0].entries.map((_, i) => {
    // get each value relative to its start value.
    let value = 0;
    let divValue = 0;
    for (const producer of prod) {
      if (producer.entries[i] !== undefined) {
        value += producer.entries[i].txBytes - producer.entries[0].txBytes;
        divValue += 1;
      }
    }
    return value / divValue;
  });

  return avgConsumerTX.map((consTX, i) => consTX + avgProducerTX[i]);
}


const tcpBandwidth = getCumulativeBandwidth(...getStats('tcp'));
const lrdpBandwidth = getCumulativeBandwidth(...getStats('lrdp'));

const chartConfig: ChartConfiguration = {
  type: 'line',
  data: {
    labels: tcpBandwidth.map(_ => ''),
    datasets: [
      {
        pointRadius: 0,
        label: 'LRDP bandwidth',
        data: lrdpBandwidth,
        fill: false,
        borderColor: '#1838ed',
        lineTension: 0.1,
      },
      {
        label: 'TCP bandwidth',
        pointRadius: 0,
        data: tcpBandwidth,
        fill: false,
        borderColor: '#ed7809',
        lineTension: 0.1,
      },
    ]
  },
  options: {
    title: {
      display: true,
      position: 'bottom',
      fontColor: 'black',
    },
    legend: {
      labels: {
        fontColor: 'black'
      }
    },
    scales: {
      xAxes: [{
        display: false,
        ticks: {
          display: false,
        }
      }],
      yAxes: [{
        ticks: {
          fontColor: 'black',
          beginAtZero: true,
          callback: (v: number) => `${(v / 1024).toFixed(1)}KB`,
        }
      }]
    }
  }
}

const graphRender = new CanvasRenderService(1000, 400);
graphRender.renderToBuffer(chartConfig, 'image/png').then((buffer) => {
  writeFileSync(join(process.cwd(), 'bandwidth-graph.png'), buffer);
  console.log('done');
});
