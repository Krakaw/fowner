import React from 'react';
import {useRepositories} from "../hooks/queries.hooks";

interface ProjectProps {
    onChange: (id: number) => void
}
export const Projects = (props: ProjectProps) => {

    const { isLoading, error, data = [] } = useRepositories();

    if (isLoading) return <>Loading...</>

    if (error) return (<>An error has occurred</>);
    return (
        <select onChange={e => {props.onChange(parseInt(e.target.value))}}>
            <option value={""}>Select Project</option>
            {data.map((r: any) => <option key={r.id} value={r.id}>{r.name} - {r.path}</option>)}
        </select>
    )
}
export default Projects;
