# AWS Route 53 Dynamic DNS

## Introduction

This app provides a way to keep a consistant url for a network where the external ip address may change from time to time, by adding a record to a domain where the primary DNS is hosted by the [Amazon Route 53](https://aws.amazon.com/route53/) service.

There are many reasons why you would want this, the most common is to provide a URL to a service hosted on a domestic network where the ISP is not providing a static ip address, e.g. webserver, vpn to home, etc..  There are other DDNS services out there that may give you a free option, I just want to use my own domain.

This is not is a publicly facing DDNS API, for that I would recomend another repository with a similar name: [aws-ddns](https://github.com/dixonwille/aws-ddns).  This is an application that provides DDNS using the R53 API, it is intended that you run this on one a computer within your network at a frequency that you are happy with.
