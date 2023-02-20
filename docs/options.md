---
layout: default
nav_order: 2
---

# Options

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
