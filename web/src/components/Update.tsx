import React from "react";
import {useUpdateProject} from "../hooks/queries.hooks";

interface UpdateProps {
    projectId: number
}
function Update(props: UpdateProps) {
    const {error, isLoading, data = {}, refetch} = useUpdateProject(props.projectId);
    
    return (
        <>
            <div style={{flexDirection: 'column'}}>
            <div>{data.commits_processed !== undefined ? `Processed: ${data.commits_processed}` : ''}</div>
            <div>{data.total_commits !== undefined ? `Total: ${data.total_commits}` : ''}</div>
            </div>
            <button disabled={isLoading || !!error} onClick={() => {
                refetch()
            }}>{isLoading ? 'Fetching...' : 'Update'}</button>
        </>
    )
}
export default Update;
