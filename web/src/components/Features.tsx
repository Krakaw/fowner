import {useFeatures} from "../hooks/queries.hooks";
import "./Table.css";

interface FeatureProps {
    start?: string,
    end?: string
}

function Features({start, end}: FeatureProps) {

    const {isLoading = false, error = false, data = []} = useFeatures(start, end);
    return (
        <div>
            <table className={"styled-table"}>
                <thead>
                <tr>
                    <th>Features</th>
                </tr>
                </thead>
                <tbody>
                {error && <tr className={"error"}><td>Start and End required</td></tr>}
                {isLoading && <tr className={"loading"}><td>Loading...</td></tr>}
                {!isLoading && !error && data.length === 0 && <tr className={"loading"}><td>No Features</td></tr>}
                {data.map((r: any) => <tr key={r.id}>
                    <td>{r.name}</td>
                </tr>)}
                </tbody>
            </table>
        </div>
    )
}

export default Features;
