import { ParticleState, Vec2 } from '.';

export class Target {
    public trail: Vec2[] = [];

    private state: ParticleState = {
        position: [0.0, 0.0],
        velocity: [0.0, 0.0],
        turn_rate: 0.0,
        pose: 0.0,
        mode: 'Stationary'
    };
    private lastUpdate: number = 0;
    private currentTime: number = 0;

    private wasUpdated: boolean = true;

    constructor(private offset: Vec2, private scale: number) {}

    public update(state: ParticleState) {
        this.state = state;
        this.wasUpdated = true;

        this.trail.push(this.getLocation());
        if (this.trail.length > 100) {
            this.trail.shift();
        }
    }

    public tick(time: number) {
        if (this.wasUpdated) {
            this.lastUpdate = time;
            this.wasUpdated = false;
        }
        this.currentTime = time;
    }

    public getLocation(): Vec2 {
        let dt = this.wasUpdated ? 0 : (this.currentTime - this.lastUpdate) / 1000.0;
        if (this.state.mode === 'Stationary') {
            dt = 0.0;
        }
        return [
            this.scale * (this.state.position[0] + dt * this.state.velocity[0]) + this.offset[0],
            this.scale * (this.state.position[1] + dt * this.state.velocity[1]) + this.offset[1],
        ];
    }

    public getPose(): number {
        return this.state.pose;
    }
}