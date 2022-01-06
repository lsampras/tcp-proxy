## TCP Proxy

usage
```
cargo run -- --ports <list of ports to proxy> --config_port <config server port> --rules <configuration file>
```

Cargo help
```
proxy 0.1.0

USAGE:
    main [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --config_port <config_port>    The port on which the http configuration server would run [default: 8001]
        --ports <ports>...             List of ports that the proxy should listen to [default: 3000]
        --rules <rules_path>           Configuration for connection routing logic [default: config.json]
```
This proxy listens on a list of ports provided via cli,

It also start a configuration server that can be used to change the state of the proxy or issue commands to it.

config-server options

Get current configuration of the proxy
```
curl 127.0.0.1:8001/routes
```
This fetches the current rules that the proxy is using


Update configuration for proxy
```
curl -X POST 127.0.0.1:8001/routes
   -H 'Content-Type: application/json'
   -d '{}' // json body here
```
This updates the rules for proxy targets.
Connections already established won't be affected by this, But all newer connections would start following the updated rules.

This rules aren't flushed to files as of now, so there won't be any state persistence between redeploys.


Stopping the proxy (gracefully)
this stops listening for new connections and shutdowns once all existing connections are completed
```
curl 127.0.0.1:8001/stop
```



Redeploying:
It is possible to redeploy the proxy without breaking any ongoing connections

Steps:
1. Start an instance of your proxy (proxy_ver_1)
	This would be your old version of proxy which is currently working properly and accepting connections
2. Start the newer version of your proxy which you wanna deploy
	The newer version while started is still waiting for the old proxy to stop listening on the inbound ports
3. run curl 127.0.0.1:8001/stop which asks the older proxy to stop
	This would cause the older proxy to stop listening on all ports (including the http configuration server)
	However it will still continue to proxy any tcp connections that are already established.
	The older proxy app won't quit until all the existing connections are closed
4. Start a new tcp connection which will be picked up by proxy_ver_2
	The proxy_ver_2 instance is now able to listen to the inbound ports that were given up by the older version
	Any new connections established would be proxied by this instance,
	Also the http configuration server for the new version would be available at the appropriate port