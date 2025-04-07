#!/usr/bin/env bash

device_id="671fcf2b563d61d474af8ad8"
server="localhost"
sub_topic="sensors/realtime"
user="fellipe"
pass="123"
pid=0

trap ctrl_c INT

function ctrl_c() {
    printf "\n Trapped ctrl-c\n"
    echo "killing $pid"
    kill $pid
    exit 0
}

mosquitto_sub -h "$server" -t "$sub_topic" -u "$user" -P "$pass" | while read -r msg;
do
    echo $msg
    deviceId=$(echo "$msg" | jq -r .deviceId)
    start=$(echo "$msg" | jq -r .start)

    if [[ "$deviceId" == "$device_id" ]]; then
        if [[ "$start" == "true" ]]; then
            echo "Received new start state: $start"
            # script_pid=$( & echo $!)
            sh ./simulate-device-sending-data.sh &
            script_pid=$!
            pid=$script_pid
            echo $script_pid
        else
            echo "should stop pid: $script_pid"
            kill $script_pid
        fi
    else
        continue
    fi
done
