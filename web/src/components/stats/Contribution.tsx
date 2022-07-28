import {Chart, registerables} from 'chart.js';
import {Line} from 'react-chartjs-2';

Chart.register(...registerables);

function Contribution({contribution = {}}: { contribution: any }) {
    const counts = contribution.contribution_counts.map((c: any) => c.commit_count);
    const labels = contribution.contribution_counts.map((c: any) => c.commit_time);
    const options = {
        responsive: true,
        plugins: {
            title: {
                display: true,
                text: `${contribution.owner_handle} (${contribution.total_contributions})`

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
                        return index % 5 === 0 ? this.getLabelForValue(val) : '';
                    }
                }
            }
        }

    };
    const data = {
        labels,
        datasets: [
            {
                fill: true,
                label: `Commits ${labels[0]} - ${labels[labels.length - 1]}`,
                data: counts,
                borderColor: '#009879',
                borderWidth: 1,
                backgroundColor: 'rgba(0,152,151, 0.2)',
                line: {
                    tension: 0.1
                }
            }
        ]
    }
    return (<>
        <Line options={options} data={data}/>
    </>)
}

export default Contribution;
