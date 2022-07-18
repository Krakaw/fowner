import React from "react";
import {useDeleteProject} from "../hooks/queries.hooks";

interface UpdateProps {
    projectId: number
}

function Delete(props: UpdateProps) {
    const {
        error,
        isLoading: loading,
        isRefetching,
        refetch
    } = useDeleteProject(props.projectId);
    const isLoading = loading || isRefetching;
    const deletePrompt = () => {
        if (window.confirm("Are you sure you want to delete this Project?")) {
            refetch();
        }
    };
    return (
        <button disabled={isLoading || !!error} onClick={() => {
            deletePrompt();
        }}>{isLoading ? 'Deleting...' : 'Delete'}</button>
    )
}

export default Delete;
