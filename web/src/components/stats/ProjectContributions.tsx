import Contribution from "./Contribution";

function ProjectContributions({data = {}}: { data: any }) {
    const contributions = Object.values(data.contributions).sort((a: any, b: any) => b.total_contributions - a.total_contributions);
    return (
        <>
            <h3>{data.project_name}</h3>
            {contributions.map((c: any) => <Contribution key={c.owner_id} contribution={c}/>)}
        </>
    )
}

export default ProjectContributions;
