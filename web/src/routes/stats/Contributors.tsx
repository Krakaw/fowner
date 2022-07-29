import {useCombinedContributorStats} from "../../hooks/queries.hooks";
import {useParams} from "react-router-dom";
import CombinedContribution from "../../components/stats/CombinedContribution";
import '../../styles/Contributors.css'

function Contributors() {
    const {projectId} = useParams();
    const {isLoading, error, data = []} = useCombinedContributorStats(projectId ? parseInt(projectId) : undefined);
    if (isLoading) {
        return <span>Loading...</span>
    }
    if (error) {
        return <span>Error</span>
    }
    return (<div className={"graph-container"}>
        {data.map((d: any) => <CombinedContribution key={d.owner_id} data={d}/>)}
    </div>)
}

export default Contributors;
