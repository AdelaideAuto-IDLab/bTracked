# bTracked
_Highly Accurate Field Deployable Real-Time Indoor Spatial Tracking for Human Behavior Observations_

### Abstract

Methods for accurate indoor spatial tracking remains a challenge. Low cost and power efficient Bluetooth Low Energy (BLE) beacon technology's ability to run maintenance-free for many years on a single coin cell battery provides an attractive methodology to realize accurate and low cost indoor spatial tracking. However an easy to deploy and accurate methodology still remains a problem of ongoing research interest.

We propose a *field deployable* tracking system based on BLE beacon signals together with a particle filter based approach for *online* and *real-time* tracking of persons with a body-worn Bluetooth receiver to support fine grain human behavior observations.

First, we develop the concept of *generic sensor models* for generalized indoor environments and build *pluggable* sensor models for re-use in unseen environments during deployment. Second, we exploit *pose* information and *void constraints* in our problem formulation to derive additional information about the person tracked. Third, we build the infrastructure to easily setup and operate our tracking system to support end-users to remotely track ambulating persons in real-time over a web-based interface. Fourth, we assess *five* different tracking methodologies together with *two* approaches for formulating pose information and show that our method of probabilistic multilateration including the modeling of pose leads to the best performance; a mean path estimation error of 23.5 cm in a new complex indoor environment.
