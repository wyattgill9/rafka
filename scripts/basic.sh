#!/bin/bash

set -e

BROKER_PORT=50051
BROKER_PARTITION=0
TOTAL_PARTITIONS=1
TOPIC="greetings"
MESSAGE="Hello, World!"
KEY="example-key"

echo "Starting the Rafka broker..."
cargo run -q broker &
BROKER_PID=$!
echo "Broker is running with PID $BROKER_PID"
sleep 2 

echo "Starting the Rafka consumer..."
cargo run -q consumer &
CONSUMER_PID=$!
echo "Consumer is running with PID $CONSUMER_PID"
sleep 2 

echo "Publishing a message to the Rafka broker..."
cargo run -q producer 

echo "Waiting for the consumer to process the message..."
sleep 5

echo "Stopping the broker and consumer..."
kill $BROKER_PID $CONSUMER_PID
echo "Rafka demonstration completed."
