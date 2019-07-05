import * as React from 'react';

import {
    Button, Card, CardContent, CardHeader, Divider, Grid, IconButton, List, ListItem, ListSubheader,
    MenuItem, Table, TableBody, TableCell, TableHead, TableRow, Typography
} from '@material-ui/core';

import RefreshIcon from '@material-ui/icons/Refresh';

import { ConfigEditor, SelectWithButton } from '../components';
import { drawCircle } from '../components/map/canvas';
import { ConfigMetadata, InstanceMetadata } from '../model';
import Api from '../services/Api';
import { getCtxAndClearCanvas } from '../utils';

interface MeasurementInfo {
    latestRssi: number;
    count: number;
    prevCount: number;
    currentReadRate: number;
    firstSeen: number;
}

interface RssiData {
    time: number;
    rssi: number;
}

interface CalibrationState {
    instances: InstanceMetadata[];
    selectedInstance: string | null;
    selectedMac: string | null;
    existingSignalModels: SignalModelConfig[];
    measurementStats: Map<string, MeasurementInfo>;
}

export default class Calibration extends React.Component<{}, CalibrationState> {
    private measurements: Map<string, MeasurementInfo> = new Map();
    private measurementRssi: Map<string, RssiData[]> = new Map();

    private updateInterval: number | null = null;
    private updateWebSocket: WebSocket | null = null;
    private reconnectHandle: number | null = null;

    constructor(props: {}) {
        super(props);
        this.state = {
            instances: [],
            selectedInstance: null,
            selectedMac: null,
            existingSignalModels: [],
            measurementStats: new Map(),
        };
    }

    public render() {
        const InstanceList = () => {
            if (this.state.instances.length === 0) {
                return <CardContent><Typography>No active instances</Typography></CardContent>;
            }

            return (
                <List subheader={<ListSubheader component="div">Instances</ListSubheader>}>
                    {this.state.instances.map(i => (
                        <ListItem key={i.name} selected={i.name === this.state.selectedInstance} onClick={this.instanceSelected(i)} button={true}>
                            {i.name} (map: {i.map_key})
                        </ListItem>
                    ))}
                </List>
            );
        };

        const MeasurementTable = () => {
            const measurements = Array.from(this.state.measurementStats.entries()).map(entry => {
                const resetFirstSeen = () => {
                    const measurement = this.measurements.get(entry[0])!;
                    measurement.firstSeen = Date.now();
                    measurement.count = 0;
                    measurement.prevCount = 0;
                    this.measurementRssi.set(entry[0], []);
                };
                const setSelected = () => {
                    this.setState({ selectedMac: entry[0] });
                };

                const selected = this.state.selectedMac === entry[0];

                const dt = (Date.now() - entry[1].firstSeen) / 1000.0;
                return (
                    <TableRow key={entry[0]} hover={true} selected={selected} onClick={setSelected}>
                        <TableCell>{entry[0]}</TableCell>
                        <TableCell numeric={true}>{entry[1].latestRssi}</TableCell>
                        <TableCell numeric={true}>{entry[1].count}</TableCell>
                        <TableCell numeric={true}>{(entry[1].count / dt).toFixed(4)}</TableCell>
                        <TableCell numeric={true}>{entry[1].currentReadRate}</TableCell>
                        <TableCell style={{ width: 50 }}>
                            <IconButton onClick={resetFirstSeen}><RefreshIcon /></IconButton>
                        </TableCell>
                    </TableRow>
                );
            });

            return (
                <Table padding="dense">
                    <TableHead>
                        <TableRow>
                            <TableCell>MAC</TableCell>
                            <TableCell numeric={true}>Rssi</TableCell>
                            <TableCell numeric={true}>Count</TableCell>
                            <TableCell numeric={true}>Avg Read Rate</TableCell>
                            <TableCell numeric={true}>Current Read Rate</TableCell>
                            <TableCell style={{ width: 50 }} />
                        </TableRow>
                    </TableHead>
                    <TableBody>
                        {measurements}
                    </TableBody>
                </Table>
            );
        };

        return (
            <Grid container={true} spacing={24}>
                <Grid item={true} xs={3}>
                    <Card>
                        <CardHeader title="Active instances" />
                        <Divider />
                        <InstanceList />
                    </Card>
                </Grid>

                <Grid item={true} xs={9}>
                    <Card>
                        <CardHeader title="Measurements" />
                        <Divider />
                        {this.state.selectedInstance != null && <MeasurementTable />}
                    </Card>

                    {this.state.selectedMac != null &&
                        (
                            <Card style={{ marginTop: 20 }}>
                                <MeasurementViewer data={this.measurementRssi.get(this.state.selectedMac)} />
                            </Card>
                        )
                    }

                    <SignalModelConfig />
                </Grid>
            </Grid>
        );
    }

    public componentDidMount() {
        this.loadAsync();
        this.updateInterval = window.setInterval(this.updateState, 1000)
    }

    public componentWillUnmount() {
        if (this.updateInterval != null) {
            window.clearInterval(this.updateInterval);
            this.updateInterval = null;
        }

        if (this.updateWebSocket != null) {
            this.updateWebSocket.close();
            this.updateWebSocket = null;
        }
    }

    private loadAsync = async () => {
        const api = new Api();
        const instances = await api.get<InstanceMetadata[]>(`instance`);
        this.setState({ instances });
    }

    private connectToWs = (selectedInstance: string) => {
        if (this.updateWebSocket != null) {
            this.updateWebSocket.close();
        }

        const webSocket = new WebSocket(`ws://${window.location.host}/ws/listener`);
        webSocket.onmessage = (event) => this.newMeasurement(JSON.parse(event.data));
        const config = {
            'measurement': {
                'MeasurementListener': {
                    'instance_name': selectedInstance,
                    'raw': true,
                }
            }
        };
        webSocket.onopen = (event) => {
            webSocket.send(JSON.stringify(config));
        }
        webSocket.onerror = (err) => {
            console.warn(err);
            if (this.reconnectHandle == null && this.updateWebSocket != null) {
                this.reconnectHandle =
                    window.setTimeout(() => this.connectToWs(selectedInstance), 200);
            }
        }
        this.updateWebSocket = webSocket;
    }

    private newMeasurement = (data: { measurement: { mac: string, rssi: number } }) => {
        const { mac, rssi } = data.measurement;

        let stats = this.measurements.get(mac);
        if (stats == null) {
            stats = {
                latestRssi: 0,
                count: 0,
                prevCount: 0,
                firstSeen: Date.now(),
                currentReadRate: 0
            };
        };

        stats.count += 1;
        stats.latestRssi = rssi;

        this.measurements.set(mac, stats);

        let rssiValues = this.measurementRssi.get(mac);
        if (rssiValues == null) {
            rssiValues = [];
        }
        rssiValues.push({ time: Date.now(), rssi });
        this.measurementRssi.set(mac, rssiValues);
    }

    private instanceSelected = (instance: InstanceMetadata) => async () => {
        this.connectToWs(instance.name);
        this.setState({ selectedInstance: instance.name });
    };

    private updateState = () => {
        this.measurements.forEach(m => {
            m.currentReadRate = m.count - m.prevCount;
            m.prevCount = m.count;
        });
        this.setState({ measurementStats: this.measurements });
    }

    // private handleChange = (name: string) => (event: React.ChangeEvent<any>) => {
    //     this.setState({ [name]: event.target.value } as any);
    // }
}

interface MeasurementViewerProps {
    data: RssiData[] | undefined;
}

class MeasurementViewer extends React.Component<MeasurementViewerProps, {}> {
    private canvas: HTMLCanvasElement | null = null;

    public render() {
        return (
            <canvas
                ref={canvas => this.canvas = canvas}
                width={800}
                height={400}
                style={{ border: 'solid 1px black' }}
            />
        );
    }

    public componentDidMount() {
        this.drawRssiData();
    }

    public componentDidUpdate() {
        this.drawRssiData();
    }

    private drawRssiData() {
        const data = this.props.data;
        const ctx = getCtxAndClearCanvas(this.canvas);
        if (ctx == null || data == null || data.length < 2) {
            return;
        }

        ctx.save();

        const xScale = 800 / (data[data.length - 1].time - data[0].time);
        const yScale = 400 / 100;

        for (let i = 0; i <= 5; ++i) {
            const step = i / 5 * 100;
            ctx.fillText(`${-step}`, 0, step * yScale);
        }

        for (const item of data) {
            drawCircle(ctx, [xScale * (item.time - data[0].time), -yScale * item.rssi], 2);
        }

        ctx.restore();
    }
}

interface SignalModelConfigState {
    selectedModel: string;
    dialogOpen: boolean;
    signalModels: ConfigMetadata[];
}

class SignalModelConfig extends React.Component<{}, SignalModelConfigState> {
    private api = new Api();

    constructor(props: {}) {
        super(props);
        this.state = {
            selectedModel: '',
            dialogOpen: false,
            signalModels: [],
        };
    }

    public componentDidMount() {
        this.loadAsync();
    }

    public render() {
        const modelMenuItem = (model: ConfigMetadata) => (
            <MenuItem key={model.key} value={model.key}>
                {model.key}
            </MenuItem>
        );

        return (
            <Card style={{ marginTop: 20 }}>
                <CardHeader title="Signal model" />
                <Divider />
                <CardContent>
                    <SelectWithButton
                        buttonText="Load model"
                        value={this.state.selectedModel}
                        onChange={this.handleChange('selectedModel')}
                    >
                        {this.state.signalModels.map(modelMenuItem)}
                    </SelectWithButton>

                    <ConfigEditor
                        title="Add or Update signal model"
                        type="signal_model"
                        existingKey={this.state.selectedModel}
                        open={this.state.dialogOpen}
                        onClose={this.closeModelDialog}
                    />
                    <Button variant="outlined" style={{ marginLeft: 20 }} onClick={this.openModelDialog}>
                        Edit signal model
                    </Button>
                </CardContent>
            </Card>
        )
    }

    private loadAsync = async () => {
        const signalModels = await this.api.get<any[]>('signal_model');
        this.setState({ signalModels });
    }

    private openModelDialog = () => {
        this.setState({ dialogOpen: true });
    }

    private closeModelDialog = async () => {
        await this.loadAsync();
        this.setState({ dialogOpen: false });
    }

    private handleChange = (name: string) => (event: React.ChangeEvent<any>) => {
        this.setState({ [name]: event.target.value } as any);
    }
}