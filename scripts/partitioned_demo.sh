#!/bin/bash
set -e

# Configuration
BROKER_COUNT=3
BASE_PORT=50051
TOPIC="partitioned-topic"
MESSAGE_COUNT=10

echo "Starting Rafka partitioned demo..."
echo "- Brokers: $BROKER_COUNT"
echo "- Topic: $TOPIC"
echo "- Messages: $MESSAGE_COUNT"

# Start multiple brokers
BROKER_PIDS=()
for ((i=0; i<BROKER_COUNT; i++)); do
    PORT=$((BASE_PORT + i))
    echo "Starting broker $i on port $PORT..."
    cargo run -q broker --port $PORT &
    BROKER_PIDS+=($!)
    sleep 2
done

# Start consumers (one per partition/broker)
CONSUMER_PIDS=()
for ((i=0; i<BROKER_COUNT; i++)); do
    PORT=$((BASE_PORT + i))
    echo "Starting consumer $i on port $PORT..."
    cargo run -q consumer --port $PORT --partition $i &
    CONSUMER_PIDS+=($!)
    sleep 2
done

# Send messages with partitioning
echo "Sending messages..."
for ((i=0; i<MESSAGE_COUNT; i++)); do
    # Simple hash-based partitioning
    PARTITION=$((i % BROKER_COUNT))
    PORT=$((BASE_PORT + PARTITION))
    MESSAGE="Message-$i"
    BROKER_ADDRESS="127.0.0.1:$PORT"
    echo "Sending '$MESSAGE' to partition $PARTITION (port $PORT)"
    cargo run -q producer --brokers $BROKER_ADDRESS --message "$MESSAGE"
    sleep 1
done

# Clean up
echo "Cleaning up..."
for pid in "${BROKER_PIDS[@]}" "${CONSUMER_PIDS[@]}"; do
    kill $pid 2>/dev/null || true
done

echo "Demo completed" 
