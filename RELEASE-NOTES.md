# Release Notes

## v1.0.1

- Fix to issue [#26](https://github.com/a1ecbr0wn/r53-ddns/issues/26) where, after an interface failover on a multi-wan network, the cached reference to the AWS client cannot resolve the AWS API and fails to update the new DNS record

## v1.0.2

- Added the following new parameter for supplying a script that is called when a DNS record is changed or an error is detected.  For further information see: [alert-script](https://r53-ddns.a1ecbr0wn.com/docs/alert-script):

```text
-a, --alert-script <ALERT_SCRIPT>
```
