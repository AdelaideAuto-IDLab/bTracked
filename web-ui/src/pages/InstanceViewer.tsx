import * as React from 'react';

import { Button, Card, CardContent, CardHeader, Divider, Grid, LinearProgress, Typography } from '@material-ui/core';

import { MapInstance } from '../components';

import { distanceSqr } from '../utils';
import { InstanceDetails, ParticleState, UpdateMessage, } from '../model';
import Api from '../services/Api';

interface ViewerState {
    collisionImage: HTMLImageElement | null;
    instanceDetails: InstanceDetails | null;
    activeSim: string | null,

    estimatePose: number;
    estimatePos: number[];
    estimateMode: string;
    estimateTurnRate: number;
    estimateError: number;

    simPose: number;
    simPos: number[];
    simMode: string;
}

class Viewer extends React.Component<{ match: any }, ViewerState> {
    private api = new Api();

    private viewer: MapInstance | null = null;
    private updateWebSocket: WebSocket | null = null;
    private reconnectHandle: number | null = null;

    private updateInterval: number | null = null;
    private lastSim: ParticleState | null = null;
    private lastEstimate: ParticleState | null = null;

    constructor(props: { match: any }) {
        super(props);
        this.state = {
            collisionImage: null,
            instanceDetails: null,
            activeSim: null,

            estimatePose: 0.0,
            estimatePos: [0.0, 0.0],
            estimateMode: 'Stationary',
            estimateTurnRate: 0.0,
            estimateError: 0.0,

            simPose: 0.0,
            simPos: [0.0, 0.0],
            simMode: 'Stationary',
        };
    }

    public render() {
        if (this.state.instanceDetails == null) {
            return <LinearProgress />
        }
        const config = this.state.instanceDetails.config;
        const mapData = config.tracking.geometry;

        return (
            <Grid container={true} spacing={10}>
                <Grid item={true}>
                    <MapInstance
                        ref={viewer => this.viewer = viewer}
                        mapData={mapData}
                        instanceName={config.name}
                        simName="test-sim"
                    />
                </Grid>
                <Grid item={true} xs="auto">
                    <Card>
                        <CardHeader title="Estimated state" />
                        <Divider />

                        <CardContent>
                            <Typography>
                                x: {this.state.estimatePos[0].toFixed(3)},
                                y: {this.state.estimatePos[1].toFixed(3)} <br />

                                Pose: {this.state.estimatePose.toFixed(3)} <br />
                                {/* TurnRate: {this.state.estimateTurnRate} <br /> */}
                                Mode: {this.state.estimateMode}
                            </Typography>
                        </CardContent>
                    </Card>
                </Grid>

                <Grid item={true} xs="auto">
                    <Card>
                        <CardHeader title="Simulation" />
                        <Divider />

                        <CardContent>
                            <Button variant="outlined" color="primary" onClick={this.startSim}>
                                Start Sim
                            </Button>
                            {this.state.activeSim != null && (
                                <Button
                                    variant="outlined"
                                    color="secondary"
                                    onClick={this.stopSim}
                                    style={{ marginLeft: 20 }}
                                >
                                    Stop Sim
                                </Button>
                            )}
                        </CardContent>
                        <CardContent hidden={this.state.activeSim == null}>
                            <Typography>
                                x: {this.state.simPos[0].toFixed(3)},
                                y: {this.state.simPos[1].toFixed(3)} <br />
                                Pose: {this.state.simPose} <br />
                                Mode: {this.state.simMode}
                            </Typography>
                        </CardContent>
                    </Card>
                </Grid>
            </Grid>
        );
    }

    public componentDidMount() {
        this.loadAsync();
    }

    public componentWillUnmount() {
        if (this.updateInterval != null) {
            window.clearInterval(this.updateInterval);
            this.updateInterval = null;
        }

        if (this.reconnectHandle != null) {
            window.clearTimeout(this.reconnectHandle);
            this.reconnectHandle = null;
        }

        if (this.updateWebSocket != null) {
            this.updateWebSocket.close();
            this.updateWebSocket = null;
        }
    }

    private loadAsync = async () => {
        const instanceKey = this.props.match.params.mapKey;

        const api = new Api();
        const instanceDetails = await api.get<InstanceDetails>(`instance/${instanceKey}`);
        this.setState({ instanceDetails, activeSim: instanceDetails.attached_simulations[0] });

        this.connectToWs();
        this.updateInterval = window.setInterval(this.updateState, 1000)
    }

    private connectToWs = () => {
        if (this.updateWebSocket != null) {
            this.updateWebSocket.close();
        }

        const webSocket = new WebSocket(`ws://${window.location.host}/ws/listener`);
        webSocket.onmessage = (event) => this.newUpdate(JSON.parse(event.data));

        const instanceKey = this.props.match.params.mapKey;
        const config = {
            'tracking': {
                'TrackingListener': {
                    'instance_name': instanceKey,
                    'num_particles': 2000
                }
            },
            'sim_state': {
                'SimListener': {
                    'instance_name': instanceKey,
                    'sim_name': 'test-sim',
                    'update_rate': 200,
                }
            }
        };
        webSocket.onopen = (event) => {
            webSocket.send(JSON.stringify(config));
        }
        webSocket.onerror = (err) => {
            console.warn(err);
            if (this.reconnectHandle == null && this.updateWebSocket != null) {
                this.reconnectHandle = window.setTimeout(this.connectToWs, 200);
            }
        }
        this.updateWebSocket = webSocket;
        this.reconnectHandle = null;
    }

    private newUpdate = (data: UpdateMessage) => {
        if (this.viewer == null) {
            return;
        }
        this.viewer.updateTracking(data);

        if (data.sim_state != null) {
            this.lastSim = data.sim_state;
        }
        if (data.tracking != null) {
            this.lastEstimate = data.tracking.estimate;
        }
    }

    private updateState = () => {
        const stateUpdate: any = {};

        if (this.lastSim != null) {
            stateUpdate.simPos = this.lastSim.position;
            stateUpdate.simMode = this.lastSim.mode;
            stateUpdate.simPose = this.lastSim.pose;
        }

        if (this.lastEstimate != null) {
            stateUpdate.estimatePos = this.lastEstimate.position;
            stateUpdate.estimateMode = this.lastEstimate.mode;
            stateUpdate.estimatePose = this.lastEstimate.pose;
            stateUpdate.estimateTurnRate = this.lastEstimate.turn_rate;
        }

        if (this.lastEstimate != null && this.lastSim != null) {
            stateUpdate.estimateError = Math.sqrt(distanceSqr(this.lastEstimate.position, this.lastSim.position));
        }

        this.setState(stateUpdate);
    }

    private startSim = async () => {
        if (this.state.instanceDetails == null) {
            return;
        }
        const config = {
            initial_state: {
                position: [this.state.estimatePos[0], this.state.estimatePos[1], 0.0],
                velocity: [0.0, 0.0, 0.0],
                pose: 0.0,
                turn_rate: 0.0,
                mode: "Stationary",
            },
            min_rssi: -100,
        };
        await this.api.post(`instance/${this.state.instanceDetails.config.name}/sim/test-sim`, config);
        this.setState({ activeSim: 'test-sim' });
    }

    private stopSim = async () => {
        if (this.state.instanceDetails == null) {
            return;
        }
        await this.api.delete(`instance/${this.state.instanceDetails.config.name}/sim/test-sim`);
        this.setState({ activeSim: null });
    }

    // private handleChange = (name: string) => (event: React.ChangeEvent<any>) => {
    //     this.setState({ [name]: event.target.value } as any);
    // }
}

export default Viewer;