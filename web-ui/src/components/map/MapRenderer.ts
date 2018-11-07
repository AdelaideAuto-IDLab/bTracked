import { MapData, ParticleSnapshot, Point, Target } from '../../model';
import { drawBeacons, drawLine, drawPolygon, drawRects, drawTriangle, drawWalls } from './canvas';

export interface TrackingInfoTarget {
    target: Target;
    color: string;
}

class MapRenderer {
    public hasChanged: boolean = true;

    private canvasOffset: Point = [0, 0];
    private canvasZoom: number = 1.0;

    public drawMap(ctx: CanvasRenderingContext2D, mapData: MapData) {
        ctx.save();
        this.applyTransformationToCtx(ctx);

        drawWalls(ctx, mapData.walls, 'red');
        drawRects(ctx, mapData.obstacles, 'rgba(100,100,100,0.9)');
        for (const key of Object.keys(mapData.zones)) {
            drawPolygon(ctx, mapData.zones[key], 'rgba(100,0,0,0.1)');
        }
        drawBeacons(ctx, mapData.signal_sources, 'blue');

        const { x, y, width, height } = mapData.boundary;
        ctx.strokeRect(x, y, width, height);

        ctx.restore();
    }

    public drawBgImage(ctx: CanvasRenderingContext2D, image: HTMLImageElement, alpha: number) {
        ctx.save();
        this.applyTransformationToCtx(ctx);
        ctx.globalAlpha = alpha;
        ctx.drawImage(image, 0, 0);
        ctx.restore();
    }

    public drawMapImage(
        ctx: CanvasRenderingContext2D,
        mapData: MapData,
        imageScale: number,
        image: HTMLImageElement,
        alpha: number
    ) {
        ctx.save();
        this.applyTransformationToCtx(ctx);

        const { x, y } = mapData.boundary;
        const scale = mapData.scale / imageScale;

        ctx.translate(x, y);
        ctx.scale(scale, scale);

        ctx.globalAlpha = alpha;
        ctx.drawImage(image, 0, 0);
        ctx.restore();
    }

    public drawTrackingInfo(ctx: CanvasRenderingContext2D, info: TrackingInfoTarget[]) {
        ctx.save();
        this.applyTransformationToCtx(ctx);

        for (const { target, color } of info) {
            ctx.strokeStyle = color;
            drawLine(ctx, target.trail);
        }

        for (const { target, color } of info) {
            ctx.fillStyle = color;
            drawTriangle(ctx, target.getLocation(), target.getPose());
        }
        ctx.restore();
    }

    public drawParticles(
        ctx: CanvasRenderingContext2D,
        mapData: MapData,
        particles: ParticleSnapshot
    ) {
        ctx.save();
        this.applyTransformationToCtx(ctx);

        const { x, y } = mapData.boundary;
        const scale = mapData.scale;

        ctx.fillStyle = 'rgba(0, 255, 0, 0.6)';
        const moving = particles.moving;
        for (let i = 0; i < moving.length; i += 2) {
            ctx.fillRect(scale * moving[i] + x, scale * moving[i + 1] + y, 2, 2);
        }

        ctx.fillStyle = 'rgba(0, 0, 0, 0.4)';
        const stationary = particles.stationary;
        for (let i = 0; i < stationary.length; i += 2) {
            ctx.fillRect(scale * stationary[i] + x, scale * stationary[i + 1] + y, 2, 2);
        }
        ctx.restore()
    }

    public applyTransformationToCtx(ctx: CanvasRenderingContext2D) {
        const { offset, scale } = this.getTransformation();
        ctx.translate(offset[0], offset[1]);
        ctx.scale(scale, scale);
    }

    public getTransformation() {
        return {
            offset: this.canvasOffset,
            scale: 1.0 / this.canvasZoom
        };
    }

    public updateZoom(amount: number, centerPoint: Point)  {
        const newZoom = this.canvasZoom * Math.pow(1.5, amount);

        // To apply the zoom from the perspective of `centerPoint`, we need to adjust the canvas
        // offset such that `centerPoint` remains in the same relative position on the canvas
        // before and after the zoom.
        const zoomOffsetFactor = (newZoom - this.canvasZoom) / (newZoom * this.canvasZoom);
        this.canvasOffset[0] += centerPoint[0] * zoomOffsetFactor;
        this.canvasOffset[1] += centerPoint[1] * zoomOffsetFactor;

        this.canvasZoom = newZoom;
        this.hasChanged = true;
    }

    public setZoomFromScale(scale: number) {
        this.canvasZoom = 1.0 / scale;
    }

    public setCanvasOffset(point: Point) {
        this.canvasOffset = [
            point[0] / this.canvasZoom + this.canvasOffset[0],
            point[1] / this.canvasZoom + this.canvasOffset[1],
        ];
        this.hasChanged = true;
    }

    public setCanvasOffsetUnscaled(point: Point) {
        this.canvasOffset = point;
        this.hasChanged = true;
    }
}

export default MapRenderer;