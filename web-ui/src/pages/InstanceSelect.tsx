import * as React from 'react';
import { NavLink } from 'react-router-dom';

import {
    Button, Card, CardContent, CardHeader, Divider, Grid, IconButton, List, ListItem,
    ListItemSecondaryAction, ListSubheader, MenuItem, Paper, Select, TextField, Typography
} from '@material-ui/core';

import DeleteIcon from '@material-ui/icons/Delete';

import { ConfigEditor, SimpleMapViewer } from '../components';
import { ConfigMetadata, FilterConfig, InstanceMetadata, MapData, MapMetadata } from '../model';
import Api from '../services/Api';

interface InstanceSelectState {
    instanceName: string;
    instances: InstanceMetadata[];
    maps: MapMetadata[];
    mapKey: string | null;
    selectedMap: MapData | null;
    selectedFilterConfig: string;
    filterConfigOptions: ConfigMetadata[];
    filterDialogOpen: boolean;
}

class InstanceSelect extends React.Component<{}, InstanceSelectState> {
    private api = new Api();

    constructor(props: {}) {
        super(props);
        this.state = {
            instanceName: 'new-instance',
            instances: [],
            maps: [],
            mapKey: null,
            selectedMap: null,
            selectedFilterConfig: 'default',
            filterConfigOptions: [],
            filterDialogOpen: false,
        };
    }

    public render() {
        const InstanceList = () => {
            if (this.state.instances.length === 0) {
                return <CardContent><Typography>No active instances</Typography></CardContent>;
            }

            const stopInstance = (instanceName: string) => () => this.stopInstance(instanceName);
            return (
                <List subheader={<ListSubheader component="div">Existing instances</ListSubheader>}>
                    {this.state.instances.map(i => (
                        <ListItem button={true} key={i.name}>
                            <NavLink to={`/viewer/${i.name}`} className="buttonLink">
                                {i.name} (map: {i.map_key})
                            </NavLink>
                            <ListItemSecondaryAction>
                                <IconButton onClick={stopInstance(i.name)}><DeleteIcon /></IconButton>
                            </ListItemSecondaryAction>
                        </ListItem>
                    ))}
                </List>
            );
        };

        const MapList = () => (
            <List subheader={<ListSubheader component="div">Maps</ListSubheader>} dense={true}>
                {this.state.maps.map(m => (
                    <ListItem key={m.map_key} selected={m.map_key === this.state.mapKey} button={true} onClick={this.mapSelected(m)}>
                        {m.map_key}
                    </ListItem>
                ))}
            </List>
        );

        const MapPreview = () => {
            if (this.state.selectedMap == null) {
                return <></>;
            }

            return (
                <Paper style={{ margin: 10, padding: 10, textAlign: 'center' }}>
                    <Typography variant="subtitle1">{this.state.mapKey}</Typography>
                    <SimpleMapViewer mapData={this.state.selectedMap} width={300} height={300} />
                    <Button variant="outlined" style={{ marginTop: 10 }} onClick={this.startInstance}>
                        Start Instance
                    </Button>
                </Paper>
            );
        };

        const filterConfigItems = (config: ConfigMetadata) => (
            <MenuItem key={config.key} value={config.key}>
                {config.key}
            </MenuItem>
        );

        return (
            <Grid container={true} spacing={10}>
                <Grid item={true} xs={3}>
                    <Card>
                        <CardHeader title="Active instances" />
                        <Divider />
                        <InstanceList />
                    </Card>
                </Grid>

                <Grid item={true} xs={9}>
                    <Card>
                        <CardHeader title="Start new instance" />

                        <Divider />
                        <CardContent>
                            <TextField
                                style={{ width: 200 }}
                                value={this.state.instanceName}
                                onChange={this.handleChange('instanceName')}
                            />
                            <br />

                            Particle filter config:

                            <Select
                                style={{ width: 200, marginLeft: 20 }}
                                value={this.state.selectedFilterConfig}
                                onChange={this.handleChange('selectedFilterConfig')}
                            >
                                {this.state.filterConfigOptions.map(filterConfigItems)}
                            </Select>

                            <ConfigEditor
                                title="Add or update filter config"
                                type="filter_config"
                                existingKey={this.state.selectedFilterConfig}
                                open={this.state.filterDialogOpen}
                                onClose={this.closeFilterDialog}
                            />
                            <Button variant="outlined" style={{ marginLeft: 20 }} onClick={this.openFilterDialog}>
                                Edit filter config
                            </Button>

                            {/* <Typography variant="subheading">Basestation mapping config (Optional, otherwise the existing basestation names are used as mapping)</Typography> */}
                        </CardContent>

                        <Divider />

                        <Grid container={true} spacing={10} style={{flexGrow: 1}}>
                            <Grid item={true} xs="auto" style={{flexGrow: 1}}>
                                <MapList />
                            </Grid>
                            <Grid item={true}>
                                <MapPreview />
                            </Grid>
                        </Grid>
                    </Card>
                </Grid>
            </Grid>
        );
    }

    public componentDidMount() {
        this.loadAsync();
    }

    private loadAsync = async () => {
        const [instances, maps, filterConfigOptions] = await Promise.all([
            this.api.get<InstanceMetadata[]>('instance'),
            this.api.get<MapMetadata[]>('map'),
            this.api.get<ConfigMetadata[]>('filter_config')
        ]);

        this.setState({ instances, maps, filterConfigOptions });
    }

    private mapSelected = (map: MapMetadata) => async () => {
        const api = new Api();
        const selectedMap = await api.get<MapData>(`map/${map.map_key}/config`);
        this.setState({ selectedMap, mapKey: map.map_key });
    };

    private startInstance = async () => {
        const instanceName = this.state.instanceName;

        const filterConfig = await this.api.get<FilterConfig>(`filter_config/${this.state.selectedFilterConfig}/value`)
        const instanceConfig = {
            'map_key': this.state.mapKey,
            'filter_config': filterConfig,
            'beacon_mapping': {},
        };

        await this.api.post(`instance/${instanceName}/start`, instanceConfig);

        await this.loadAsync();
    }

    private stopInstance = async (instanceName: string) => {
        await this.api.delete(`instance/${instanceName}`);
        const instances = await this.api.get<InstanceMetadata[]>('instance');
        this.setState({ instances });
    }

    private openFilterDialog = () => {
        this.setState({ filterDialogOpen: true });
    }

    private closeFilterDialog = async () => {
        await this.loadAsync();
        this.setState({ filterDialogOpen: false });
    }

    private handleChange = (name: string) => (event: React.ChangeEvent<any>) => {
        this.setState({ [name]: event.target.value } as any);
    }
}

export default InstanceSelect;