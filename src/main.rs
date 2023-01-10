mod cli;

use std::net::IpAddr;

use clap::Parser;
use futures::stream::{FuturesUnordered, StreamExt};
use lazy_static::lazy_static;
use log::{error, info, warn, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::filter::threshold::ThresholdFilter;
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
    // logging
    let verbose = options.verbose;
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{m}{n}")))
        .build();
    let rolling_policy = CompoundPolicy::new(
        Box::new(SizeTrigger::new(1024 * 1024 * 4)), // 4mb
        Box::new(
            FixedWindowRoller::builder()
                .build("/var/tmp/r53-ddns.{}.log", 10)
                .unwrap(),
        ),
    );
    let to_file = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build("/var/tmp/r53-ddns.log", Box::new(rolling_policy))
        .unwrap();
    let console_level = if verbose {
        LevelFilter::Info
    } else {
        LevelFilter::Warn
    };
    if pshell::find().is_some() {
        let config = Config::builder()
            .appender(
                Appender::builder()
                    .filter(Box::new(ThresholdFilter::new(console_level)))
                    .build("stdout", Box::new(stdout)),
            )
            .appender(Appender::builder().build("to_file", Box::new(to_file)))
            .build(
                Root::builder()
                    .appender("stdout")
                    .appender("to_file")
                    .build(LevelFilter::Info),
            )
            .unwrap();
        let _handle = log4rs::init_config(config).unwrap();
    }

    let nat = options.nat;
    if options.server.is_some() && options.domain.is_some() {
        let mut host_name = options.server.unwrap();
        let mut zone_name = options.domain.unwrap();
        if is_valid_hostname(&host_name) {
            if !host_name.ends_with('.') {
                host_name += ".";
            }
            if !zone_name.ends_with('.') {
                zone_name += ".";
            }
            let dns_name = format!("{host_name}{zone_name}");
            info!("server:        {}", host_name.clone());
            info!("domain:        {}", zone_name.clone());
            let client = Route53Client::new(Region::UsEast1);
            let zone_id = get_zone_id(&client, &zone_name).await;
            let external_ip_future = get_external_ip_address();
            let dns_ip_future = get_dns_record(&client, &zone_id, &zone_name, &host_name, "A");
            let ((external_ip_address, external_address_svc), dns_ip_address) =
                join!(external_ip_future, dns_ip_future);
            info!("zone id:       {zone_id}");
            if !external_ip_address.is_empty() {
                info!("external ip:   {external_ip_address}  (from {external_address_svc})");
            } else {
                warn!("external ip:   not found");
            }
            if let Some(dns_ip_address) = dns_ip_address {
                info!("dns ip:        {dns_ip_address}");
                if dns_ip_address != external_ip_address
                    && !dns_ip_address.is_empty()
                    && !external_ip_address.is_empty()
                {
                    warn!("{dns_name} ip address has changed from {dns_ip_address} to {external_ip_address}");
                    set_dns_record(
                        &client,
                        &zone_id,
                        &zone_name,
                        &host_name,
                        "A",
                        &external_ip_address,
                    )
                    .await;
                }
            } else if !external_ip_address.is_empty() {
                warn!("{dns_name} ip address is {external_ip_address}");
                set_dns_record(
                    &client,
                    &zone_id,
                    &zone_name,
                    &host_name,
                    "A",
                    &external_ip_address,
                )
                .await;
            }
            if nat {
                let nat_host_name = "\\052.".to_string() + &host_name;
                let dns_nat_cname =
                    get_dns_record(&client, &zone_id, &zone_name, &nat_host_name, "CNAME").await;
                if dns_nat_cname.is_none()
                    || (dns_nat_cname.is_some() && dns_nat_cname.clone().unwrap() != dns_name)
                {
                    info!("{nat_host_name}{zone_name} nat CNAME set to {dns_name}");
                    set_dns_record(
                        &client,
                        &zone_id,
                        &zone_name,
                        &nat_host_name,
                        "CNAME",
                        dns_name.as_str(),
                    )
                    .await
                }
            }
        } else {
            info!("invalid server value: {host_name}\n");
        }
    } else if options.server.is_some() || options.domain.is_some() {
        info!("r53-ddns v{}\n", DESCRIPTION.as_str());
        error!("server and domain parameters need to be supplied together");
        return Ok(());
    }

    if options.version {
        info!("r53-ddns v{}", DESCRIPTION.as_str());
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
        "myip.dnsomatic.com",
        "diagnostic.opendns.com/myip",
    ]
    .iter()
    .map(|x| get_http_resp(x));
    futures.extend(addresses);
    for _ in 0..futures.len() {
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
async fn get_dns_record(
    client: &Route53Client,
    zone_id: &str,
    zone_name: &str,
    host_name: &str,
    record_type: &str,
) -> Option<String> {
    let dns_name = format!("{host_name}{zone_name}");
    let request = ListResourceRecordSetsRequest {
        hosted_zone_id: zone_id.to_string(),
        start_record_name: Some(dns_name.clone()),
        ..Default::default()
    };
    if let Ok(response) = client.list_resource_record_sets(request).await {
        let record_sets = response.resource_record_sets;
        for record_set in record_sets {
            if record_set.name == dns_name && record_set.type_ == *record_type {
                if let Some(records) = &record_set.resource_records {
                    if let Some(record) = records.first() {
                        let ip_record = record.value.clone();
                        if record_type == "A" {
                            if ip_record.parse::<IpAddr>().is_ok() {
                                return Some(ip_record);
                            }
                        } else {
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
async fn set_dns_record(
    client: &Route53Client,
    zone_id: &str,
    zone_name: &str,
    host_name: &str,
    record_type: &str,
    record_value: &str,
) {
    // Build the request to change the resource record sets
    let dns_name = format!("{host_name}{zone_name}");
    let request = ChangeResourceRecordSetsRequest {
        hosted_zone_id: zone_id.to_string(),
        change_batch: ChangeBatch {
            changes: vec![Change {
                action: String::from("UPSERT"),
                resource_record_set: ResourceRecordSet {
                    name: dns_name,
                    resource_records: Some(vec![ResourceRecord {
                        value: record_value.to_string(),
                    }]),
                    ttl: Some(300),
                    type_: record_type.to_string(),
                    ..Default::default()
                },
            }],
            comment: Some(String::from("Updated by r53-ddns")),
        },
    };

    // Call the Route53 API to change the resource record sets
    let _response = client.change_resource_record_sets(request).await;
}
