use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::task;
use trust_dns_resolver::config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts};
use trust_dns_resolver::Resolver;

pub async fn check_dns_propagation(
    domain: &str,
    expected_ip: Ipv4Addr,
) -> Result<bool, Box<dyn Error>> {
    // Define a list of DNS servers to check against.
    // These are the IP addresses of well-known DNS servers.
    let dns_servers = [
        Ipv4Addr::new(8, 8, 8, 8), // Google
                                   // Ipv4Addr::new(1, 1, 1, 1), // Cloudflare
                                   // Ipv4Addr::new(9, 9, 9, 9), // Quad9
                                   // Add more if needed
    ];

    for server in dns_servers {
        let server_config = NameServerConfig {
            socket_addr: SocketAddr::new(IpAddr::V4(server), 53),
            protocol: Protocol::Udp, // or Protocol::Tcp
            tls_dns_name: None,
            trust_nx_responses: true,
        };

        let config = ResolverConfig::from_parts(None, vec![], vec![server_config]);
        let resolver = Resolver::new(config, ResolverOpts::default())?;

        // Perform the DNS query
        let domain_clone = domain.to_string();
        let response = task::spawn_blocking(move || resolver.lookup_ip(domain_clone)).await??;

        // Check if any of the IPs in the response match the expected IP
        if response
            .iter()
            .any(|ip_addr| ip_addr == IpAddr::V4(expected_ip))
        {
            println!("DNS record matches on server {}", server);
        } else {
            println!("DNS record does not match on server {}", server);
            return Ok(false);
        }
    }

    Ok(true)
}
