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
    }
)
