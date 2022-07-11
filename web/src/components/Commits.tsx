import {useQuery} from "react-query";
import config from "../helpers/config";
import {useEffect, useState} from "react";
import {useCommits} from "../hooks/queries.hooks";

interface CommitsProps {
    projectId?: number,
    onCommitSelected: (key: string, commit: any) => void
}

function Commits(props: CommitsProps) {

    let initialState = {sha: null, commit_time: null};
    const [start, setStart] = useState(initialState);
    const [end, setEnd] = useState(initialState);
    const [page, setPage] = useState(0);
    const [limit, setLimit] = useState(100);
    const {isLoading, error, data = [], refetch} = useCommits(props.projectId || 0, page, limit)

    useEffect(() => {
        refetch()
    }, [page, limit, refetch]);
    if (error) return <>Error</>;

    if (isLoading) {
        return <>Loading ...</>
    }


    return (
        <table>
            <thead>
            <tr>
                <th>&nbsp;</th>

                <th>SHA</th>
                <th>Description</th>
                <th>Features</th>
                <th>Time</th>
            </tr>
            </thead>
            <tbody>
            {
                data.map((r: any) => <tr key={r.sha}>
                    <td><input type={"checkbox"}
                               disabled={!(start.sha === r.sha || end.sha === r.sha) && !!start.commit_time && !!end.commit_time}
                               checked={start.sha === r.sha || end.sha === r.sha}
                               onChange={(e) => {
                                   const checked = e.target.checked;

                                   if (checked) {
                                       if (!start.commit_time) {
                                           setStart(r);
                                           props.onCommitSelected('start', r);
                                       } else {
                                           setEnd(r);
                                           props.onCommitSelected('end', r);
                                       }

                                   } else {
                                       if (start.sha === r.sha) {
                                           setStart(initialState)
                                       } else if (end.sha === r.sha) {
                                           setEnd(initialState)
                                       }
                                   }

                               }}/></td>
                    <td>{r.sha.substring(0, 7)}</td>
                    <td>{r.description}</td>
                    <td>{r.feature_names.join(", ")}</td>
                    <td>{new Date(r.commit_time).toLocaleString()}</td>
                </tr>)
            }

            </tbody>
            <tfoot>
            <tr>
                <td>
                    <button disabled={page === 0} onClick={() => setPage(Math.max(page - 1, 0))}>Prev</button>
                </td>
                <td>
                    <button onClick={() => setPage(page + 1)}>Next</button>
                </td>
            </tr>
            </tfoot>
        </table>
    )
}

export default Commits;

