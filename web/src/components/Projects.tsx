import React from 'react';
import {useRepositories} from "../hooks/queries.hooks";
import {Link, useNavigate} from "react-router-dom";

interface ProjectProps {
    onChange?: (id: number) => void,
    showSelect: boolean,
    projectId?: number
}

export const Projects = (props: ProjectProps) => {
    let navigate = useNavigate();
    const {isLoading, error, data = []} = useRepositories();

    if (isLoading) return <>Loading...</>

    if (error) return (<>An error has occurred</>);
    return props.showSelect ? (
        <select
            value={props.projectId}
            onChange={(e) => {
                const id = e.target.value;
                const url = id ? `/projects/${id}` : '/';
                navigate(url);
            }}
            >
            <option value={""}>Select Project</option>
            {data.map((r: any) => <option key={r.id} value={r.id}>{r.name} - {r.path}</option>)}
        </select>
    ) : (
        <ul className={"ProjectsSelect"}>
            {data.map((r: any) => <li key={r.id}>
                <Link to={`/projects/${r.id}`}>
                    {r.name} - {r.path}
                </Link>
            </li>)}
        </ul>
    )
}
export default Projects;
