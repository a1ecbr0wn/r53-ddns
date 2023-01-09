mod cli;

use std::net::IpAddr;

use clap::Parser;
use futures::stream::{FuturesUnordered, StreamExt};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Client;
use rusoto_core::{Region, RusotoError};
use rusoto_route53::{
    Change, ChangeBatch, ChangeResourceRecordSetsRequest, ListHostedZonesRequest,
    ListResourceRecordSetsRequest, ResourceRecord, ResourceRecordSet, Route53, Route53Client,
};
use tokio::{join, select};

#[tokio::main]
async fn main() -> Result<(), RusotoError<RusotoError<()>>> {
    lazy_static! {
        static ref DESCRIPTION: String = format!(env!("CARGO_PKG_VERSION"));
    };

    let options = cli::Options::parse();
    let verbose = options.verbose;
    if options.server.is_some() && options.domain.is_some() {
        let host_name = options.server.unwrap();
        let zone_name = options.domain.unwrap();
        if is_valid_hostname(&host_name) {
            let mut host_name = host_name.to_string();
            if !host_name.ends_with('.') {
                host_name += ".";
            }
            let mut zone_name = zone_name.to_string();
            if !zone_name.ends_with('.') {
                zone_name += ".";
            }
            if verbose {
                println!(
                    "server:        {}\ndomain:        {}",
                    host_name.clone(),
                    zone_name.clone()
                );
            }
            let client = Route53Client::new(Region::UsEast1);
            let zone_id = get_zone_id(&client, &zone_name).await;
            let external_ip_future = get_external_ip_address();
            let dns_ip_future = get_ip_record(&client, &zone_id, &zone_name, &host_name);
            let ((external_ip_address, external_address_svc), dns_ip_address) =
                join!(external_ip_future, dns_ip_future);
            if verbose {
                println!("zone id:       {zone_id}");
                if !external_ip_address.is_empty() {
                    println!("external ip:   {external_ip_address}  (from {external_address_svc})");
                }
            }
            if let Some(dns_ip_address) = dns_ip_address {
                if verbose {
                    println!("dns ip:        {dns_ip_address}");
                }
                if dns_ip_address != external_ip_address
                    && !dns_ip_address.is_empty()
                    && !external_ip_address.is_empty()
                {
                    println!("{host_name}{zone_name} ip address has changed from {dns_ip_address} to {external_ip_address}");
                    set_resource_record(
                        &client,
                        zone_id,
                        zone_name,
                        host_name,
                        "A".to_string(),
                        external_ip_address,
                    )
                    .await;
                }
            }
        } else if verbose {
            println!("invalid server value: {host_name}\n");
        }
    } else if options.server.is_some() || options.domain.is_some() {
        println!("r53-ddns v{}", DESCRIPTION.as_str());
        println!("\n  server and domain parameters need to be supplied together");
        return Ok(());
    }

    if options.version {
        println!("r53-ddns v{}", DESCRIPTION.as_str());
        return Ok(());
    }

    Ok(())
}

/// This function checks to see whether the host_name entered into the zone and cname tags is a valid host_name.
fn is_valid_hostname(host_name: &str) -> bool {
    if host_name.len() > 255 {
        return false;
    }
    let re = Regex::new(r"^[a-zA-Z\d-]{1,63}$").unwrap();
    for s in host_name.split('.') {
        if !re.is_match(s) && !s.is_empty() {
            return false;
        }
    }
    true
}

/// Returns the zone id for a given zone name
async fn get_zone_id(client: &Route53Client, zone_name: &str) -> String {
    let mut next_marker: Option<String> = None;
    loop {
        // Create a request to list all hosted zones
        let request = ListHostedZonesRequest {
            marker: next_marker,
            ..Default::default()
        };
        // Send the request and print the response
        if let Ok(response) = client.list_hosted_zones(request).await {
            for zone in response.hosted_zones {
                if zone.name == zone_name {
                    let zone_id = zone.id.rsplit_once('/').unwrap().1.to_string();
                    return zone_id;
                }
            }
            if response.is_truncated {
                next_marker = response.next_marker;
            } else {
                return "".to_string();
            }
        } else {
            return "".to_string();
        }
    }
}

/// Get the external ip address from an external service
async fn get_external_ip_address() -> (String, String) {
    let mut futures = FuturesUnordered::new();
    let addresses = [
        "ident.me",
        "ifconfig.me/ip",
        "icanhazip.com",
        "myexternalip.com/raw",
        "ipecho.net/plain",
        "checkip.amazonaws.com",
        "whatismyip.akamai.com",
        "myip.dnsomatic.com/",
        "diagnostic.opendns.com/myip",
    ]
    .iter()
    .map(|x| get_http_resp(x));
    futures.extend(addresses);
    for _ in 1..futures.len() {
        select! {
            response = futures.select_next_some() => {
                if let Ok(response) = response {
                    return response;
                }
            }
        }
    }
    ("".to_string(), "".to_string())
}

async fn get_http_resp(address: &str) -> Result<(String, String), ()> {
    let client = Client::new();
    if let Ok(resp) = client.get(format!("https://{address}")).send().await {
        if let Ok(response) = resp.text().await {
            let response = response.trim().to_string();
            if response.parse::<IpAddr>().is_ok() {
                return Ok((response, address.to_string()));
            }
        }
    }
    Err(())
}

/// Lists the ip address of a given zone/host A record
async fn get_ip_record(
    client: &Route53Client,
    zone_id: &str,
    zone_name: &str,
    host_name: &str,
) -> Option<String> {
    let record_name = format!("{}{}", host_name, zone_name);
    let request = ListResourceRecordSetsRequest {
        hosted_zone_id: zone_id.to_string(),
        start_record_name: Some(record_name.clone()),
        ..Default::default()
    };
    if let Ok(response) = client.list_resource_record_sets(request).await {
        let record_sets = response.resource_record_sets;
        for record_set in record_sets {
            if record_set.name == record_name {
                if let Some(records) = &record_set.resource_records {
                    if let Some(record) = records.first() {
                        let ip_record = record.value.clone();
                        if ip_record.parse::<IpAddr>().is_ok() {
                            return Some(ip_record);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Creates resource records in the given hosted zone
async fn set_resource_record(
    client: &Route53Client,
    zone_id: String,
    zone_name: String,
    host_name: String,
    record_type: String,
    record_value: String,
) {
    // Build the request to change the resource record sets
    let request = ChangeResourceRecordSetsRequest {
        hosted_zone_id: zone_id,
        change_batch: ChangeBatch {
            changes: vec![Change {
                action: String::from("UPSERT"),
                resource_record_set: ResourceRecordSet {
                    name: format!("{host_name}{zone_name}"),
                    resource_records: Some(vec![ResourceRecord {
                        value: record_value,
                    }]),
                    ttl: Some(300),
                    type_: record_type,
                    ..Default::default()
                },
            }],
            comment: Some(String::from("Updated by r53-ddns")),
        },
    };

    // Call the Route53 API to change the resource record sets
    let _response = client.change_resource_record_sets(request).await;
}
