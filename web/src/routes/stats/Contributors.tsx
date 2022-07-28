import {useContributorStats} from "../../hooks/queries.hooks";
import ProjectContributions from "../../components/stats/ProjectContributions";
import {useParams} from "react-router-dom";


function Contributors() {


    const {projectId} = useParams();

    const {isLoading, error, data = {}} = useContributorStats(projectId ? parseInt(projectId) : undefined);
    if (isLoading) {
        return <span>Loading...</span>
    }
    if (error) {
        return <span>Error</span>
    }

    return (<>
        {Object.values(data).map((d: any) => <ProjectContributions key={d.project_id} data={d}/>)}
    </>)
}

export default Contributors;
