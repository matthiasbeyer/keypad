#!/bin/python
# python 3.11

import random
import json
import struct

from paho.mqtt import client as mqtt_client

broker = 'mqtt.subraum.c3pb.de'
port = 1883
topic_mx = "mx-blue/arr/out"
topic_buzzer = "xuhaktu/buzzer"
topic_announce = "/unicorn"
# Generate a Client ID with the subscribe prefix.
client_id = f'klausdieter-{random.randint(0, 100)}'

released_default="101010"
released_default_no_sound="000000"
pressed_default="00ff00"

def connect_mqtt() -> mqtt_client:
    def on_connect(client, userdata, flags, rc):
        if rc == 0:
            print("Connected to MQTT Broker!")
        else:
            print("Failed to connect, return code %d\n", rc)

    client = mqtt_client.Client(mqtt_client.CallbackAPIVersion.VERSION1, client_id)
    # client.username_pw_set(username, password)
    client.on_connect = on_connect
    client.connect(broker, port)
    return client

def handle_keystroke(client, userdata, msg):
    print(msg)

def handle_announce(client, userdata, msg):
    try:
        released_colors = [released_default_no_sound] * 25
        pressed_colors = [pressed_default] * 25

        client.publish("mx-blue/arr/released", struct.pack(">HH",0,25) + bytes.fromhex("".join(pressed_colors)))
        client.publish("mx-blue/arr/pressed", struct.pack(">HH",0,25) + bytes.fromhex("".join(released_colors)))
    except Exception as e:
        print(e)


def buzz(client, userdata, msg):
    if msg.payload.decode() == "+0":
        print("buzz")
        #sob_top = "edi/cmd/sob"
        #result = client.publish(sob_top, json.dumps({"args":"hackbuzzer", "user":"klausdieter"}))
        ## result: [0, 1]
        #status = result[0]
        #if status == 0:
        #    print(f"Send `{sob_top}` to topic `{sob_top}`")
        #else:
        #    print(f"Failed to send message to topic {sob_top}")


def on_message(client, userdata, msg):
    print(f"Received `{msg.payload.decode()}` from `{msg.topic}` topic")
    if msg.topic == topic_mx:
        handle_keystroke(client, userdata, msg)
        handle_announce(client, userdata, msg)

    if msg.topic == topic_buzzer:
        buzz(client, userdata, msg)

    if msg.topic == topic_announce and msg.payload.decode().startswith("mx-blue.connect"):
        handle_announce(client, userdata, msg)


def subscribe(client: mqtt_client):
    client.subscribe(topic_mx)
    client.subscribe(topic_buzzer)
    client.subscribe(topic_announce)
    client.on_message = on_message

def run():
    client = connect_mqtt()
    subscribe(client)
    client.loop_forever()

if __name__ == '__main__':
    run()

