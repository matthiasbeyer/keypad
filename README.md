# keypad

This piece of software implements my home keypad.

The keypad is a piece of hardware I was gifted, which has 25 buttons with a LED background.
It is configured and sends events via MQTT.

This software maps human-readable-ish MQTT commands to the keypad MQTT protocol
and translates button-press events to human-readable-ish MQTT messages.

For example something like:

> If key 1 is pressed, send to "keypad/commands" the following: `{"command":"pause_music"}`

Which can then be processed further by my home-assistant instance.

## License

(c) 2025 Matthias Beyer

MIT
