import {Chart, registerables} from 'chart.js';
import {Line} from 'react-chartjs-2';

Chart.register(...registerables);

function CombinedContribution({data = {}}: { data: any }) {
    const options = {
        responsive: true,
        plugins: {
            title: {
                display: true,
                text: `${data.owner_handle} (${data.total})`

            }
        },
        elements: {
            point: {
                radius: 0
            }
        },
        scales: {
            y: {
                ticks: {
                    precision: 0
                }
            },
            x: {
                ticks: {
                    callback: function (val: any, index: any): any {
                        // @ts-ignore
                        return index % data.tickMod === 0 ? this.getLabelForValue(val) : '';
                    }
                }
            }
        }

    };

    return (<>
        <Line options={options} data={data}/>
    </>)
}

export default CombinedContribution;
