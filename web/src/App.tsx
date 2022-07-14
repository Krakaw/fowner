import React, {useCallback, useEffect, useState} from 'react';
import './App.css';
import Projects from "./components/Projects";
import {QueryClient, QueryClientProvider} from 'react-query'
import config from "./helpers/config";
import {Outlet, useNavigate, useParams} from "react-router-dom";
import Update from "./components/Update";


const queryClient = new QueryClient()

function App() {
    const params = useParams();
    let navigate = useNavigate();
    const projectId = params.projectId ? +params.projectId : undefined;
    const [commits, setCommits] = useState<Record<string, any>>({});
    const [count, setCount] = useState(0);


    const deleteProject = useCallback(() => {
        if (!window.confirm("Are you sure you want to delete this project?")) {
            return;
        }
        fetch(`${config.apiUrl}/projects/${projectId}`, {
            method: 'DELETE',
            headers: {
                'content-type': 'application/json'
            },
        }).then(r => {
            setCount(0);
            navigate("/");
        })
    }, [projectId, navigate])


    useEffect(() => {
        setCommits({});
    }, [projectId]);


    return (


        <QueryClientProvider client={queryClient}>

            <div className="App">
                <header className="App-header">
                    <img src="/images/logo.svg" className="App-logo" alt="fowner-logo"/>

                    <span style={{flex: 1}} onClick={() => {
                        setCount(count + 1)
                    }}></span>
                    {projectId && count > 5 && <button onClick={() => {
                        deleteProject()
                    }}>Delete</button>}

                    {projectId && <Update projectId={projectId}/>}
                    <Projects showSelect={true} projectId={projectId}/>
                    {commits.start?.sha}
                </header>
                {projectId ?
                    <div className={"Details"}>
                        <Outlet/>

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
