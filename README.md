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

The application works gets it's external network address over https from some of the many services that are available online.  The application has a number of these sites set up as default, but you can choose your own by providing a comma separated list of sites using the `-i` parameter.  Many of these service have limits to the frequency that you can call them, so r532-ddns limits each run to check a random two services out of the services available.  The value returned for the ip address is compared to the value that is stored in the Amazon Route 53 DNS settings, and if they differ, the DNS record is updated in Route 53.

## Options

``` text
$ r53-ddns -h

Set an Amazon Route 53 DNS record for the server/network

Usage: r53-ddns [OPTIONS]

Options:
  -s, --subdomain <SUBDOMAIN>          The subdomain to save (required)
  -d, --domain <DOMAIN>                The domain to save the record in (required)
  -r, --region <REGION>                The aws region [default: us-east-1]
  -i, --ipaddress-svc <IPADDRESS_SVC>  The ip address services to use, e.g. ident.me,ifconfig.me/ip
  -n, --nat                            The record is a nat router and so a *.<subdomain>.<domain> CNAME record will be set
  -l, --logdir <LOGDIR>                Absolute path for the directory where log file should be written [default: /var/tmp]
  -a, --alert-script <ALERT_SCRIPT>    Script called on error or ip address changes, see: https://r53-ddns.a1ecbr0wn.com/alert-script
  -c, --check <CHECK>                  Consecutive check gap in seconds for continuous checking [default: 0]
  -v, --verbose                        Verbose logging
  -V, --version                        Print version information
  -h, --help                           Print help
```

- To set a DNS record, the application must be provided with the subdomain and domain parameters.
- If the region for the domain is in a different region to the default `us-east-1`, it needs to be specified with the `-r` parameter.
- The default list of ip address https services can be overridden with the `-i` parameter.
- If the `-n` parameter is supplied, an additional `*.<subdomain>.<domain>` DNS record is set.  This can be used to route traffic to all hosts with that pattern to the network.
- To get the application to work continuously, the `-c` parameter can be used to pass in the length of the gap in seconds between consecutive checks.  Some of the ip address web servers will return errors if they are called too often, this application tries to address this by randomising the services that are used, but it is recomended that with the default list, the consecutive check gap is not below 300 seconds.

## Installation

- [Homebrew](docs/install-homebrew.md)
- [Snap](docs/install-snapcraft.md)
- [Cargo](docs/install-cargo.md)
- [Other Package Managers](docs/install-other.md)

## Usage

r53-ddns can be used adhoc if you wish, but you probably want to set this up to run continuously in case your external ip address changes.  There are two ways that you can do this, using a job scheduler such as cron or as a service.

- [Setup cron](docs/setup-cron.md)
- [Setup Systemd Service on Linux](docs/setup-systemd.md)
- [Setup Launchd Service on MacOS](docs/setup-launchd.md)

## Issues

If you are having issues with the execution of this application, start by enabling verbose logging with the `-v` parameter.  If you are trying to resolve an issue while running this in cron or as a service, logging is enabled by default to the file `/var/tmp/r53-ddns.log` which is a file that should be writable whichever user you are sunning the application under.

If the issues are not obvious or you think there is a bug, please raise an issue via [GitHub](https://github.com/a1ecbr0wn/homebrew-r53-ddns/issues).

## Contribute

Feel free to contribute I would be happy to take a look at any PRs raised.

[![Get it from the Snap Store](https://snapcraft.io/static/images/badges/en/snap-store-black.svg)](https://snapcraft.io/r53-ddns)

## Other Info

- [Release Notes](RELEASE-NOTES.md)
