import { MapData } from '../model';

export function updateMap(data: MapData) {
    if (data.version == null) {
        data.version = '0';
    }
    if (data.version === '0') {
        updateVersion0(data);
    }
    if (data.version === '1') {
        updateVersion1(data);
    }
}

function updateVersion0(data: MapData) {
    if (data.signal_sources == null) {
        data.signal_sources = {};
    }
    if (data.obstacles == null) {
        data.obstacles = [];
    }
    for (const key of Object.keys(data.signal_sources)) {
        const source = data.signal_sources[key];
        if (source.position == null) {
            const oldSource = source as any;
            data.signal_sources[key] = {
                position: [oldSource.x, oldSource.y, 2.7],
                direction: [0.0, 0.0, -1.0],
                model_id: 0,
            };
        }
    }
    data.version = "1";
}

function updateVersion1(data: MapData) {
    if (data.zones == null) {
        data.zones = {};
    }
    data.version = "2";
}
