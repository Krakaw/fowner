import React, {useState} from 'react';
import './App.css';
import Projects from "./components/Projects";
import {QueryClient, QueryClientProvider, useQuery} from 'react-query'
import Commits from "./components/Commits";

const queryClient = new QueryClient()

function App() {
    const [projectId, setProjectId] = useState<number>();
    return (
        <QueryClientProvider client={queryClient}>

            <div className="App">
                <header className="App-header">
                    <img src="images/logo.svg" className="App-logo" alt="logo"/>
                    <span style={{flex: 1}}></span>
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
