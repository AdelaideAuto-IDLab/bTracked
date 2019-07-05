use std::{error::Error, fs, thread, time::Duration};

use crossbeam_channel as channel;
use reqwest;

use {BeaconPacket, config, dest::Destination};

pub struct HttpDestination {
    measurement_tx: channel::Sender<BeaconPacket>,
}

impl HttpDestination {
    pub fn new(
        config: config::HttpConfig,
        retry_attempts: u64,
        queue_rate_ms: u64,
    ) -> Result<HttpDestination, Box<dyn Error>> {
        let (measurement_tx, measurement_rx) = channel::bounded(255);
        let sender = HttpSender::new(config, retry_attempts, measurement_rx)?;

        thread::spawn(move || sender.run(queue_rate_ms));

        Ok(HttpDestination { measurement_tx })
    }
}

impl Destination for HttpDestination {
    fn new_measurement(&mut self, measurement: &BeaconPacket) -> Result<(), Box<dyn Error>> {
        self.measurement_tx.send(measurement.clone())?;
        Ok(())
    }
}


struct HttpSender {
    client: reqwest::Client,
    endpoint: String,
    retry_attempts: u64,
    measurement_rx: channel::Receiver<BeaconPacket>,
    measurement_buffer: Vec<BeaconPacket>,
}

impl HttpSender {
    pub fn new(
        config: config::HttpConfig,
        retry_attempts: u64,
        measurement_rx: channel::Receiver<BeaconPacket>
    ) -> Result<HttpSender, Box<dyn Error>> {
        let mut builder = reqwest::ClientBuilder::new();

        if let Some(proxy) = config.http_proxy {
            builder = builder.proxy(reqwest::Proxy::http(&proxy)?);
        }
        if let Some(proxy) = config.https_proxy {
            builder = builder.proxy(reqwest::Proxy::https(&proxy)?);
        }
        if let Some(cert_path) = config.identity_cert {
            let pass = config.identity_cert_pass
                .ok_or_else(|| "Password for certificate not provided")?;
            let buf = fs::read(cert_path)?;
            let cert = reqwest::Identity::from_pkcs12_der(&buf, &pass)?;
            builder = builder.identity(cert);
        }

        for cert_path in config.root_certs {
            let buf = fs::read(&cert_path)?;
            let cert = match cert_path.extension() {
                Some(x) if x == "der" || x == "DER" => reqwest::Certificate::from_der(&buf)?,
                _ => reqwest::Certificate::from_pem(&buf)?,
            };
            builder = builder.add_root_certificate(cert);
        }

        Ok(HttpSender {
            client: builder.build()?,
            endpoint: config.endpoint,
            retry_attempts,
            measurement_rx,
            measurement_buffer: vec![],
        })
    }

    fn run(mut self, queue_rate_ms: u64) {
        let tick = channel::tick(Duration::from_millis(queue_rate_ms));
        loop {
            select! {
                recv(self.measurement_rx) -> msg => match msg {
                    Ok(m) => self.measurement_buffer.push(m),
                    Err(_) => break,
                },

                recv(tick) -> _ => {
                    if let Err(e) = self.try_send() {
                        error!("Error sending measurement: {}", e);
                    }
                },
            }
        }
    }

    fn try_send(&mut self) -> Result<(), Box<dyn Error>> {
        if self.measurement_buffer.is_empty() {
            return Ok(());
        }

        debug!("Trying to send: {} measurements", self.measurement_buffer.len());

        let mut result: Result<bool, reqwest::Error> = Ok(true);

        for _ in 0..(self.retry_attempts + 1) {
            result = self.client.post(&self.endpoint)
                .json(&self.measurement_buffer)
                .send()
                .and_then(|mut r| r.json());
            if let Err(ref e) = result {
                warn!("Error sending measurement: {}", e);
            }
        }

        self.measurement_buffer.clear();
        let _ = result?;

        Ok(())
    }
}