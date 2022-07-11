import React, {useCallback, useEffect, useState} from 'react';
import './App.css';
import Projects from "./components/Projects";
import {QueryClient, QueryClientProvider} from 'react-query'
import Commits from "./components/Commits";
import config from "./helpers/config";
import Features from "./components/Features";

const queryClient = new QueryClient()

function App() {
    const [projectId, setProjectId] = useState<number>();
    const [start, setStart] = useState({sha: ''});
    const [end, setEnd] = useState({sha: ''});
    const [commits, setCommits] = useState<Record<string, any>>({});
    const fetchRepo = useCallback(() => {
        fetch(`${config.apiUrl}/projects/${projectId}/fetch`, {
            method: 'POST',
            headers: {
                'content-type': 'application/json'
            },
            body: JSON.stringify({})
        })
    }, [projectId])
    const onCommitSelected = (key: string, commit: any) => {
        if (key === 'start') {
            setStart(commit);
        } else {
            setEnd(commit)
        }
    }

    useEffect(() => {
        setCommits({});
    }, [projectId]);


    return (
        <QueryClientProvider client={queryClient}>

            <div className="App">
                <header className="App-header">
                    <img src="images/logo.svg" className="App-logo" alt="logo"/>
                    <span style={{flex: 1}}></span>

                    {projectId && <button onClick={() => {
                        fetchRepo()
                    }}>Update</button>}
                    <Projects showSelect={true} projectId={projectId} onChange={(id) => {
                        setProjectId(id);
                    }}/>
                    {commits.start?.sha}
                </header>
                {projectId ?
                    <div className={"Details"}>
                        <div>
                            {projectId &&
                                <Commits projectId={projectId} onCommitSelected={(k, c) => onCommitSelected(k, c)}/>}
                        </div>
                        <div className={"Features"}>
                            {(projectId && start?.sha && end?.sha) &&
                                <Features start={start.sha} end={end.sha}/>}
                        </div>
                    </div> :
                    (
                        <>
                            <h3>Choose a project to continue</h3>
                            <Projects showSelect={false} onChange={(id) => {
                                setProjectId(id);
                            }}/>
                        </>
                    )
                }

            </div>
        </QueryClientProvider>
    );
}

export default App;
