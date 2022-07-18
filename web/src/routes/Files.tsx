import {useFiles} from "../hooks/queries.hooks";
import "../styles/Table.css";
import {useEffect} from "react";
import {useParams} from "react-router-dom";


function Files() {
    const {startSha: start, endSha: end} = useParams();
    const {isLoading: loading = false, isRefetching, error = false, data = [], refetch} = useFiles(start, end);
    const isLoading = loading || isRefetching;
    const inputPending = (!start || !end);
    useEffect(() => {
        if (start && end) {
            refetch();
        }
    }, [start, end, refetch])
    return (<table style={{flex: 1}} className={"styled-table sticky"}>
        <thead>
        <tr>
            <th>Path</th>
            <th>Features</th>
            <th>Owners</th>
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
            <td className={r.no_features && 'loading'}>{r.no_features ? "Not Tracked" : r.feature_names.join(", ")}</td>
            <td>{r.owners.join(", ")}</td>
        </tr>)}
        </tbody>
    </table>)
}

export default Files;
