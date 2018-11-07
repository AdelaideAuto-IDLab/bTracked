import { Line, Point, Rect } from '../model';

/// Computes the squared distance between two points
export function distanceSqr(a: Point, b: Point) {
    return (b[0] - a[0]) * (b[0] - a[0]) + (b[1] - a[1]) * (b[1] - a[1]);
}

/// Finds the endpoint of a line that is closest to a point
export function closestEndPoint(point: Point, line: Line) {
    const startDist = distanceSqr(point, line[0]);
    const endDist = distanceSqr(point, line[1]);
    return (startDist < endDist) ?
        { point: line[0], dist: startDist } :
        { point: line[1], dist: endDist };
};

/// Rounds an angle (in degrees) to a factor of a provided interval
export function roundAngleTo(angle: number, interval: number) {
    return Math.round(angle / interval) * interval;
}

export function closestPointOnLine(point: Point, line: Line): Point {
    const lenSqr = distanceSqr(line[0], line[1]);
    const dx = line[1][0] - line[0][0];
    const dy = line[1][1] - line[0][1];

    let t = ((point[0] - line[0][0]) * dx + (point[1] - line[0][1]) * dy) / lenSqr;
    t = Math.max(0.0, Math.min(1.0, t));

    return [line[0][0] + t * dx, line[0][1] + t * dy];
}

/// Generates a rectangle from a starting and ending corner.
export function computeRect(start: Point, end: Point) {
    return {
        x: Math.min(start[0], end[0]),
        y: Math.min(start[1], end[1]),
        width: Math.abs(start[0] - end[0]),
        height: Math.abs(start[1] - end[1]),
    }
}

/// Checks whether a rectangle contains a point
export function containsPoint(rect: Rect, point: Point) {
    return !(
        point[0] < rect.x || point[1] < rect.y ||
            point[0] > rect.x + rect.width || point[1] > rect.y + rect.height
    )
}