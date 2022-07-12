import React, {useCallback, useEffect, useState} from 'react';
import './App.css';
import Projects from "./components/Projects";
import {QueryClient, QueryClientProvider} from 'react-query'
import config from "./helpers/config";
import {Outlet, useParams} from "react-router-dom";


const queryClient = new QueryClient()

function App() {
    const params = useParams();
    const projectId = params.projectId ? +params.projectId : undefined;
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


    useEffect(() => {
        setCommits({});
    }, [projectId]);


    return (


            <QueryClientProvider client={queryClient}>

            <div className="App">
                <header className="App-header">
                    <img src="/images/logo.svg" className="App-logo" alt="logo"/>

                    <span style={{flex: 1}}></span>

                    {projectId && <button onClick={() => {
                        fetchRepo()
                    }}>Update</button>}
                    <Projects showSelect={true} projectId={projectId} />
                    {commits.start?.sha}
                </header>
                {projectId ?
                    <div className={"Details"}>
                        <Outlet />

                    </div> :
                    (
                        <>
                            <h3>Choose a project to continue</h3>
                            <Projects showSelect={false}/>
                        </>
                    )
                }

            </div>
        </QueryClientProvider>

    );
}

export default App;
