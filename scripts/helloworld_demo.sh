#!/bin/bash
set -e

echo "Starting Rafka demo..."

# Start broker
echo "Starting broker..."
cargo run -q broker &
BROKER_PID=$!
sleep 2

# Start consumer
echo "Starting consumer..."
cargo run -q consumer &
CONSUMER_PID=$!
sleep 2

# Run producer
echo "Sending message..."
cargo run -q producer --message "Hello, World!"
sleep 1

# Clean up
kill $BROKER_PID $CONSUMER_PID 2>/dev/null || true

echo "Demo completed"
