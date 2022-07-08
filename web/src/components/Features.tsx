import {useQuery} from "react-query";
import config from "../helpers/config";
import {useFeatures} from "../hooks/queries.hooks";

interface FeatureProps {
    start: any,
    end: any
}
function Features(props: FeatureProps) {

    const {isLoading, error, data = []} = useFeatures(props.start, props.end);
    if (isLoading) return <>Loading...</>
    console.log(data);
    return (
        <div>
            Features
        <ul>
            {data.map((r: any) => <li>{r.name}</li>)}
        </ul>
        </div>
    )
}

export default Features;
