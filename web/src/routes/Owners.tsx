import {useOwners} from "../hooks/queries.hooks";
import React from "react";
import {Link} from "react-router-dom";

function Owners() {
    const {isLoading, error, data = []} = useOwners();
    if (isLoading) {
        return <span>Loading Owner...</span>
    }
    if (error) {
        return <span>Error Loading Owners</span>
    }

    return (<>
        <table className={"styled-table sticky"}>
            <thead>
            <tr>
                <th>Handle</th>
                <th>Name</th>
                <th>Primary Handle</th>
                <th>&nbsp;</th>
            </tr>
            </thead>
            <tbody>
            {data.sort((a: any, b: any) => a.handle.toLowerCase().localeCompare(b.handle.toLowerCase())).map((r: any) =>
                <tr>
                    <td>{r.handle}</td>
                    <td>{r.name}</td>
                    <td>{r.primary_owner_id ? data.find((d: any) => d.id === r.primary_owner_id).handle : ''}&nbsp;</td>
                    <td><Link to={`/owners/${r.id}`}>Edit</Link></td>
                </tr>)}
            </tbody>
        </table>
    </>)
}

export default Owners;
