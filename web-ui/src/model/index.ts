export * from './target';

export type Point = number[];
export type Vec2 = [number, number];
export type Vec3 = [number, number, number];
export type Line = [Point, Point];

export interface Rect {
    x: number;
    y: number;
    width: number;
    height: number;
}

export interface ParticleState {
    position: Vec2;
    velocity: Vec2;
    turn_rate: number;
    pose: number;
    mode: string;
}

export interface ParticleSnapshot {
    stationary: number[];
    moving: number[];
}

export interface UpdateMessage {
    tracking?: {
        snapshot: ParticleSnapshot;
        estimate: ParticleState;
    };
    sim_state?: ParticleState;
}

export interface MapData {
    version: string;
    boundary: { x: number, y: number, width: number, height: number };
    scale: number;
    walls: Line[];
    obstacles: Rect[];
    signal_sources: { [id: string]: SignalSource };
    zones: { [id: string]: Polygon };
}

export interface SignalSource {
    position: Vec3;
    direction: Vec3;
    model_id: number;
}

export type Polygon = number[][];

export interface MapMetadata {
    map_key: string;
    description: string;
}

export interface ConfigMetadata {
    key: string;
    type: string;
    description: string;
}

export interface ModelConfig {
    kinematic_noise: number;
    turn_rate_noise: number;
    pose_noise: number;
    transition_prop: number;
}

export interface SignalConfig {
    alpha: number;
    beta: number;
    noise: number;
    gain_table: { horizontal: number[], vertical: number [] };
}

export interface FilterConfig {
    num_particles: number;
    update_rate_ms: number;
    stationary: ModelConfig;
    motion: ModelConfig;
    speed: number;
    turn_rate_mean: number;
    turn_rate_stddev: number;
    signal: SignalConfig[];
}

export interface InstanceMetadata {
    name: string;
    map_key: string;
    attached_simulations: string[];
}

export interface InstanceConfig {
    name: string;
    map_key: string;
    tracking: {
        geometry: MapData;
        filter: any;
    };
    beacon_mapping: { [name: string]: string };
}

export interface InstanceDetails {
    config: InstanceConfig;
    attached_simulations: string[];
}