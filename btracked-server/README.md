# BTracked Server

Responsible for hosting the web-ui, storing map data and configuration, and managing active instances.

## Build guide

1. Install [Rust](https://www.rust-lang.org/en-US/install.html)

2. Build the WebApp:

    1. Navigate to the `web-ui` directory.
    2. Run `(yarn|npm) install`
    3. Run `(yarn|npm) dist`
    4. Copy the content of `web-ui/build` to `btracked-server/

3. Create the database:

    1. Install the database migration tool (diesel)**:

        ```
        cargo install diesel_cli --no-default-features --features sqlite
        ```

    2. Crate the database: `diesel setup`

4. Run the server `cargo run --release`.


** Note: in order to install `sqlite` library is installed and visible to `cargo`:

_Windows (MSVC)_:
* Download appropriate version of sqlite from: https://www.sqlite.org/download.html.

* Generate a lib file from `sqlite3.def` and `sqlite3.dll` using the Native Tools Command Prompt for Visual Studio:

    ```
    lib /def:sqlite3.def /machine:X64 /out:sqlite3.lib
    ```

* Set the `SQLITE3_LIB_DIR` environment variable to the directory that contains the newly created `sqlite3.lib` file.

_Linux_:

* Install the `libsqlite3-dev` package.

## Integration

The server offers an integration endpoint via the WebSocket protocol at `/ws/listener`. In order to receive data from the endpoint you must send a configuration to the endpoint.

The configuration consists of key/value pairs where the key corresponds to the name of the output message, and the value corresponds to a listener config:

```
{
    "output-1": <ListenerConfig>,
    "output-2": <ListenerConfig>,
    ...
}
```

Currently 3 listener configs are currently available:

```json
"TrackingListener": { "instance_name": "...", "num_particles": 0 }
```

```json
"SimListener": { "instance_name": "...", "sim_name": "...", "update_rate": 200 }
```

```json
"MeasurementListener": { "instance_name": "..." }
```