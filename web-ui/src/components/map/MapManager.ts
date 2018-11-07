import { Line, MapData, Point, Rect, SignalSource } from '../../model';

import { closestEndPoint, closestPointOnLine, containsPoint, distanceSqr, roundAngleTo, swapRemoveIndex} from '../../utils';

export default class MapManager {
    public snapAngle = 10;
    public enableSnapPoints = true;

    public nextFreeBeaconId = 1;
    public nextFreeZoneId = 1;

    public mapData: MapData = {
        version: "2",
        boundary: { x: 0, y: 0, width: 600, height: 600 },
        scale: 100.0,
        walls: [],
        obstacles: [],
        signal_sources: {},
        zones: {},
    };

    constructor(private onBeaconsChanged?: (beacons: { [id: string]: SignalSource }) => void) { }

    public restore(data: MapData) {
        this.mapData = data;
        if (this.onBeaconsChanged != null) {
            this.onBeaconsChanged(this.mapData.signal_sources);
        }
    }

    public setScale(scale: number) {
        this.mapData.scale = scale;
    }

    public setBoundary(boundary: Rect) {
        this.mapData.boundary = boundary;
    }

    public removeBeacon(id: string) {
        delete this.mapData.signal_sources[id];
        if (this.onBeaconsChanged != null) {
            this.onBeaconsChanged(this.mapData.signal_sources);
        }
    }

    public addBeacon(beacon: Point, angle: number | null) {
        while (this.mapData.signal_sources[`${this.nextFreeBeaconId}`] != null) {
            this.nextFreeBeaconId += 1;
        }

        this.mapData.signal_sources[`${this.nextFreeBeaconId}`] = {
            position: [beacon[0], beacon[1], 2.7],
            direction: angle != null ?
                [Math.cos(angle), Math.sin(angle), 0.0] :
                [0, 0, -1.0],
            model_id: 0,
        };
        if (this.onBeaconsChanged != null) {
            this.onBeaconsChanged(this.mapData.signal_sources);
        }
    }

    public removeObstacle(point: Point): any {
        const obstacles = this.mapData.obstacles;
        for (let i = 0; i < obstacles.length; ++i) {
            if (containsPoint(obstacles[i], point)) {
                swapRemoveIndex(obstacles, i);
                return;
            }
        }
    }

    public addObstacle(rect: Rect) {
        this.mapData.obstacles.push(rect);
    }

    public addZone(rect: Rect) {
        while (this.mapData.zones[`${this.nextFreeZoneId}`] != null) {
            this.nextFreeZoneId += 1;
        }

        this.mapData.zones[`${this.nextFreeZoneId}`] = [
            [rect.x, rect.y],
            [rect.x + rect.width, rect.y],
            [rect.x + rect.width, rect.y + rect.height],
            [rect.x, rect.y + rect.height],
        ];
    }

    public addWall(wall: Line) {
        const initialWall = wall;
        if (this.enableSnapPoints) {
            wall = this.snapWallPoints(wall);
        }
        if (this.snapAngle > 0 && initialWall[1][0] === wall[1][0]) {
            wall = this.snapWallAngle(wall, this.snapAngle);
        }
        this.mapData.walls.push(wall);
    }

    public removeWallNear(point: Point): any {
        const walls = this.mapData.walls;
        for (let i = 0; i < walls.length; ++i) {
            const distanceToWall = distanceSqr(closestPointOnLine(point, walls[i]), point);
            if (distanceToWall < 5 * 5) {
                swapRemoveIndex(walls, i);
                return;
            }
        }
    }

    private snapWallPoints(wall: Line): Line {
        let start = wall[0];
        let startDist = Math.pow(10.0, 2);
        let end = wall[1];
        let endDist = Math.pow(10.0, 2);

        for (const otherWall of this.mapData.walls) {
            let point = closestEndPoint(wall[0], otherWall);
            if (point.dist < startDist) {
                start = point.point;
                startDist = point.dist;
            }

            point = closestEndPoint(wall[1], otherWall);
            if (point.dist < endDist) {
                end = point.point;
                endDist = point.dist;
            }
        }

        return [start, end];
    }

    private snapWallAngle(wall: Line, snapAngle: number): Line {
        const dx = wall[1][0] - wall[0][0];
        const dy = wall[1][1] - wall[0][1];

        let angle = Math.atan2(dy, dx);
        angle = roundAngleTo(angle * 180 / Math.PI, snapAngle) / 180 * Math.PI;

        const len = Math.sqrt(dx * dx + dy * dy)

        const end: Point = [
            wall[0][0] + len * Math.cos(angle),
            wall[0][1] + len * Math.sin(angle)
        ];
        return [wall[0], end];
    }
}