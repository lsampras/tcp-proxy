## Global Proxy:

For a global proxy which would run in a distributed fashion,
some points to consider are

------

### State management:-
Current state information to make routing decisions for an individual proxy would be,
-> list of targets
-> latency to the targets
-> some list of adjacent proxies that it can communicate with

#### list of targets ->
this info can be provided by the orchestration service periodically (or polled by the proxy server) for each proxy instance.
In case there is a very high amount of updates, we can update the state by allowing the peer-to-peer updates (this would use the http config port for each server),
(this will require the config to maintain some sort of version number, so proxy instances can understand whether the proxy they've received is a newer one)
we can also split the config into individial rules 
e.g config a contains ["rule_1": {"version": 1, "data":{}},"rule_2": {"version": 2, "data":{}}]
additional rules will be given a unique rule identifier & updates would be recognized with the rule versions


#### latency at target ->
Latency to the corresponding target, this data can be updated each time we need to make a request to the target,
For initializing this data we can either fetch it from a peer instance or use a default value (which will be updated after each request)
preferably we could add a TTL/decay for the latency records so that the proxy can discover new targets (which might have been ignored due to high latency).


#### Adjacent proxies ->
this is needed if we want to use peer to peer communication between proxy instances,
On initialization the instance needs some method of connecting to the neighbouring proxies
the adjacent proxies can be discovered between instances once atleast 1 proxy is met,
if proxy1 & proxy2 are neighbouring proxy instances aware of each other then proxy1 & proxy2 share information
about their respective neighbouring proxies, after this each of them update their list to retain the n closest instances (we can check the latency to determine how close a proxy is)
we can also choose to hardcode this info if we have a fixed topology

----------

### Communication:
All communication could be done over the attached http server for each proxy instance.
The various type of intra cluster communications that would take place are.

GET Configuration (or Rule): => (returns the current configuration on a server)

POST Configuration (or Rule): => (updates the configuration of a server, Only intended if need to manually set the conifguration)

POST ConfigurationUpdate: => 
(sends a condensed version of rule updates, i.e only the latest version and no actual data,
If the destination thinks it needs to update it can ask you back for a complete copy with GET configuration)

GET Neighbour => (get the adjacent proxies from a proxy instance => this should return a list of IP Address)
the number of neighbours can be fixed e.g (5 => only store 5 proxy address with least latency (or hardcoded 5 address that are adjacent))

GET LatencyInfo => (This can be helpful when creating new proxy instances)

