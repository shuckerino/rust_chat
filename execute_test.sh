#!/bin/bash

# Determine the absolute path to the directory containing this script
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

# Read database password
read -s -p "Enter the database password: " DB_PASSWORD

# Reset the database
echo "Setting up the database"
mysql --host 10.8.13.39 --port 47777 --user root -p${DB_PASSWORD} chatclient < "${SCRIPT_DIR}/src/database_setup.sql"

# Run the tests with cargo-tarpaulin
echo "Running the tests"
cargo tarpaulin

# Reset the database again after tests
echo "Resetting the database"
mysql --host 10.8.13.39 --port 47777 --user root -p${DB_PASSWORD} chatclient < "${SCRIPT_DIR}/src/database_setup.sql"
