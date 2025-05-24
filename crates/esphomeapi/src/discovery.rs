use std::{
  collections::{HashMap, HashSet},
  net::IpAddr,
  sync::{Arc, RwLock},
  time::Duration,
};

use mdns_sd::{ServiceDaemon, ServiceEvent};

const SERVICE_NAME: &str = "_esphomelib._tcp.local.";

#[derive(Clone, Debug)]
pub struct ServiceInfo {
  pub ty_domain: String, // <service>.<domain>

  /// See RFC6763 section 7.1 about "Subtypes":
  /// <https://datatracker.ietf.org/doc/html/rfc6763#section-7.1>
  pub sub_domain: Option<String>, // <subservice>._sub.<service>.<domain>

  pub fullname: String, // <instance>.<service>.<domain>
  pub server: String,   // fully qualified name for service host
  pub addresses: HashSet<IpAddr>,
  pub port: u16,
  pub host_ttl: u32,  // used for SRV and Address records
  pub other_ttl: u32, // used for PTR and TXT records
  pub priority: u16,
  pub weight: u16,
}

pub async fn discover(seconds: u32) -> Vec<ServiceInfo> {
  let mdns = ServiceDaemon::new().unwrap();
  let receiver = mdns.browse(SERVICE_NAME).expect("Failed to browse");

  let found_services = Arc::new(RwLock::new(HashMap::new()));
  let found_services_clone = found_services.clone();

  tokio::select! {
    _ = tokio::time::sleep(Duration::from_secs(seconds as u64)) => mdns.shutdown().unwrap(),
    _ = async move {
      loop {
        match receiver.recv_async().await {
          Ok(ServiceEvent::ServiceResolved(info)) => {
            let mut write_guard = found_services_clone.write().unwrap();
                    write_guard.insert(
                      info.get_fullname().to_owned(),
                      ServiceInfo {
                        ty_domain: info.get_type().to_owned(),
                        sub_domain: info.get_subtype().to_owned(),
                        fullname: info.get_fullname().to_owned(),
                        server: info.get_hostname().to_owned(),
                        addresses: info.get_addresses().clone(),
                        port: info.get_port(),
                        host_ttl: info.get_host_ttl(),
                        other_ttl: info.get_other_ttl(),
                        priority: info.get_priority(),
                        weight: info.get_weight(),
                      },
                    );
          }
          _ => { }
        }
      }
    } => mdns.shutdown().unwrap(),
  };

  let services = found_services.read().unwrap();
  services.values().cloned().collect()
}
