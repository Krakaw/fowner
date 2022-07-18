import React from 'react';
import './App.css';
import Projects from "./components/Projects";
import {QueryClient, QueryClientProvider} from 'react-query'
import {Link, Outlet, useParams} from "react-router-dom";


const queryClient = new QueryClient()

function App() {
    const params = useParams();
    const projectId = params.projectId ? +params.projectId : undefined;

    return (
        <QueryClientProvider client={queryClient}>
            <div className="App">
                <header className="App-header">
                    <Link to={"/"}>
                        <img src="/images/logo.svg" className="App-logo" alt="fowner-logo"/>
                    </Link>

                </header>
                {projectId ?
                    <Outlet/>
                    :
                    <Projects showSelect={false}/>
                }

            </div>
        </QueryClientProvider>

    );
}

export default App;
