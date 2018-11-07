import * as React from 'react';
import './App.css';

import { RouteComponentProps } from 'react-router';
import { Link, Redirect, Route } from 'react-router-dom';

import { createMuiTheme, MuiThemeProvider } from '@material-ui/core';
import { AppBar, Button, Toolbar } from '@material-ui/core';

import * as Pages from './pages';

const theme = createMuiTheme({
});

const styles: { [index: string]: React.CSSProperties } = {
    appBar: {
        backgroundColor: '#e0e0e0',
        height: 64,
    },
    flex: {
        flex: 1,
    },
    menuButton: {
        color: '#fff',
        fontWeight: 'bold',
        marginLeft: -12,
        marginRight: 20,
        textTransform: 'none',
    },
};

class Navigation extends React.PureComponent {
    public render() {
        const NavButton = (props: { link: string, label: string }) => {
            return (
                <Link to={props.link}>
                    <Button style={styles.menuButton}>{props.label}</Button>
                </Link>
            );
        };

        return (
            <Toolbar>
                <span style={{ marginRight: 40 }}>bTracked Deployment Plan Designer</span>
                <nav>
                    <NavButton link="/editor" label="Map Editor" />
                    <NavButton link="/instances" label="Instances" />
                    <NavButton link="/calibration" label="Calibration" />
                </nav>
            </Toolbar>
        );
    }
}

class MainComponent extends React.Component {
    public render() {
        return (
            <>
                <AppBar>
                    <Navigation />
                </AppBar>
                <div style={{ padding: 15, marginTop: 90 }}>
                    <Route path="/editor" component={Pages.Editor} />
                    <Route path="/instances" component={Pages.InstanceSelect} />
                    <Route path="/viewer/:mapKey" component={Pages.InstanceViewer} />
                    <Route path="/calibration" component={Pages.Calibration} />
                    <Route exact={true} path="/" render={this.redirect} />
                </div>
            </>
        );
    }

    private redirect = () => <Redirect from="/" to="editor" />;
}


class App extends React.Component<RouteComponentProps<any>, {}> {
    public render() {
        return (
            <MuiThemeProvider theme={theme}>
                <MainComponent />
            </MuiThemeProvider>
        );
    }
}

export default App;
