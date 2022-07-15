import {useQuery} from "react-query";
import config from "../helpers/config";

export const useRepositories = () => useQuery(['repositories'], () =>
    fetch(`${config.apiUrl}/projects`).then(res =>
        res.json()
    )
)

export const useCommits = (projectId: number, page: number = 0, limit: number = 50) => useQuery(['commits', projectId], () =>
    fetch(`${config.apiUrl}/projects/${projectId}/commits?offset=${page * limit}`).then(res =>
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
