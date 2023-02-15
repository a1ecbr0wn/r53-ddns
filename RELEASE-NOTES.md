# Release Notes

## v1.0.1

- Fix to issue [#26](https://github.com/a1ecbr0wn/r53-ddns/issues/26) where, after an interface failover on a multi-wan network, the cached reference to the AWS client cannot resolve the AWS API and fails to update the new DNS record
