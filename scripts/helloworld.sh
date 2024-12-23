#!/bin/bash

# Start the broker in the background
echo "Starting the broker..."
RUST_LOG=info cargo run --bin broker & 
BROKER_PID=$!
sleep 2 # Give the broker some time to start

# Start the producer and publish a message
echo "Testing the producer..."
RUST_LOG=info cargo run --bin producer -- "127.0.0.1:8080" "test_topic" "key1" "Hello, Rafka!" &
PRODUCER_PID=$!
sleep 2 # Allow some time for the producer to run

# Start the consumer and attempt to consume the message
echo "Testing the consumer..."
RUST_LOG=info cargo run --bin consumer -- "127.0.0.1:8080" "test_topic" &
CONSUMER_PID=$!
sleep 5 # Allow some time for the consumer to run

# Kill all background processes
echo "Cleaning up..."
kill $BROKER_PID $PRODUCER_PID $CONSUMER_PID
wait $BROKER_PID $PRODUCER_PID $CONSUMER_PID 2>/dev/null

echo "Test complete."
