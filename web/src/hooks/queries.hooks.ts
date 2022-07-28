import {useQuery} from "react-query";
import config from "../helpers/config";
import {Breakdown, ContributionResponse, ContributionResult} from "../types/ContributorTypes";
import owner from "../routes/Owner";
import {colours} from "../helpers/colours";

export const useRepositories = () => useQuery(['repositories'], () =>
    fetch(`${config.apiUrl}/projects`).then(res =>
        res.json()
    )
)

export const useCommits = (projectId: number, page: number = 0, limit: number = 50) => useQuery(['commits', projectId, page, limit], () =>
    fetch(`${config.apiUrl}/projects/${projectId}/commits?limit=${limit}&offset=${page * limit}`).then(res =>
        res.json()
    )
)

export const useOwner = (ownerId: number) => useQuery(['owner', ownerId], () =>
    fetch(`${config.apiUrl}/owners/${ownerId}`).then(res =>
        res.json()
    )
)
export const useOwners = () => useQuery(['owners'], () =>
    fetch(`${config.apiUrl}/owners`).then(res =>
        res.json()
    )
)
export const useFeatures = (start?: string, end?: string) => useQuery(['features', start, end], () => {
        if (!start || !end) {
            throw Error('Waiting for shas');
        }
        return fetch(`${config.apiUrl}/features/${start}/${end}`).then(res =>
            res.json()
        )
    }, {
        refetchOnWindowFocus: false,
        enabled: false
    }
)

export const useFiles = (start?: string, end?: string) => useQuery(['files', start, end], () => {
        if (!start || !end) {
            throw Error('Waiting for shas');
        }
        return fetch(`${config.apiUrl}/files/${start}/${end}`).then(res =>
            res.json()
        )
    }, {
        refetchOnWindowFocus: false,
        enabled: false
    }
)

export const useUpdateProject = (projectId: number, stop_at_sha?: string, skip_github_labels?: boolean) => useQuery(['updateProject', projectId], (props) => {
        const body = {};
        if (stop_at_sha !== undefined && stop_at_sha.trim() !== '') {
            // @ts-ignore
            body['stop_at_sha'] = stop_at_sha;
        }
        if (skip_github_labels !== undefined) {
            // @ts-ignore
            body['skip_github_labels'] = skip_github_labels;
        }
        return fetch(`${config.apiUrl}/projects/${projectId}/fetch`, {
            method: 'POST',
            headers: {
                'content-type': 'application/json'
            },
            body: JSON.stringify(body)
        }).then(res =>
            res.json()
        )
    },
    {
        refetchOnWindowFocus: false,
        enabled: false
    }
)


export const useDeleteProject = (projectId: number) => useQuery(['deleteProject', projectId], () => {
        return fetch(`${config.apiUrl}/projects/${projectId}`, {
            method: 'DELETE',
            headers: {
                'content-type': 'application/json'
            },
        }).then(res =>
            res.json()
        )
    },
    {
        refetchOnWindowFocus: false,
        enabled: false,
    }
)

export const fetchContributorStats = async (projectId?: number): Promise<ContributionResult> => {
    const res = await fetch(`${config.apiUrl}/stats/contributions?${projectId ? `project_id=${projectId}` : ''}`);
    return await res.json();
}

export const useContributorStats = (projectId?: number) => useQuery(['contributorStats', projectId], () => fetchContributorStats(projectId))

export const useCombinedContributorStats = (projectId?: number) => useQuery(['combinedContributorStats', projectId], () => fetchContributorStats(projectId), {
    select: (data): any[] => {
        let start: Date | undefined, end: Date | undefined, breakdown: Breakdown | undefined;
        Object.values(data).forEach((v: ContributionResponse) => {
            breakdown = v.breakdown;
            const currentStart = new Date(v.start);
            const currentEnd = new Date(v.end);
            if (!start || currentStart < start) {
                start = currentStart;
            }
            if (!end || currentEnd > end) {
                end = currentEnd;
            }
        });
        if (!start || !end || !breakdown) {
            return [];
        }
        let labels: string[] = [];
        let tickMod = 5;
        switch (breakdown) {
            case Breakdown.Daily:
                labels = getDatesBetweenInDays(start, end).map((d: Date) => formatDate(d));
                tickMod = 5;
                break;
            case Breakdown.Monthly:
                tickMod = 1;
                break;
            case Breakdown.Yearly:
                tickMod = 1;
                break;
        }
        const dataPerOwner: Record<number, any> = {};
        Object.values(data).forEach((contributionResponse: ContributionResponse, index) => {
            const {project_name, contributions: contributionsList} = contributionResponse;
            Object.values(contributionsList).forEach(contributions => {
                const {owner_id, owner_handle, total_contributions, contribution_counts} = contributions;
                if (!dataPerOwner.hasOwnProperty(owner_id)) {
                    dataPerOwner[owner_id] = {
                        tickMod,
                        owner_id,
                        total: 0,
                        owner_handle,
                        labels,
                        datasets: []
                    }
                }
                dataPerOwner[owner_id].total += total_contributions;
                const countByDate: Record<string, number> = {};
                contribution_counts.forEach((cc) => {
                    countByDate[cc.commit_time] = cc.commit_count;
                });
                labels.forEach(label => {
                    if (!countByDate.hasOwnProperty(label)) {
                        countByDate[label] = 0;
                    }
                });
                const data = labels.map(l => countByDate[l]);
                dataPerOwner[owner_id].datasets.push({
                    fill: true,
                    label: `${project_name} ${labels[0]} - ${labels[labels.length - 1]} (${total_contributions})`,
                    data,
                    borderColor: `rgb(${colours[index].join(',')})`,
                    borderWidth: 1,
                    backgroundColor: `rgba(${colours[index].join(',')},0.2)`,
                    line: {
                        tension: 0.1
                    }
                })


            })
        });
        const result = Object.values(dataPerOwner).sort((a, b) => b.total - a.total);

        return result;
    }
})

const formatDate = (d: Date) => {
    let month = (d.getMonth() + 1).toString();
    let day = d.getDate().toString();
    let year = d.getFullYear();
    if (month.length < 2) {
        month = '0' + month;
    }
    if (day.length < 2) {
        day = '0' + day;
    }
    return [year, month, day].join('-');
}

function addDays(date: Date, days: number) {
    date.setDate(date.getDate() + days);
    return date;
}

function getDatesBetweenInDays(startDate: Date, stopDate: Date) {
    const dateArray = [];
    let currentDate = startDate;
    while (currentDate <= stopDate) {
        dateArray.push(new Date(currentDate));
        currentDate = addDays(currentDate, 1);
    }
    return dateArray;
}
