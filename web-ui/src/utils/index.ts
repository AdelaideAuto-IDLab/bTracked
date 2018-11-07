export * from './geometry';
export * from './versionUpdater';

export function orDefault<T>(value: T | (null | undefined), defaultValue: T): T {
    if (value == null) {
        return defaultValue;
    }
    return value;
}

/// Removes an element in an array by swapping it with the last element and performing a pop
export function swapRemove<T>(array: T[], item: T) {
    let index = -1;
    for (let i = 0; i < array.length; ++i) {
        if (Object.is(array[i], item)) {
            index = i
            break;
        }
    }

    if (index !== -1) {
        swapRemoveIndex(array, index);
    }
}

/// Removes an element specified by an index in an array by swapping it with the last element and
/// performing a pop
export function swapRemoveIndex<T>(array: T[], index: number) {
    array[index] = array[array.length - 1];
    array.pop();
}

export function getCtxAndClearCanvas(canvas: HTMLCanvasElement | null) {
    if (canvas == null) {
        return null;
    }
    const ctx = canvas.getContext('2d');
    if (ctx == null) {
        return null;
    }
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    return ctx;
}