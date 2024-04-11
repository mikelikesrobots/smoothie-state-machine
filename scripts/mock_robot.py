from awscrt import mqtt
from awsiot import mqtt_connection_builder
import boto3
import json
from pathlib import Path
import time


class MockRobot:
    def __init__(self):
        self._conn = None

        client = boto3.client("iot")
        endpoint = client.describe_endpoint(endpointType="iot:Data-ATS")["endpointAddress"]

        certs_path = Path(__file__).resolve().parent / "certs"

        self._conn = mqtt_connection_builder.mtls_from_path(
            endpoint=endpoint,
            port=8883,
            cert_filepath=str(certs_path / "mock_robot_cert.pem"),
            pri_key_filepath=str(certs_path / "mock_robot_priv.key"),
            client_id="MockRobot",
        )
        connect_future = self._conn.connect()
        connect_future.result()

        message_topic = "robots/+/order"
        print("Subscribing to topic '{}'...".format(message_topic))
        subscribe_future, _ = self._conn.subscribe(
            topic=message_topic,
            qos=mqtt.QoS.AT_LEAST_ONCE,
            callback=self._on_message_received
        )

        subscribe_result = subscribe_future.result()
        print("Subscribed with {}".format(str(subscribe_result['qos'])))

    def _on_message_received(self, topic, payload, **kwargs):
        print("Received message from topic '{}': {}".format(topic, payload))
        payload = json.loads(payload)
        token = payload["task_token"]
        robot = topic.split("/")[1]
        topic = f"robots/{robot}/success"

        time.sleep(3)
        print("Smoothie made - sending task success!")
        msg = json.dumps({"TaskToken": token})
        self._conn.publish(topic=topic, payload=msg, qos=mqtt.QoS.AT_LEAST_ONCE)


x = MockRobot()
while True:
    time.sleep(1)
