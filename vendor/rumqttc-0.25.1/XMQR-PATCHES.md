# XMQR patch of rumqttc 0.25.1

Source: published `rumqttc` 0.25.1 crate from crates.io, corresponding to upstream Git SHA `f1e9e8d558783f942993046679cdf3c8c3a3d36b` in `bytebeamio/rumqtt`. The source declares Apache-2.0; the license text from the upstream `rumqttc-0.25.1` tag is retained as `LICENSE`.

XMQR changes the following MQTT 5 paths:

- Expose `Request::Auth` and `AsyncClient::auth`, serialize it through the outgoing state and emit an outgoing event.
- Accept incoming AUTH in the packet decoder and event loop, including AUTH Success with zero remaining length; reject nonzero reserved AUTH header flags.
- Correct AUTH property byte counts and variable-length framing, reject duplicate/trailing properties, and re-export the AUTH property/reason types.
- For C104, enqueue an explicit DISCONNECT packet carrying Session Expiry Interval while retaining the original abbreviated packet when the option is absent.
- For C105, expose `AsyncClient::publish_packet` for a caller-owned QoS 1 Packet Identifier and DUP bit after the outbox has committed them. This does not add persistence to rumqttc itself.

The XMQR client does not use this copy for MQTT 3.1.1 behavior changes. When updating upstream, reapply these focused changes and run the C92 wire fixture and the existing MQTT 3.1.1/5 suite before replacing the pinned crate.
