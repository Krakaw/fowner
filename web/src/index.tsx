import React from 'react';
import './index.css';
import App from './App';
import reportWebVitals from './reportWebVitals';
import ReactDOM from "react-dom/client";
import {HashRouter, Route, Routes,} from "react-router-dom";
import Commits from "./components/Commits";
import Files from "./routes/Files";

const root = ReactDOM.createRoot(
    document.getElementById('root') as HTMLElement
);
root.render(
    <React.StrictMode>
        <HashRouter>
            <Routes>
                <Route path={"/"} element={<App/>}>
                    <Route path={"/projects/:projectId"} element={<Commits/>}/>
                    <Route path={"/files/:projectId/:startSha/:endSha"} element={<Files/>}/>
                </Route>
            </Routes>
        </HashRouter>

    </React.StrictMode>
);

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals();
