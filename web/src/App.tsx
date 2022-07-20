import React from 'react';
import './App.css';
import Projects from "./components/Projects";
import {QueryClient, QueryClientProvider} from 'react-query'
import {Link, Outlet, useParams} from "react-router-dom";


const queryClient = new QueryClient()

function App() {
    const {projectId, ownerId} = useParams();

    return (
        <QueryClientProvider client={queryClient}>
            <div className="App">
                <header className="App-header">
                    <Link to={"/"}>
                        <img src="/images/logo.svg" className="App-logo" alt="fowner-logo"/>
                    </Link>
                </header>
                {projectId || ownerId ?
                    <Outlet/>
                    :
                    <Projects showSelect={false}/>
                }

            </div>
        </QueryClientProvider>

    );
}

export default App;
