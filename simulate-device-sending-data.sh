#!/usr/bin/env bash

device_id="671fcf2b563d61d474af8ad8"
server="localhost"
sub_topic="sensors/realtime"
pub_topic="sensors/realtime/data"
user="fellipe"
pass="123"
start="false"  # Use string "false" for consistency

# Function to generate a random float
random_float() {
    min=$1
    max=$2
    awk -v min="$min" -v max="$max" 'BEGIN { srand(); print min + rand() * (max - min) }'
}

# Function to generate sensor data as JSON
generate_sensors_json() {
    local timestamp=$1
    cat <<EOF
[
    {
        "type": "temperature",
        "min": { "timestamp": $timestamp, "value": $(random_float 18.0 22.0) },
        "max": { "timestamp": $timestamp, "value": $(random_float 22.0 26.0) },
        "average": $(random_float 20.0 24.0)
    },
    {
        "type": "pressure",
        "min": { "timestamp": $timestamp, "value": $(random_float 87000 89000) },
        "max": { "timestamp": $timestamp, "value": $(random_float 89000 91000) },
        "average": $(random_float 88000 90000)
    },
    {
        "type": "humidity",
        "min": { "timestamp": $timestamp, "value": $(random_float 40.0 60.0) },
        "max": { "timestamp": $timestamp, "value": $(random_float 60.0 80.0) },
        "average": $(random_float 50.0 70.0)
    },
    {
        "type": "wind_direction",
        "min": { "timestamp": $timestamp, "value": $(random_float 0.0 180.0) },
        "max": { "timestamp": $timestamp, "value": $(random_float 180.0 360.0) },
        "average": $(random_float 90.0 270.0)
    },
    {
        "type": "wind_speed",
        "min": { "timestamp": $timestamp, "value": $(random_float 0.0 5.0) },
        "max": { "timestamp": $timestamp, "value": $(random_float 5.0 10.0) },
        "average": $(random_float 2.0 6.0)
    },
    {
        "type": "rain",
        "values": [
        { "timestamp": $timestamp, "value": $(random_float 0.0 10.0) }
        ]
    }
]
EOF
}

# Function to generate the full JSON payload
generate_json() {
    local current_timestamp=$(date +%s)
    cat <<EOF
{
    "deviceId": "$device_id",
    "timestamp": "$current_timestamp",
    "sensors": $(generate_sensors_json $current_timestamp)
}
EOF
}

while true; do
    json=$(generate_json)
    mosquitto_pub -h "$server" -t "$pub_topic" -m "$json" -u "$user" -P "$pass"
    sleep 10
done
