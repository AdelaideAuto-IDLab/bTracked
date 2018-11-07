import { Line, Point, Polygon, Rect, SignalSource } from '../../model';

export function getMousePositionOnCanvas(
    canvas: HTMLCanvasElement | null,
    event: React.MouseEvent<HTMLCanvasElement>
): Point {
    if (canvas == null) {
        return [0, 0];
    }

    const boundingRect = canvas.getBoundingClientRect();
    return [
        event.clientX - boundingRect.left,
        event.clientY - boundingRect.top,
    ];
}

export function drawBeacons(
    ctx: CanvasRenderingContext2D,
    circles: { [id: string]: SignalSource },
    color: string
) {
    ctx.save();

    ctx.fillStyle = color;
    for (const key of Object.keys(circles)) {
        const circle = circles[key];
        if (circle.direction[2] !== 0) {
            drawCircle(ctx, circle.position);
        }
        else {
            const angle = Math.atan2(circle.direction[1], circle.direction[0]);
            drawTriangle(ctx, circle.position, angle);
        }
    }

    ctx.fillStyle = 'black';
    ctx.textAlign = 'center';
    for (const key of Object.keys(circles)) {
        const circle = circles[key];

        const x = circle.position[0];
        const y = circle.position[1] + 10.0;

        ctx.font = '12px sans-serif';
        ctx.fillText(key, x, y + 12.0);
    }

    ctx.restore();
}

export function drawWalls(ctx: CanvasRenderingContext2D, walls: Line[], color: string) {
    ctx.save();

    ctx.strokeStyle = color;
    ctx.lineWidth = 4;
    for (const wall of walls) {
        ctx.beginPath();
        ctx.moveTo(wall[0][0], wall[0][1]);
        ctx.lineTo(wall[1][0], wall[1][1]);
        ctx.stroke();
    }

    ctx.restore();
}

export function drawRects(ctx: CanvasRenderingContext2D, rects: Rect[], color: string) {
    ctx.save();

    ctx.fillStyle = color;
    ctx.strokeStyle = 'black';
    for (const rect of rects) {
        ctx.fillRect(rect.x, rect.y, rect.width, rect.height);
        ctx.strokeRect(rect.x, rect.y, rect.width, rect.height);
    }

    ctx.restore();
}

export function drawCircle(ctx: CanvasRenderingContext2D, circle: Point, radius?: number) {
    ctx.beginPath();
    ctx.arc(circle[0], circle[1], radius != null ? radius : 5.0, 0, 2 * Math.PI);
    ctx.fill();
}

export function drawTriangle(ctx: CanvasRenderingContext2D, center: Point, angle: number) {
    const cosA = Math.cos(angle - Math.PI / 2);
    const sinA = Math.sin(angle - Math.PI / 2);
    const [x, y] = center;
    const [sx, sy] = [5.0, 10.0]

    ctx.beginPath();
    ctx.moveTo(x + -sx*cosA + sy*sinA, y + -sx*sinA - sy*cosA);
    ctx.lineTo(x + -sy*sinA, y + sy*cosA);
    ctx.lineTo(x + sx*cosA + sy*sinA, y + sx*sinA - sy*cosA);
    ctx.fill();
}

export function drawLine(ctx: CanvasRenderingContext2D, line: Point[]) {
    ctx.beginPath();
    for (const point of line) {
        ctx.lineTo(point[0], point[1]);
    }
    ctx.stroke();
}

export function drawPolygon(ctx: CanvasRenderingContext2D, polygon: Polygon, color: string) {
    ctx.save();

    ctx.fillStyle = color;
    ctx.strokeStyle = 'black';

    ctx.beginPath();
    for (const point of polygon) {
        ctx.lineTo(point[0], point[1]);
    }
    ctx.closePath();

    ctx.fill();
    ctx.stroke();

    ctx.restore();
}