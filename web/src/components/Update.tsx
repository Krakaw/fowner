import React, {useState} from "react";
import {useUpdateProject} from "../hooks/queries.hooks";
import Modal from "react-modal";
interface UpdateProps {
    projectId: number
}

function Update(props: UpdateProps) {
    const [stopAtSha, setStopAtSha] = useState<string | undefined>(undefined);
    const [skipGithubLabels, setSkipGithubLabels] = useState<boolean | undefined>(undefined);
    const {error, isLoading: loading, isRefetching, data = {}, refetch} = useUpdateProject(props.projectId, stopAtSha, skipGithubLabels);
    const isLoading = loading || isRefetching;
    const [showModal, setShowModal] = useState(false);
    return (
        <>
            <Modal
                isOpen={showModal}
                contentLabel="Fetch Updates From Project"
                onRequestClose={() => {
                    setShowModal(false)}}
                shouldCloseOnEsc={true}
                shouldCloseOnOverlayClick={true}
            >
                <div>
                    <input type={"text"} disabled={isLoading} placeholder={"Stop At Sha (Optional)"} onChange={(e) => {
                        const sha = e.target.value;
                        setStopAtSha(sha);
                    }}  />
                    <label><input type={"checkbox"} disabled={isLoading}
                    checked={skipGithubLabels}
                                  onChange={(e) => {
                        const checked = e.target.checked;
                        setSkipGithubLabels(checked);
                    }} />Do not fetch Github Tags</label>
                </div>
                <div style={{flexDirection: 'column'}}>
                    <div>Processed: {isLoading ? 'Loading...' : data.commits_processed !== undefined ? `${data.commits_processed}` : 'N/A'}</div>
                    <div>Total: {isLoading ? 'Loading...' : data.total_commits !== undefined ? `${data.total_commits}` : 'N/A'}</div>
                </div>
                <button disabled={isLoading} onClick={() => {
                    refetch()
                }}>Fetch{isLoading && 'ing'}</button>
                <button onClick={() => {setShowModal(false)}}>Cancel</button>
            </Modal>

            <button disabled={isLoading || !!error} onClick={() => {
                setShowModal(true)
            }}>{isLoading ? 'Fetching...' : 'Update'}</button>
        </>
    )
}

export default Update;
