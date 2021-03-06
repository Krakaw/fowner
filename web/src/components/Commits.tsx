import React, {useEffect, useState} from "react";
import {useCommits} from "../hooks/queries.hooks";
import "../styles/Table.css";
import {Link, useParams, useSearchParams} from "react-router-dom";
import Features from "./Features";
import {DateTime} from "luxon";


function Commits() {
    const {projectId} = useParams();
    let [searchParams, setSearchParams] = useSearchParams();
    const start = searchParams.get("start") || '';
    const end = searchParams.get("end") || '';
    const [page, setPage] = useState(0);
    // eslint-disable-next-line
    const [limit, _setLimit] = useState(100);
    const {isLoading, error, data = {data: []}, refetch} = useCommits(projectId ? parseInt(projectId) : 0, page, limit)

    useEffect(() => {
        refetch()
    }, [page, limit, refetch]);


    if (error) return <>Error</>;

    if (isLoading) {
        return <>Loading ...</>
    }
    return (
        <div>
            <div className={"Commits"}>
                <table className={"styled-table sticky"}>
                    <thead>
                    <tr>
                        <th>
                            {start && end && (<Link to={`/files/${projectId}/${start}/${end}`}>Files</Link>)}
                        </th>
                        <th>SHA</th>
                        <th>Description</th>
                        <th>Features</th>
                        <th>Author</th>
                        <th>Time</th>
                    </tr>
                    </thead>
                    <tbody>
                    {
                        data.data.map((r: any) =>
                            <tr key={r.sha} className={(start === r.sha || end === r.sha) ? "active-row" : ""}>
                                <td>
                                    <input type={"checkbox"}
                                           checked={start === r.sha || end === r.sha}
                                           onChange={(e) => {
                                               const checked = e.target.checked;
                                               if (checked) {
                                                   if (!start) {
                                                       setSearchParams({
                                                           start: r.sha,
                                                           end: searchParams.get("end") || ''
                                                       })
                                                   } else {
                                                       setSearchParams({
                                                           end: r.sha,
                                                           start: searchParams.get("start") || ''
                                                       })
                                                   }
                                               } else {
                                                   if (start === r.sha) {
                                                       setSearchParams({start: '', end: searchParams.get("end") || ''})
                                                   } else if (end === r.sha) {
                                                       setSearchParams({
                                                           end: '',
                                                           start: searchParams.get("start") || ''
                                                       })
                                                   }
                                               }

                                           }}/></td>
                                <td>{r.sha.substring(0, 7)}</td>
                                <td>{r.description}</td>
                                <td>{r.feature_names.join(", ")}</td>
                                <td>{r.owner_handle}</td>
                                <td title={new Date(r.commit_time).toLocaleString()}>{DateTime.fromISO(r.commit_time).toRelative()}</td>
                            </tr>)
                    }

                    </tbody>
                    <tfoot>
                    <tr>
                        <td colSpan={6}>
                            <button disabled={page === 0} onClick={() => setPage(0)}>First</button>
                            <button disabled={page === 0} onClick={() => setPage(Math.max(page - 1, 0))}>Prev</button>
                            <button onClick={() => setPage(page + 1)}>Next</button>
                            <button onClick={() => setPage(Math.floor(data.paging.total / limit))}>Last</button>
                        </td>
                    </tr>
                    </tfoot>
                </table>
                <Features start={start} end={end}/>
            </div>
            <div className={"Features"}>

            </div>
        </div>
    )
}

export default Commits;

