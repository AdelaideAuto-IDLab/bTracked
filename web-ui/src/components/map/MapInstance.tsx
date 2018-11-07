import * as React from 'react';

import { MapData, ParticleSnapshot, Target, UpdateMessage } from '../../model';

import * as mode from './modes';

import MapRenderer from './MapRenderer';
import MapViewer, { ModeBuilder } from './MapViewer';

interface MapInstanceProps {
    mapData: MapData;
    instanceName: string;
    simName: string;
}

export default class MapInstance extends React.Component<MapInstanceProps, {}> {
    private viewer: MapViewer | null;

    private particles: ParticleSnapshot = { stationary: [], moving: [] };

    private target: Target;
    private estimate: Target;

    private particlesChanged: boolean = false;
    private stopped: boolean = false;
    private modeBuilder: ModeBuilder;

    constructor(props: MapInstanceProps) {
        super(props);

        const { instanceName, simName } = this.props;
        this.modeBuilder = (viewer: MapViewer) => {
            return new mode.InstanceViewerMode(viewer, this.props.mapData, instanceName, simName);
        };

        const { x, y } = this.props.mapData.boundary;
        const scale = this.props.mapData.scale;
        this.estimate = new Target([x, y], scale);
        this.target = new Target([x, y], scale);
    }

    public render() {
        return (
            <MapViewer
                style={{ border: 'solid 1px black' }}
                ref={viewer => this.viewer = viewer}
                modeBuilder={this.modeBuilder}
                drawMain={this.drawMain}
                drawParticles={this.drawParticles}
            />
        );
    }

    public componentDidMount() {
        this.stopped = false;
        window.requestAnimationFrame(this.animate);
    }

    public componentWillUnmount() {
        this.stopped = true;
    }

    public updateTracking(update: UpdateMessage) {
        if (update.tracking != null) {
            this.particles = update.tracking.snapshot;
            this.particlesChanged = true;
        }
        if (update.tracking != null) {
            this.estimate.update(update.tracking.estimate);
        }

        if (update.sim_state != null) {
            this.target.update(update.sim_state);
        }
    }

    public animate = (time: number) => {
        if (this.stopped) {
            return;
        }

        this.target.tick(time);
        this.estimate.tick(time);

        this.viewer!.drawMain();

        if (this.particlesChanged) {
            this.viewer!.drawParticles();
            this.particlesChanged = false;
        }
        window.requestAnimationFrame(this.animate)
    }

    private drawMain = (ctx: CanvasRenderingContext2D, renderer: MapRenderer) => {
        renderer.drawMap(ctx, this.props.mapData);
        renderer.drawTrackingInfo(ctx, [
            { target: this.target, color: 'rgb(255, 0, 0)' },
            { target: this.estimate, color: 'rgb(255, 0, 255)' },
        ]);
    }

    private drawParticles = (ctx: CanvasRenderingContext2D, renderer: MapRenderer) => {
        renderer.drawParticles(ctx, this.props.mapData, this.particles);
    }
}
