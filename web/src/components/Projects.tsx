import React from 'react';
import {useQuery} from "react-query";
import config from "../helpers/config";

interface ProjectProps {
    onChange: (id: number) => void
}
function Projects(props: ProjectProps) {
    const { isLoading, error, data } = useQuery('projectData', () =>
        fetch(`${config.apiUrl}/projects`).then(res =>
            res.json()
        )
    )


    if (isLoading) return <>Loading...</>

    if (error ) return (<>An error has occurred: {error}</>);


    return (
        <select onChange={e => {props.onChange(parseInt(e.target.value))}}>
            <option value={""}>Select Project</option>
            {data.map((r: any) => <option key={r.id} value={r.id}>{r.name} - {r.path}</option>)}
        </select>
    )
}
export default Projects;
