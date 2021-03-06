import React, {useState} from 'react';
import {useRepositories} from "../hooks/queries.hooks";
import {Link} from "react-router-dom";
import Update from "./Update";
import Delete from "./Delete";

interface ProjectProps {
    onChange?: (id: number) => void,
    showSelect: boolean,
    projectId?: number
}

export const Projects = (props: ProjectProps) => {
    const {isLoading, error, data = []} = useRepositories();
    const [count, setCount] = useState(0);
    if (isLoading) return <>Loading...</>

    if (error) return (<>An error has occurred</>);
    return (
        <table className={"styled-table"} style={{width: '100%'}} onClick={() => {
            setCount(0);
        }}>
            <thead>
            <tr>
                <th>Choose a Project</th>
                <th>&nbsp;</th>
            </tr>
            </thead>
            <tbody>
            {data.map((r: any) => <tr onClick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                setCount(count + 1)
            }} key={r.id}>
                <td>
                    <Link to={`/projects/${r.id}`}>
                        {r.name} - {r.path}
                    </Link>
                </td>
                <td>
                    <div style={{display: 'flex', flexDirection: 'row'}}>
                        <Update projectId={r.id}/>
                        <span style={{flex: 1}}></span>
                        {count > 4 &&
                            <Delete projectId={r.id}/>
                        }
                    </div>
                    <Link to={`/stats/contributors/${r.id}`}>
                        Contributor Stats
                    </Link>
                </td>

            </tr>)}
            </tbody>

        </table>
    )
}
export default Projects;
