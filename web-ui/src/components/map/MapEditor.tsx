import * as React from 'react';

import { MapData, SignalSource } from '../../model';
import { updateMap } from '../../utils';

import * as mode from './modes';

import MapManager from './MapManager';
import MapRenderer from './MapRenderer';
import MapViewer, { ModeBuilder } from './MapViewer';

interface MapEditorProps {
    mode: string;
    floorPlanImage?: HTMLImageElement | null;
    collisionImage?: HTMLImageElement | null;
    showFloorPlan?: boolean;
    snapToPoints?: boolean;
    snapAngles?: boolean;
    snapAngle?: number;
    onKeyPress?: (event: React.KeyboardEvent) => void;
    onBeaconsChanged?: (beacons: { [id: string]: SignalSource }) => void;
}

export default class MapEditor extends React.Component<MapEditorProps, {}> {
    public mapManager: MapManager;
    private viewer: MapViewer | null;
    private modes: { [name: string]: ModeBuilder };

    constructor(props: MapEditorProps) {
        super(props);

        this.mapManager = new MapManager(this.props.onBeaconsChanged);
        this.modes = {
            'editWalls': (viewer: MapViewer) => new mode.EditWallMode(viewer, this.mapManager),
            'editObstacles': (viewer: MapViewer) => new mode.EditObstaclesMode(viewer, this.mapManager),
            'addBeacons': (viewer: MapViewer) => new mode.AddBeaconMode(viewer, this.mapManager),
            'addZone': (viewer: MapViewer) => new mode.AddZone(viewer, this.mapManager),
            'setScale': (viewer: MapViewer) => new mode.SetScale(viewer, this.mapManager),
            'setBoundary': (viewer: MapViewer) => new mode.SetBoundary(viewer, this.mapManager),
            'adjustCanvas': (viewer: MapViewer) => new mode.AdjustCanvas(viewer),
        };
    }

    public render() {
        return (
            <MapViewer
                style={{ border: 'solid 1px black' }}
                ref={viewer => this.viewer = viewer}
                modeBuilder={this.modes[this.props.mode]}
                onKeyPress={this.props.onKeyPress}
                drawBg={this.drawBg}
                drawMain={this.drawMain}
            />
        );
    }

    public componentDidUpdate(prevProps: MapEditorProps) {
        if (prevProps.floorPlanImage !== this.props.floorPlanImage ||
            prevProps.showFloorPlan !== this.props.showFloorPlan ||
            prevProps.collisionImage !== this.props.collisionImage
        ) {
            this.viewer!.drawBg();
        }

        this.mapManager.snapAngle = 0.0;
        if (this.props.snapAngles === true && this.props.snapAngle != null) {
            this.mapManager.snapAngle = this.props.snapAngle;
        }
        this.mapManager.enableSnapPoints = this.props.snapToPoints === true;
    }

    public save(): MapData {
        return this.mapManager.mapData;
    }

    public restore(data: MapData) {
        updateMap(data);
        this.mapManager.restore(data)
        if (this.viewer != null) {
            this.viewer.drawMain();
        }
    }

    public removeBeacon(id: string) {
        this.mapManager.removeBeacon(id);
    }

    private drawBg = (ctx: CanvasRenderingContext2D, renderer: MapRenderer) => {
        if (this.props.collisionImage != null) {
            renderer.drawMapImage(
                ctx,
                this.mapManager.mapData,
                100.0,
                this.props.collisionImage,
                1.0
            );
        }
        if (this.props.floorPlanImage != null && this.props.showFloorPlan === true) {
            renderer.drawBgImage(ctx, this.props.floorPlanImage, 0.3);
        }
    }

    private drawMain = (ctx: CanvasRenderingContext2D, renderer: MapRenderer) => {
        renderer.drawMap(ctx, this.mapManager.mapData);
    }
}
