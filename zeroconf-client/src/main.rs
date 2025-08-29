use std::collections::HashSet;
use std::time::Duration;
use std::thread;

fn main() {
    let service_type = "_myapp._tcp.local";
    println!("Looking for services of type '{}'", service_type);




    let discovery = ServiceDiscovery::browse(service_type).expect("Failed to browse services");
    let mut seen = HashSet::new();

    for event in discovery {
        match event {
            libmdns::ServiceDiscoveryEvent::ServiceResolved(info) => {
                let addr = info
                    .get_addresses()
                    .into_iter()
                    .find(|ip| ip.is_ipv4())
                    .unwrap_or_else(|| "unknown".parse().unwrap());

                if seen.insert(info.get_fullname().to_string()) {
                    println!(
                        "Discovered: {} at {}:{} with TXT: {:?}",
                        info.get_fullname(),
                        addr,
                        info.get_port(),
                        info.get_properties()
                    );
                }
            }
            _ => {}
        }

        // Wait a bit before checking again
        thread::sleep(Duration::from_millis(100));
    }
}
