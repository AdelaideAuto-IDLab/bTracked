class Api {
    public get<T>(path: string): Promise<T> {
        return fetch(`${this.getEndpoint()}${path}`, {
            method: 'GET',
            headers: this.getHeaders(),
        }).then(parseBody);
    }

    public post<T>(path: string, value: any): Promise<T> {
        return fetch(`${this.getEndpoint()}${path}`, {
            method: 'POST',
            headers: this.getHeaders(),
            body: JSON.stringify(value)
        }).then(parseBody);
    }

    public delete<T>(path: string): Promise<T> {
        return fetch(`${this.getEndpoint()}${path}`, {
            method: 'DELETE',
            headers: this.getHeaders(),
        }).then(parseBody);
    }

    private getEndpoint() {
        return '/api/'
    }

    private getHeaders(): Headers {
        return new Headers({
            'Content-Type': 'application/json'
        });
    }
}

export default Api;


function extractErrorMessage(input: string) {
    try {
        const json = JSON.parse(input);
        if (json.message !== undefined && json.code !== undefined) {
            return `${json.message}\n\n(Code: ${json.code})`;
        }
    }
    catch {
        // Ignore json decoding error and just return the raw string
    }

    return input;
}

async function parseBody(response: Response): Promise<any> {
    const contentType = response.headers.get('content-type');
    const isJson = contentType != null && contentType.indexOf('application/json') !== -1;

    if (response.ok) {
        if (isJson) {
            return response.json();
        }
        return response.text();
    }
    else if (response.status === 404) {
        return null;
    }

    let result = null;
    try {
        result = await response.text();
    }
    catch {
        throw new Error(`${response.statusText} (${response.status})`);
    }

    throw new Error(extractErrorMessage(result));
}