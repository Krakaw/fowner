import React, {useCallback, useState} from 'react';
import './App.css';
import Projects from "./components/Projects";
import {QueryClient, QueryClientProvider, useQuery} from 'react-query'
import Commits from "./components/Commits";
import axios from "axios";
import config from "./helpers/config";

const queryClient = new QueryClient()

function App() {
    const [projectId, setProjectId] = useState<number>();
    const fetchRepo = useCallback(() => {
        axios.post(`${config.apiUrl}/projects/${projectId}/fetch`);
    }, [[projectId]])
    return (
        <QueryClientProvider client={queryClient}>

            <div className="App">
                <header className="App-header">
                    <img src="images/logo.svg" className="App-logo" alt="logo"/>
                    <span style={{flex: 1}}></span>

                    {projectId && <button onClick={() => {fetchRepo()}}>Update</button>}
                    <Projects onChange={(id) => {
                        setProjectId(id);
                    }}/>
                </header>
                {projectId && <Commits projectId={projectId}/>}
            </div>
        </QueryClientProvider>
    );
}

export default App;
