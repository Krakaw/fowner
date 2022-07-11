import {useFeatures} from "../hooks/queries.hooks";
import "./Table.css";

interface FeatureProps {
    start: any,
    end: any
}

function Features(props: FeatureProps) {

    const {isLoading, data = []} = useFeatures(props.start, props.end);
    if (isLoading) return <>Loading...</>
    return (
        <div>
            <table className={"styled-table"}>
                <thead>
                <tr>
                    <th>Features</th>
                </tr>
                </thead>
                <tbody>
                {data.map((r: any) => <tr key={r.id}>
                    <td>{r.name}</td>
                </tr>)}
                </tbody>
            </table>
        </div>
    )
}

export default Features;
