import * as React from 'react';

import {
    Button, Card, CardContent, CardHeader, Checkbox, Divider, FormControlLabel, Grid, IconButton,
    MenuItem, Radio, RadioGroup, Table, TableBody, TableCell, TableHead, TableRow,
    TextField, Typography,
} from '@material-ui/core';

import DeleteIcon from '@material-ui/icons/Delete';

import { FilePicker, MapEditor, SelectWithButton } from '../components';
import MapRenderer from '../components/map/MapRenderer';

import { MapData, MapMetadata, SignalSource } from '../model';
import Api from '../services/Api';

interface EditorState {
    mode: string;
    floorPlanImage: HTMLImageElement | null;
    collisionImage: HTMLImageElement | null;
    showFloorPlan: boolean;
    snapToPoints: boolean;
    snapAngles: boolean;
    snapAngle: number;
    saveFileName: string;
    loadFileName: string;
    existingMapFiles: MapMetadata[],
    beacons: BeaconData[];
}

interface BeaconData {
    id: string;
    info: SignalSource;
}

class Editor extends React.Component<{}, EditorState> {
    private mapEditor: MapEditor | null = null;
    private collisionMapLoading = false;

    constructor(props: {}) {
        super(props);
        this.state = {
            mode: 'adjustCanvas',
            floorPlanImage: null,
            collisionImage: null,
            showFloorPlan: true,
            snapToPoints: true,
            snapAngles: true,
            snapAngle: 10,
            saveFileName: 'Untitled',
            loadFileName: '',
            existingMapFiles: [],
            beacons: []
        };
    }

    public render() {
        return (
            <Grid container={true} spacing={24}>
                <Grid item={true}>
                    <MapEditor
                        ref={mapEditor => this.mapEditor = mapEditor}
                        mode={this.state.mode}
                        floorPlanImage={this.state.floorPlanImage}
                        collisionImage={this.state.collisionImage}
                        showFloorPlan={this.state.showFloorPlan}
                        snapToPoints={this.state.snapToPoints}
                        snapAngles={this.state.snapAngles}
                        snapAngle={this.state.snapAngle}
                        onKeyPress={this.keyPressed}
                        onBeaconsChanged={this.onBeaconsChanged}
                    />
                </Grid>
                <Grid item={true} xs="auto">
                    <Card>
                        {this.renderManagementContent()}
                    </Card>
                </Grid>
                <Grid item={true} xs="auto">
                    <Card>
                        {this.renderBeaconsEditor()}
                    </Card>
                </Grid>
            </Grid>
        );
    }

    public componentDidMount() {
        this.loadExistingMaps();
    }

    private async loadExistingMaps() {
        const api = new Api();
        this.setState({ existingMapFiles: await api.get<MapMetadata[]>('map') });
    }

    private onBeaconsChanged = (beacons: { [id: string]: SignalSource }) => {
        this.setState({
            beacons: Object.keys(beacons).map(key => ({ id: key, info: beacons[key] }))
        });
    }

    private renderManagementContent() {
        const EditorCheckBox = ({ label, property }: { label: string, property: string }) => (
            <FormControlLabel
                control={<Checkbox value={property} />}
                checked={this.state[property]}
                onChange={this.handleChecked(property)}
                label={label}
            />
        );

        const mapMenuItem = (map: MapMetadata) => (
            <MenuItem key={map.map_key} value={map.map_key}>
                {map.map_key}
            </MenuItem>
        );

        return (
            <>
                <CardHeader title="Editor" />
                <Divider />

                <CardContent>
                    <EditorCheckBox label="Snap to Points" property="snapToPoints" />
                    <br />
                    <EditorCheckBox label="Snap Angles" property="snapAngles" />
                    <TextField
                        label='Angle'
                        value={this.state.snapAngle}
                        onChange={this.handleChange('snapAngle')}
                    />

                    <br />
                    <FilePicker
                        style={{ marginRight: 20 }}
                        variant="outlined"
                        label="Load floor plan"
                        onFileSelected={this.floorPlanSelected}
                    />
                    <EditorCheckBox label="Show floor plan" property="showFloorPlan" />

                    <br />
                    <Button variant="outlined" onClick={this.generateCollisionMap}>Generate Collision Map</Button>
                </CardContent>

                <Divider />

                <CardContent>
                    <Typography variant="subheading">Editor mode</Typography>
                    <RadioGroup value={this.state.mode} onChange={this.handleChange('mode')}>
                        <FormControlLabel value="editWalls" control={<Radio />} label="Edit walls" />
                        <FormControlLabel value="editObstacles" control={<Radio />} label="Edit obstacles" />
                        <FormControlLabel value="addBeacons" control={<Radio />} label="Add beacons" />
                        <FormControlLabel value="addZone" control={<Radio />} label="Add zones" />
                        <FormControlLabel value="setScale" control={<Radio />} label="Set scale" />
                        <FormControlLabel value="setBoundary" control={<Radio />} label="Set boundary" />
                        <FormControlLabel value="adjustCanvas" control={<Radio />} label="Adjust canvas" />
                    </RadioGroup>
                </CardContent>

                <Divider />

                <CardContent>
                    <TextField
                        style={{ width: 200 }}
                        label='Name'
                        value={this.state.saveFileName}
                        onChange={this.handleChange('saveFileName')}
                    />
                    <Button style={{ marginLeft: 20 }} variant="outlined" onClick={this.save}>
                        Save
                    </Button>
                    <br />
                    <SelectWithButton
                        style={{ marginTop: 10 }}
                        value={this.state.loadFileName}
                        onChange={this.handleChange('loadFileName')}
                        onClick={this.load}
                        buttonText="Load"
                    >
                        {this.state.existingMapFiles.map(mapMenuItem)}
                    </SelectWithButton>
                    <br />
                    <Button variant="outlined" onClick={this.export}>Export Image</Button>
                </CardContent>
            </>
        );
    }

    private renderBeaconsEditor() {
        if (this.mapEditor == null) {
            return;
        }
        const mapEditor = this.mapEditor;

        const beaconEntries = this.state.beacons.map(beacon => {
            const deleteBeacon = () => mapEditor.removeBeacon(beacon.id);
            return (
                <TableRow key={beacon.id}>
                    <TableCell><TextField style={{ width: 80 }} value={beacon.id} /></TableCell>
                    <TableCell numeric={true}>{beacon.info.position[0]}</TableCell>
                    <TableCell numeric={true}>{beacon.info.position[1]}</TableCell>
                    <TableCell><IconButton onClick={deleteBeacon}><DeleteIcon /></IconButton></TableCell>
                </TableRow>
            )
        });

        return (
            <>
                <CardHeader title="Beacons" />
                <Divider />

                <Table padding="dense" cellSpacing={5}>
                    <TableHead>
                        <TableRow>
                            <TableCell>Name</TableCell>
                            <TableCell style={{ width: 15 }} numeric={true}>x</TableCell>
                            <TableCell style={{ width: 15 }} numeric={true}>y</TableCell>
                            <TableCell>Actions</TableCell>
                        </TableRow>
                    </TableHead>
                    <TableBody>
                        {beaconEntries}
                    </TableBody>
                </Table>
            </>
        );
    }

    private keyPressed = (event: React.KeyboardEvent) => {
        const KeyA = 65;
        const KeyB = 66;
        const KeyE = 69;
        const KeyS = 83;

        if (event.keyCode === KeyE) {
            this.setState({ mode: 'editWalls' });
        }
        else if (event.keyCode === KeyA) {
            this.setState({ mode: 'adjustCanvas' });
        }
        else if (event.keyCode === KeyB) {
            this.setState({ mode: 'addBeacons' });
        }
        else if (event.keyCode === KeyS) {
            this.setState({ mode: 'setScale' });
        }
    }

    private floorPlanSelected = (event: React.ChangeEvent<HTMLInputElement>) => {
        if (event.target.files != null) {
            const reader = new FileReader();

            reader.onload = () => {
                if (reader.result != null) {
                    const image = new Image();
                    image.src = reader.result as string;
                    image.onload = () => this.setState({ floorPlanImage: image });
                }
            }

            reader.readAsDataURL(event.target.files[0]);
        }
    }

    private save = async () => {
        if (this.mapEditor == null) {
            return;
        }

        const mapConfig = this.mapEditor.save();

        const api = new Api();
        await api.post('/map', {
            map_key: this.state.saveFileName,
            description: '',
            config: JSON.stringify(mapConfig)
        });

        await this.loadExistingMaps();
    }

    private load = async () => {
        if (this.mapEditor == null) {
            return;
        }

        const api = new Api();
        this.mapEditor.restore(await api.get<MapData>(`map/${this.state.loadFileName}/config`));
        this.setState({ saveFileName: this.state.loadFileName });
    }

    private generateCollisionMap = async () => {
        if (this.collisionMapLoading) {
            return;
        }
        this.collisionMapLoading = true;
        this.setState({ collisionImage: null });

        const image = new Image();
        image.src = `/api/map/${this.state.loadFileName}/collision?date=${Date.now()}`;
        image.onload = () => {
            this.collisionMapLoading = false;
            this.setState({ collisionImage: image });
        };
    }

    private handleChecked = (name: string) => (event: React.ChangeEvent<{}>, checked: boolean) => {
        this.setState({ [name]: checked } as any);
    }

    private handleChange = (name: string) => (event: React.ChangeEvent<any>) => {
        this.setState({ [name]: event.target.value } as any);
    }

    private export = () => {
        if (this.mapEditor == null) {
            return;
        }
        const mapData = this.mapEditor.mapManager.mapData;

        const canvas = document.createElement('canvas');
        canvas.width = mapData.boundary.width;
        canvas.height = mapData.boundary.height;

        const ctx = canvas.getContext('2d');
        if (ctx == null) {
            return;
        }

        ctx.fillStyle = 'rgb(255, 255, 255)';
        ctx.fillRect(0, 0, canvas.width, canvas.height);

        const renderer = new MapRenderer();
        renderer.setCanvasOffset([-mapData.boundary.x, -mapData.boundary.y]);
        renderer.drawMap(ctx, mapData);

        window.open(canvas.toDataURL(), '_blank');
    }
}

export default Editor;