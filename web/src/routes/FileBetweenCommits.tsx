import {useFiles} from "../hooks/queries.hooks";
import "../styles/Table.css";
import {useEffect} from "react";
import {useParams} from "react-router-dom";
import config from "../helpers/config";


function FileBetweenCommits() {
    const {projectId, startSha: start, endSha: end} = useParams();
    const {isLoading: loading = false, isRefetching, error = false, data = [], refetch} = useFiles(start, end);
    const isLoading = loading || isRefetching;
    const inputPending = (!start || !end);
    const unlinkFile = (fileId: number) => {
        if (!window.confirm("This will remove all features linked with the file and stop any future features being linked.")) {
            return;
        }
        fetch(`${config.apiUrl}/projects/${projectId}/files/${fileId}/features`, {
            method: 'DELETE',
        }).then(() => {
            refetch();
        })
    };
    useEffect(() => {
        if (start && end) {
            refetch();
        }
    }, [start, end, refetch])
    return (
        <table style={{flex: 1}} className={"styled-table sticky"}>
            <thead>
            <tr className={'no-sticky'}>
                <th colSpan={4} style={{textAlign: 'center'}}>
                    Commits between {start?.substring(0, 7)} and {end?.substring(0, 7)}
                </th>
            </tr>
            <tr>
                <th>Path</th>
                <th>Features</th>
                <th>Owners</th>
                <th>&nbsp;</th>
            </tr>
            </thead>
            <tbody>
            {(error || inputPending) && <tr className={"error"}>
                <td colSpan={3}>Start and End required</td>

            </tr>}
            {isLoading && <tr className={"loading"}>
                <td colSpan={3}>Loading...</td>
            </tr>}
            {!inputPending && !isLoading && !error && data.length === 0 && <tr className={"loading"}>
                <td colSpan={3}>No Files</td>
            </tr>}
            {data.map((r: any) => <tr key={r.id}>
                <td>{r.path}</td>
                <td className={r.no_features ? 'loading' : ''}>{r.no_features ? "Not Tracked" : r.feature_names.join(", ")}</td>
                <td>{r.owners.join(", ")}</td>
                <td>
                    <button className={"icon-button"} onClick={() => {
                        unlinkFile(r.id);
                    }}>
                        🗑
                    </button>
                </td>
            </tr>)}
            </tbody>
        </table>)
}

export default FileBetweenCommits;
