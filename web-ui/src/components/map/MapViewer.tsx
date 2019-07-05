import * as React from 'react';

import { MapData, Point } from '../../model';
import { getCtxAndClearCanvas } from '../../utils';
import { getMousePositionOnCanvas } from './canvas';

import * as mode from './modes';

import MapRenderer from './MapRenderer';

const bgCanvasStyle: React.CSSProperties = {
    left: 0,
    position: 'absolute',
    right: 0,
    zIndex: -1,
};

const mainCanvasStyle: React.CSSProperties = {
    left: 0,
    position: 'absolute',
    right: 0,
    zIndex: 1,
};

const particleCanvasStyle: React.CSSProperties = {
    left: 0,
    position: 'absolute',
    right: 0,
    zIndex: 0,
};

const uiCanvasStyle: React.CSSProperties = {
    left: 0,
    position: 'absolute',
    right: 0,
    zIndex: 2,
};

export type RenderFunc = (ctx: CanvasRenderingContext2D, renderer: MapRenderer) => void;
export type ModeBuilder = (viewer: MapViewer) => mode.Mode;

interface MapViewerProps {
    style?: React.CSSProperties;
    width?: number;
    height?: number;
    modeBuilder?: ModeBuilder;
    drawBg?: RenderFunc;
    drawMain?: RenderFunc;
    drawParticles?: RenderFunc;
    onKeyPress?: (event: React.KeyboardEvent) => void;
}

export default class MapViewer extends React.Component<MapViewerProps, {}> {
    public renderer: MapRenderer = new MapRenderer();

    private bgCanvas: HTMLCanvasElement | null = null;
    private mainCanvas: HTMLCanvasElement | null = null;
    private particleCanvas: HTMLCanvasElement | null = null;
    private uiCanvas: HTMLCanvasElement | null = null;

    private currentMouse: Point = [0, 0];
    private mouseInCanvas: boolean = false;
    private mode = new mode.NoneMode();

    constructor(props: MapViewerProps) {
        super(props);
        if (this.props.modeBuilder != null) {
            this.mode = this.props.modeBuilder(this);
        }
    }

    public componentDidUpdate(prevProps: MapViewerProps) {
        if (prevProps == null || prevProps.modeBuilder !== this.props.modeBuilder) {
            this.mode = this.props.modeBuilder != null ?
                this.props.modeBuilder(this) :
                new mode.NoneMode();
        }
    }

    public componentDidMount() {
        this.drawAll();
    }

    public drawBg = () => {
        if (this.props.drawBg) {
            this.draw(this.bgCanvas, this.props.drawBg);
        }
    }

    public drawMain = () => {
        if (this.props.drawMain) {
            this.draw(this.mainCanvas, this.props.drawMain);
        }
    }

    public drawParticles = () => {
        if (this.props.drawParticles) {
            this.draw(this.particleCanvas, this.props.drawParticles);
        }
    }

    public drawUi = () => {
        const ctx = getCtxAndClearCanvas(this.uiCanvas);
        if (ctx != null) {
            ctx.canvas.focus();

            ctx.save();
            this.renderer.applyTransformationToCtx(ctx);
            this.mode.drawUi(ctx, this.mouseInCanvas ? this.getMousePos() : null);
            ctx.restore();
        }
    }

    public drawAll = () => {
        this.drawBg();
        this.drawMain();
        this.drawParticles();
        this.drawUi();
    }

    public render() {
        const falseCallback = (event: React.MouseEvent<HTMLCanvasElement>) => {
            event.preventDefault();
            return false;
        }

        const width = this.props.width != null ? this.props.width : 800;
        const height = this.props.height != null ? this.props.height : 800;

        return (
            <div style={{ position: 'relative', width, height, ... this.props.style }}>
                <canvas
                    ref={canvas => this.bgCanvas = canvas}
                    width={width}
                    height={height}
                    style={bgCanvasStyle}
                />
                <canvas
                    ref={canvas => this.mainCanvas = canvas}
                    width={width}
                    height={height}
                    style={mainCanvasStyle}
                />
                <canvas
                    ref={canvas => this.particleCanvas = canvas}
                    width={width}
                    height={height}
                    style={particleCanvasStyle}
                />
                <canvas
                    tabIndex={1}
                    ref={canvas => this.uiCanvas = canvas}
                    onMouseMove={this.onMouseMove}
                    onMouseDown={this.onMouseDown}
                    onMouseUp={this.onMouseUp}
                    onMouseLeave={this.onMouseLeave}
                    onMouseEnter={this.onMouseEnter}
                    onWheel={this.onWheel}
                    onContextMenu={falseCallback}
                    onKeyDown={this.props.onKeyPress}
                    width={width}
                    height={height}
                    style={uiCanvasStyle}
                />
            </div>
        );
    }

    private draw = (canvas: HTMLCanvasElement | null, func: RenderFunc) => {
        const ctx = getCtxAndClearCanvas(canvas);
        if (ctx != null) {
            func(ctx, this.renderer);
        }
    };

    private getMousePos = () => {
        const { offset, scale } = this.renderer.getTransformation();
        return [
            (this.currentMouse[0] - offset[0]) / scale,
            (this.currentMouse[1] - offset[1]) / scale,
        ];
    }

    private onMouseMove = (event: React.MouseEvent<HTMLCanvasElement>) => {
        this.currentMouse = getMousePositionOnCanvas(this.uiCanvas, event);
        this.mode.mouseMove(this.getMousePos());
        this.drawUi();
    }

    private onMouseDown = (event: React.MouseEvent<HTMLCanvasElement>) => {
        this.mode.mouseDown(this.getMousePos());
        this.drawUi();
    }

    private onMouseUp = (event: React.MouseEvent<HTMLCanvasElement>) => {
        const mousePos = this.getMousePos();
        this.mode.mouseRelease(mousePos);
        if (!this.mouseInCanvas) {
            return;
        }

        if (event.button === 0) {
            this.mode.leftClick(mousePos);
        }
        else if (event.button === 2) {
            this.mode.rightClick(mousePos);
        }
        this.drawUi();
    }

    private onWheel = (event: React.WheelEvent) => {
        this.mode.mouseScroll(event.deltaY > 0 ? 1 : -1, this.getMousePos());
        event.preventDefault();
        return false;
    }

    private onMouseLeave = () => {
        this.mouseInCanvas = false;
        this.mode.mouseRelease(this.getMousePos());
        this.drawUi();
    }

    private onMouseEnter = () => {
        this.mouseInCanvas = true;
        this.drawUi();
    }
}

interface SimpleMapViewerProps {
    width: number;
    height: number;
    mapData: MapData;
}

export class SimpleMapViewer extends React.Component<SimpleMapViewerProps, {}> {
    private viewer: MapViewer | null = null;

    constructor(props: SimpleMapViewerProps) {
        super(props);
    }

    public render() {
        return (
            <MapViewer
                ref={viewer => this.viewer = viewer}
                width={this.props.width}
                height={this.props.height}
                drawMain={this.drawMain}
            />
        );
    }

    public componentDidMount() {
        if (this.viewer != null) {
            this.viewer.drawMain();
        }
    }

    public componentDidUpdate() {
        if (this.viewer != null) {
            this.viewer.drawMain();
        }
    }

    private drawMain = (ctx: CanvasRenderingContext2D, renderer: MapRenderer) => {
        const { x, y, width, height } = this.props.mapData.boundary;
        const scale = Math.min(this.props.width / width, this.props.height / height);

        renderer.setCanvasOffsetUnscaled([-x * scale, -y * scale]);
        renderer.setZoomFromScale(scale);

        renderer.drawMap(ctx, this.props.mapData);
    }
}
