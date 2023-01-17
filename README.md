<!-- markdownlint-configure-file {
  "MD033": false,
  "MD041": false
} -->

# AWS Route 53 Dynamic DNS

[![Crates.io](https://img.shields.io/crates/l/r53-ddns)](https://github.com/a1ecbr0wn/r53-ddns/blob/main/LICENSE) [![Crates.io](https://img.shields.io/crates/v/r53-ddns)](https://crates.io/crates/r53-ddns) [![Build Status](https://github.com/a1ecbr0wn/r53-ddns/workflows/CI%20Build/badge.svg)](https://github.com/a1ecbr0wn/r53-ddns/actions/workflows/build.yml) [![dependency status](https://deps.rs/repo/github/a1ecbr0wn/r53-ddns/status.svg)](https://deps.rs/repo/github/a1ecbr0wn/r53-ddns) [![snapcraft.io](https://snapcraft.io/r53-ddns/badge.svg)](https://snapcraft.io/r53-ddns)

## Introduction

This app provides a way to keep a consistant url for a network where the external ip address may change from time to time, by adding a record to a domain where the primary DNS is hosted by the [Amazon Route 53](https://aws.amazon.com/route53/) service.

There are many reasons why you would want this, the most common is to provide a URL to a service hosted on a domestic network where the ISP is not providing a static ip address, e.g. webserver, vpn to home, etc..  There are other DDNS services out there that may give you a free option, I just want to use my own domain.

This is not is a publicly facing DDNS API, for that I would recomend another repository with a similar name: [aws-ddns](https://github.com/dixonwille/aws-ddns).  This is an application that provides DDNS using the R53 API, it is intended that you run this on one a computer within your network at a frequency that you are happy with.

## Pre-Requisites

- An AWS account, this tool uses the Amazon Route 53 service which is part of AWS, so you need an account
- Your own domain, the Route 53 service allows you to manage the DNS of your own domain and this tool allows you to map a subdomain to your current external IP address
- [AWS credentials](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-configure.html) permissioned to modify R53 records stored in a local credentials file located at `$HOME/.aws/credentials` as you would for the AWS CLI.  You can also use the `AWS_SHARED_CREDENTIALS_FILE` environment variable to locate your credential file, or use `AWS_ACCESS_KEY` / `AWS_SECRET_ACCESS_KEY` environment variables to specify your credentials.

## How it works

The application works gets it's external network address over https from some of the many services that are available online.  The application has a number of these sites set up as default, but you can choose your own by providing a comma separated list of sites using the `-i` parameter.  Many of these service have limits to the frequency that you can call them, so r532-ddns limits each run to check a random two services out of the services available.  The value returned for the ip address is compared to the value that is stored in the Amazon Route 53 DNS settings, and if they differ, the DNS recordmis updated in Route 53.

## Options

``` sh
Set an Amazon Route 53 DNS record for the server/network

Usage: r53-ddns [OPTIONS]

Options:
  -s, --subdomain <SUBDOMAIN>          The subdomain to save (required)
  -d, --domain <DOMAIN>                The domain to save the record in  (required)
  -r, --region <REGION>                The aws region, [default: us-east-1]
  -i, --ipaddress-svc <IPADDRESS_SVC>  The ip address services to use, e.g. ident.me,ifconfig.me/ip
  -n, --nat                            The record is a nat router and so a *.<subdomain>.<domain> CNAME record will be set
  -v, --verbose                        Verbose logging
  -c, --check <CHECK>                  Consecutive check gap in seconds for continuous checking [default: 0]
  -V, --version                        Print version information
  -h, --help                           Print help
```

- To set a DNS record, the application must be provided with the subdomain and domain parameters.
- If the region for the domain is in a different region to the default `us-east-1`, it needs to be specified with the `-r` parameter.
- The default list of ip address https services can be overridden with the `-i` parameter.
- If the `-n` parameter is supplied, an additional *.<subdomain>.<domain> DNS record is set.  This can be used to route traffic to all hosts with that pattern to the network.
- To get the application to work continuously, the `-c` parameter can be used to pass in the length of the gap in seconds between consecutive checks.  Some of the ip address web servers will return errors if they are called too often, this application tries to address this by randomising the services that are used, but it is recomended that with the default list, the consecutive check gap is not below 120 seconds.

## Installation

- [Homebrew](docs/install-homebrew.md)
- [Snap](docs/install-snapcraft.md)
- [Cargo](docs/install-cargo.md)
- [Other Package Managers](docs/install-other.md)

## Usage

r53-ddns can be used adhoc if you wish but you probably want to set this up to run continuously in case your external ip address changes.  There are two ways that you can do this, using a job scheduler such as cron or as a service.


### Setup with cron

The following example is an entry into a cron file that will set up the subdomain `net.example.com`, performing the external ip and dns check every 5 minutes, assuming that the application has been installed via snap:

``` sh
*/2 * * * * /snap/bin/r53-ddns -s=net -d=example.com
```

At the top of the cron file, you may also want to declare the AWS environment variables that provide the credentials:

``` sh
AWS_ACCESS_KEY = ...
AWS_SECRET_ACCESS_KEY = ...
```

Some of the default services used to return the external ip address of your network will stop giving you a response if called too frequently, it is recomended that you don't call them more often than once every 2 minutes without increasing the number of configured services (via the `-i` parameter), hence the `*/2` in the cron example above.

### Setup as a service using `systemd`

- Create a file called `/etc/systemd/system/r53-ddns.service` as root.

``` toml
[Unit]
Description=R53 DDNS Service
After=network.target
StartLimitIntervalSec=0

[Service]
Type=simple
Restart=always
RestartSec=1
User=root
ExecStart=/snap/bin/r53-ddns -s=net -d=example.com -c=120

[Install]
WantedBy=multi-user.target
```

- That's it. Just start the service with the following:

``` sh
sudo systemctl start r53-ddns
```

## Contribute

Feel free to contribute I would be happy to take a look at any PRs raised.

[![Get it from the Snap Store](https://snapcraft.io/static/images/badges/en/snap-store-black.svg)](https://snapcraft.io/r53-ddns)
