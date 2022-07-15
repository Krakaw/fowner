import {useFeatures} from "../hooks/queries.hooks";
import "../styles/Table.css";
import {useEffect} from "react";

interface FeatureProps {
    start?: string,
    end?: string
}

function Features({start, end}: FeatureProps) {

    const {isLoading: loading = false, isRefetching, error = false, data = [], refetch} = useFeatures(start, end);
    const isLoading = loading || isRefetching;
    const inputPending = (!start || !end);
    useEffect(() => {
        if (start && end) {
            refetch();
        }
    }, [start, end, refetch])
    return (<table style={{flex: 1}} className={"styled-table no-height-fill sticky"}>
        <thead>
        <tr>
            <th>Features</th>
        </tr>
        </thead>
        <tbody>
        {(error || inputPending) && <tr className={"error"}>
            <td>Start and End required</td>
        </tr>}
        {isLoading && <tr className={"loading"}>
            <td>Loading...</td>
        </tr>}
        {!inputPending && !isLoading && !error && data.length === 0 && <tr className={"loading"}>
            <td>No Features</td>
        </tr>}
        {data.map((r: any) => <tr key={r.id}>
            <td>{r.name}</td>
        </tr>)}
        </tbody>
    </table>)
}

export default Features;
