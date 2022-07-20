import {useParams} from "react-router-dom";
import {useOwner, useOwners} from "../hooks/queries.hooks";
import {useCallback, useEffect, useState} from "react";
import config from "../helpers/config";

function Owner() {
    const {ownerId = '0'} = useParams();
    const {isLoading, error, data = {}, refetch} = useOwner(parseInt(ownerId));
    const {isLoading: isLoadingOwners, error: errorOwners, data: dataOwners = []} = useOwners();
    const [name, setName] = useState<string>();
    const [primaryOwnerId, setPrimaryOwnerId] = useState<number>();
    const submit = useCallback(() => {
        fetch(`${config.apiUrl}/owners/${ownerId}`, {
            method: 'PUT',
            headers: {
                'content-type': 'application/json'
            },
            body: JSON.stringify({
                name,
                primary_owner_id: primaryOwnerId
            })
        }).then(() => {
            refetch();
        });
    }, [name, primaryOwnerId, ownerId, refetch]);
    useEffect(() => {
        setName(data.name || '');
        setPrimaryOwnerId(data.primary_owner_id || '')
    }, [data.name, data.primary_owner_id])
    if (isLoading) {
        return <span>Loading Owner...</span>
    }
    if (error) {
        return <span>Error Loading Owner: {ownerId}</span>
    }
    const dataOwnersSorted = dataOwners.filter((o: any) => o.id !== parseInt(ownerId)).sort((a: any, b: any) => {
        return a.handle.toLowerCase().localeCompare(b.handle.toLowerCase())
    });


    return (
        <div>
            <label>Handle <input type="text" disabled value={data.handle}/></label><br/>
            <label>Name <input placeholder={"Name"} type="text" value={name || ''}
                               onChange={(e) => setName(e.target.value || undefined)}/></label><br/>
            <label>Primary Handle
                <select value={primaryOwnerId} onChange={(e) => {
                    const value = e.target.value;
                    setPrimaryOwnerId(value ? +value : undefined)
                }}>
                    {isLoadingOwners && <option>Loading...</option>}
                    {errorOwners && <option>Uh oh... Error.</option>}
                    <option value={''}>No primary owner</option>
                    {dataOwnersSorted.map((o: any) =>
                        <option key={o.id}
                                value={o.id}>{o.handle}</option>)}

                </select>
            </label><br/>
            <button onClick={() => {
                submit()
            }}>Update
            </button>
        </div>
    )
}

export default Owner;
