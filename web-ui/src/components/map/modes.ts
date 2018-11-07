import MapManager from './MapManager';
import MapViewer from './MapViewer';

import { drawCircle, drawTriangle } from './canvas';

import { MapData, Point, Rect } from '../../model';
import Api from '../../services/Api';
import { computeRect, distanceSqr } from '../../utils';

export abstract class Mode {
    public leftClick(point: Point): void {
        // Default implementation does nothing
    }

    public rightClick(point: Point): void {
        // Default implementation does nothing
    }

    public mouseMove(point: Point): void {
        // Default implementation does nothing
    }

    public mouseDown(point: Point): void {
        // Default implementation does nothing
    }

    public mouseRelease(point: Point): void {
        // Default implementation does nothing
    }

    public mouseScroll(amount: number, mouse: Point): void {
        // Default implementation does nothing
    }

    public drawUi(ctx: CanvasRenderingContext2D, mouse: Point | null): void {
        // Default implementation does nothing
    }
}

export class NoneMode extends Mode {}

export class EditWallMode extends Mode {
    private selectedPoint: Point | null;

    constructor(protected viewer: MapViewer, private mapManager: MapManager) {
        super();
    }

    public leftClick(point: Point) {
        if (this.selectedPoint == null) {
            this.selectedPoint = point;
        }
        else {
            this.mapManager.addWall([this.selectedPoint, point]);
            this.selectedPoint = null;
            this.viewer.drawMain();
        }
    }

    public rightClick(point: Point) {
        if (this.selectedPoint != null) {
            this.selectedPoint = null;
        }
        else {
            this.mapManager.removeWallNear(point);
            this.viewer.drawMain();
        }
    }

    public drawUi(ctx: CanvasRenderingContext2D, mouse: Point | null) {
        if (mouse == null) {
            return;
        }

        ctx.fillStyle = 'red';
        drawCircle(ctx, mouse);

        if (this.selectedPoint != null) {
            ctx.strokeStyle = 'red';
            ctx.lineWidth = 2;

            ctx.beginPath();
            ctx.moveTo(this.selectedPoint[0], this.selectedPoint[1]);
            ctx.lineTo(mouse[0], mouse[1]);
            ctx.stroke();
        }
    }
}

export class AddBeaconMode extends Mode {
    private selectedPoint: Point | null;

    constructor(protected viewer: MapViewer, private mapManager: MapManager) {
        super();
    }

    public leftClick(point: Point) {
        if (this.selectedPoint == null) {
            this.selectedPoint = point;
        }
        else {
            this.mapManager.addBeacon(this.selectedPoint, this.getAngle(point));
            this.selectedPoint = null;
            this.viewer.drawMain();
        }
    }

    public rightClick(point: Point) {
        if (this.selectedPoint != null) {
            this.selectedPoint = null;
        }
    }

    public drawUi(ctx: CanvasRenderingContext2D, mouse: Point | null) {
        if (mouse == null) {
            return;
        }

        ctx.fillStyle = 'blue'
        if (this.selectedPoint == null) {
            drawCircle(ctx, mouse);
            return;
        }

        const [x, y] = this.selectedPoint;
        const angle = this.getAngle(mouse);
        if (angle == null) {
            drawCircle(ctx, this.selectedPoint);
            ctx.font = '12px sans-serif';
            ctx.textAlign = 'center';
            ctx.fillText('Ceiling beacon', x, y + 22.0);
            ctx.fillText('height = 2.7', x, y + 34.0);
        }
        else {
            drawTriangle(ctx, this.selectedPoint, angle);
            ctx.font = '12px sans-serif';
            ctx.textAlign = 'center';
            ctx.fillText('Wall beacon', x, y + 22.0);
            ctx.fillText('angle = ' + angle, x, y + 34.0);
            ctx.fillText('height = 1.5', x, y + 46.0);
        }
    }

    private getAngle(mouse: Point) {
        if (this.selectedPoint == null) {
            return null;
        }
        if (distanceSqr(this.selectedPoint, mouse) < 15.0 * 15.0) {
            return null;
        }
        return Math.atan2(mouse[1] - this.selectedPoint[1], mouse[0] - this.selectedPoint[0])
    }
}

export class SetScale extends Mode {
    private selectedPoint: Point | null;

    constructor(protected viewer: MapViewer, private mapManager: MapManager) {
        super();
    }

    public leftClick(point: Point) {
        if (this.selectedPoint == null) {
            this.selectedPoint = point;
        }
        else {
            this.mapManager.setScale(this.computeScale(point));
            this.selectedPoint = null;
            this.viewer.drawMain();
        }
    }

    public rightClick(point: Point) {
        this.selectedPoint = null;
    }

    public drawUi(ctx: CanvasRenderingContext2D, mouse: Point | null) {
        if (mouse == null) {
            return;
        }

        ctx.fillStyle = 'black';
        drawCircle(ctx, mouse);

        if (this.selectedPoint != null) {
            ctx.strokeStyle = 'red';
            ctx.lineWidth = 2;

            ctx.beginPath();
            ctx.moveTo(this.selectedPoint[0], this.selectedPoint[1]);
            ctx.lineTo(mouse[0], mouse[1]);
            ctx.stroke();

            ctx.font = '12px sans-serif';
            ctx.fillText(`1m : ${this.computeScale(mouse)} px`, mouse[0], mouse[1] + 12.0);
        }
    }

    private computeScale(point: Point) {
        if (this.selectedPoint == null) {
            return 0;
        }

        return Math.sqrt(distanceSqr(point, this.selectedPoint));
    }
}

abstract class SetRect extends Mode {
    private selectedPoint: Point | null;

    constructor(protected viewer: MapViewer, protected mapManager: MapManager) {
        super();
    }

    public leftClick(point: Point) {
        if (this.selectedPoint == null) {
            this.selectedPoint = point;
        }
        else {
            this.addRect(computeRect(this.selectedPoint, point));
            this.viewer.drawMain();
            this.selectedPoint = null;
        }
    }

    public rightClick(point: Point) {
        this.selectedPoint = null;
    }

    public drawUi(ctx: CanvasRenderingContext2D, mouse: Point | null) {
        if (mouse == null) {
            return;
        }

        if (this.selectedPoint != null) {
            ctx.strokeStyle = 'black';
            ctx.lineWidth = 1;

            const rect = computeRect(this.selectedPoint, mouse);
            ctx.strokeRect(rect.x, rect.y, rect.width, rect.height);
        }
    }

    protected abstract addRect(rect: Rect): any;
}

export class EditObstaclesMode extends SetRect {
    public rightClick(point: Point) {
        super.rightClick(point);
        this.mapManager.removeObstacle(point);
    }

    protected addRect(rect: Rect) {
        this.mapManager.addObstacle(rect);
    }
}

export class SetBoundary extends SetRect {
    protected addRect(rect: Rect) {
        this.mapManager.setBoundary(rect);
    }
}

export class AddZone extends SetRect {
    protected addRect(rect: Rect) {
        this.mapManager.addZone(rect);
    }
}

export class AdjustCanvas extends Mode {
    private dragStart: Point | null = null;
    private dragAmount: Point = [0, 0];

    constructor(protected viewer: MapViewer) {
        super();
    }

    public mouseScroll(amount: number, mouse: Point) {
        this.viewer.renderer.updateZoom(amount, mouse);
        this.viewer.drawAll();
    }

    public mouseDown(point: Point) {
        this.dragStart = point;
    }

    public mouseRelease(point: Point) {
        if (this.dragStart != null) {
            this.updateDrag(point);
            this.dragStart = null;
        }
        this.dragAmount = [0, 0];
    }

    public mouseMove(point: Point) {
        if (this.dragStart != null) {
            this.updateDrag(point);
        }
    }

    private updateDrag(point: Point) {
        if (this.dragStart == null) {
            return;
        }
        this.dragAmount = [point[0] - this.dragStart[0], point[1] - this.dragStart[1]];
        this.viewer.renderer.setCanvasOffset(this.dragAmount);
        this.viewer.drawAll();
    }
}

export class InstanceViewerMode extends AdjustCanvas {
    constructor(protected viewer: MapViewer, private mapData: MapData, private instanceName: string, private simName: string) {
        super(viewer);
    }

    public rightClick(point: Point) {
        const scale = 1.0 / this.mapData.scale;
        const scaledPoint = [
            scale * (point[0] - this.mapData.boundary.x),
            scale * (point[1] - this.mapData.boundary.y)
        ];
        const api = new Api();
        api.post(`instance/${this.instanceName}/sim/${this.simName}/goto`, scaledPoint);
    }
}
