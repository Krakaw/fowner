import React from 'react';
import {useRepositories} from "../hooks/queries.hooks";

interface ProjectProps {
    onChange: (id: number) => void,
    showSelect: boolean,
    projectId?: number
}

export const Projects = (props: ProjectProps) => {

    const {isLoading, error, data = []} = useRepositories();

    if (isLoading) return <>Loading...</>

    if (error) return (<>An error has occurred</>);
    return props.showSelect ? (
        <select
        value={props.projectId}
            onChange={e => {
            props.onChange(parseInt(e.target.value))
        }}>
            <option value={""}>Select Project</option>
            {data.map((r: any) => <option key={r.id} value={r.id}>{r.name} - {r.path}</option>)}
        </select>
    ) : (
        <ul className={"ProjectsSelect"}>
            {data.map((r: any) => <li key={r.id} onClick={() => {
                props.onChange(parseInt(r.id))
            }}>{r.name} - {r.path}</li>)}
        </ul>
    )
}
export default Projects;
