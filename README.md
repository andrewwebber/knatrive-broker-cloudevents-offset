# Broker replays events

From v0.19 auto.offset.reset was changed from **latest** to **earliest** resulting in cloudevents being replayed to consumers.
This can result in duplicated messages from the point of view of the consumer.

## Reproduce duplicated events

1. Create basic services
```bash
kubectl apply -f ./services.yaml
```

2. Post to trigger a cloudevent chain reaction
```bash
curl -X POST http://foo.default.127.0.0.1.sslip.io/cloudevent
kubectl logs -l serving.knative.dev/configuration=bar2 -c user-container -f
```

3. Delete triggers
```bash
kubectl delete --all trigers
```

4. Recreate triggers
```bash
kubectl apply -f ./services.yaml
```

5. Experience cloudevents from the past
```bash
kubectl logs -l serving.knative.dev/configuration=bar2 -c user-container -f
```

## Workaround

1. Patch kafka configs
```bash
kubectl edit configmaps -n knative-eventing config-kafka-broker-data-plane
```

```yaml

apiVersion: v1
data:
  config-kafka-broker-consumer.properties: |
    key.deserializer=org.apache.kafka.common.serialization.StringDeserializer
    value.deserializer=io.cloudevents.kafka.CloudEventDeserializer
    fetch.min.bytes=1
    heartbeat.interval.ms=3000
    max.partition.fetch.bytes=1048576
    session.timeout.ms=10000
    # ssl.key.password=
    # ssl.keystore.location=
    # ssl.keystore.password=
    # ssl.truststore.location=
    # ssl.truststore.password=
    allow.auto.create.topics=true
    auto.offset.reset=latest
```

2. Restart services
```bash

kubectl rollout restart deployment -n knative-eventing kafka-broker-receiver
kubectl rollout restart deployment -n knative-eventing kafka-broker-dispatcher
```
