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

export const useFeatures = (start: string, end: string) => useQuery(['features', start, end], () =>
    fetch(`${config.apiUrl}/features/${start}/${end}`).then(res =>
        res.json()
    )
)



// export const useFetchRepository = (projectId: number) => useQuery(['fetchRepository'], () => fetch(`${config.apiUrl}/projects/${projectId}/fetch`, {
//     method: 'POST',
//     body: JSON.stringify({}),
//     headers: {
//         'content-type': 'application/json'
//     }
// }))
