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

## Installation

- [Homebrew](docs/install-homebrew.md)
- [Snap](docs/install-snapcraft.md)
- [Cargo](docs/install-cargo.md)
- [Other Package Managers](docs/install-other.md)

## Contribute

Feel free to contribute I would be happy to take a look at any PRs raised.

[![Get it from the Snap Store](https://snapcraft.io/static/images/badges/en/snap-store-black.svg)](https://snapcraft.io/r53-ddns)
