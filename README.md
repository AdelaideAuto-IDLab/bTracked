# bTracked
_Highly Accurate Field Deployable Real-Time Indoor Spatial Tracking for Human Behavior Observations_

### Abstract

Methods for accurate indoor spatial tracking remains a challenge. Low cost and power efficient Bluetooth Low Energy (BLE) beacon technology's ability to run maintenance-free for many years on a single coin cell battery provides an attractive methodology to realize accurate and low cost indoor spatial tracking. However an easy to deploy and accurate methodology still remains a problem of ongoing research interest.

We propose a *field deployable* tracking system based on BLE beacon signals together with a particle filter based approach for *online* and *real-time* tracking of persons with a body-worn Bluetooth receiver to support fine grain human behavior observations.

First, we develop the concept of *generic sensor models* for generalized indoor environments and build *pluggable* sensor models for re-use in unseen environments during deployment. Second, we exploit *pose* information and *void constraints* in our problem formulation to derive additional information about the person tracked. Third, we build the infrastructure to easily setup and operate our tracking system to support end-users to remotely track ambulating persons in real-time over a web-based interface. Fourth, we assess *five* different tracking methodologies together with *two* approaches for formulating pose information and show that our method of probabilistic multilateration including the modeling of pose leads to the best performance; a mean path estimation error of 23.5 cm in a new complex indoor environment.

## Quick start

### Beacon setup

1. Build and configure the `base_station` software, (see: [base_station](./base_station) for more details).
2. Deploy base-stations.
3. Configure and deploy beacons.

### Server setup

1. Build the `btracked-server`, initialize the configuration database and start the server (see: [btracked-server](./btracked-server) for more details).
2. Open the WebApp (runs on `http://localhost:8080` by default).
3. Create a new map using the map editor tool.
4. Create an appropriate filter config (an sample config is available in: `tracking_manager/filter_config.json`)
5. Start a new instance via the instance page in the WebApp.

## Components

* `base_station` -- Responsible for sniffing BLE packets sent by the transmitter back to the server. Currently only the nRF51822 SoC is supported.

* `btracked-server` -- Responsible for hosting the web-ui, storing map data and configuration, and managing active instances.

See the respective subdirectories of each component for more details.

## Auxiliary code

* `tracking` -- Used by the `btracked-server` component. Implements the tracking algorithm and signal models.

* `web-ui` -- Source code for the instance viewer and map editor web app hosted by the `btracked-server`.

## Reference

This repository is provided as part of the following paper:

M. Chesser, L. Chea, H. V. Nguyen, and D. C. Ranasinghe. 2018. "bTracked: Highly Accurate Field Deployable Real-Time Indoor Spatial Tracking for Human Behavior Observations". In *Proceedings of 15th EAI International Conference on Mobile and Ubiquitous Systems: Computing, Networking and Services.*

Cite using:

```
@INPROCEEDINGS{bTracked2018,
    author={M. Chesser and L. Chea and H. V. Nguyen and D. C. Ranasinghe},
    booktitle={Proceedings of 15th EAI International Conference on Mobile and Ubiquitous Systems: Computing, Networking and Services},
    title={bTracked: Highly Accurate Field Deployable Real-Time Indoor Spatial Tracking for Human Behavior Observations},
    year={2018}
}
```

## License

This project is licensed under the MIT License.

See [LICENSE](./LICENSE) for details.
