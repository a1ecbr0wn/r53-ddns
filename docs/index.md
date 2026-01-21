---
layout: docs
title: "r53-ddns Amazon Route 53 DDNS | r53-ddns"
---

## AWS Route 53 Dynamic DNS

### Introduction

This app provides a way to keep a consistant url for a network where the
external ip address may change from time to time, by adding a record to a domain
where the primary DNS is hosted by the
[Amazon Route 53](https://aws.amazon.com/route53/) service.

There are many reasons why you would want this, the most common is to provide a
URL to a service hosted on a domestic network where the ISP is not providing a
static ip address, e.g. webserver, vpn to home, etc.. There are other DDNS
services out there that may give you a free option, I just want to use my own domain.

This is not is a publicly facing DDNS API, for that I would recomend another
repository with a similar name:
[aws-ddns](https://github.com/dixonwille/aws-ddns). This is an application that
provides DDNS using the R53 API, it is intended that you run this on one a
computer within your network at a frequency that you are happy with.

### Pre-Requisites

- An AWS account, this tool uses the Amazon Route 53 service which is part of
  AWS, so you need an account
- Your own domain, the Route 53 service allows you to manage the DNS of your own
  domain and this tool allows you to map a subdomain to your current external IP
  address
- [AWS credentials](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-configure.html)
  permissioned to modify R53 records stored in a local credentials file located at
  `$HOME/.aws/credentials` as you would for the AWS CLI. You can also use the
  `AWS_SHARED_CREDENTIALS_FILE` environment variable to locate your credential
  file, or use `AWS_ACCESS_KEY` / `AWS_SECRET_ACCESS_KEY` environment variables
  to specify your credentials.

### How it works

The application works gets it's external network address over https from some of
the many services that are available online. The application has a number of
these sites set up as default, but you can choose your own by providing a comma
separated list of sites using the `-i` parameter. Many of these service have
limits to the frequency that you can call them, so r532-ddns limits each run to
check a random two services out of the services available. The value returned
for the ip address is compared to the value that is stored in the Amazon Route
53 DNS settings, and if they differ, the DNS record is updated in Route 53.
