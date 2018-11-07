import * as React from 'react';
import * as ReactDOM from 'react-dom';
import { withRouter } from 'react-router';
import { BrowserRouter } from 'react-router-dom';

import App from './App';

import './index.css';
import registerServiceWorker from './registerServiceWorker';

const AppWithRouter = withRouter(App);

ReactDOM.render(
    (
        <BrowserRouter>
            <AppWithRouter />
        </BrowserRouter>
    ),
    document.getElementById('root') as HTMLElement
);
registerServiceWorker();
