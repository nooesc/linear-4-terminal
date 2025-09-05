#!/bin/bash

# Test script for Linear CLI interactive mode
echo "Linear CLI Interactive Mode Test"
echo "================================"
echo ""
echo "This script will test the interactive mode in a proper TTY environment."
echo ""

# Check if we're in a TTY
if [ -t 0 ] && [ -t 1 ]; then
    echo "✓ Running in TTY environment"
else
    echo "⚠ Not running in a TTY. Attempting to create pseudo-TTY..."
    # Use script command to create a pseudo-TTY
    exec script -q -c "$0" /dev/null
fi

echo ""
echo "Testing project and label selection..."
echo ""
echo "Instructions:"
echo "1. Navigate to any issue using j/k or arrow keys"
echo "2. Press 'p' to edit the project (test for crash)"
echo "3. Press 'l' to edit labels (test for crash)"
echo "4. Press 'q' to quit"
echo ""
echo "Log files will be saved to: ~/.cache/linear-cli/logs/"
echo ""
echo "Press Enter to start the test..."
read

# Run the linear CLI
linear interactive

# Check exit status
EXIT_CODE=$?
echo ""
echo "Linear CLI exited with code: $EXIT_CODE"

# Find the latest log file
if [ -d ~/.cache/linear-cli/logs/ ]; then
    LATEST_LOG=$(ls -t ~/.cache/linear-cli/logs/ | head -1)
    if [ ! -z "$LATEST_LOG" ]; then
        echo "Latest log file: ~/.cache/linear-cli/logs/$LATEST_LOG"
        
        # Check for errors in the log
        if grep -q "ERROR\|PANIC" ~/.cache/linear-cli/logs/$LATEST_LOG; then
            echo ""
            echo "⚠ Errors found in log:"
            grep "ERROR\|PANIC" ~/.cache/linear-cli/logs/$LATEST_LOG
        fi
        
        # Show last 20 lines of the log
        echo ""
        echo "Last 20 lines of log:"
        echo "===================="
        tail -20 ~/.cache/linear-cli/logs/$LATEST_LOG
    fi
fi

# Check for panic log reference
if [ -f /tmp/linear-cli-last-log.txt ]; then
    echo ""
    echo "⚠ Panic detected! Log file: $(cat /tmp/linear-cli-last-log.txt)"
    rm /tmp/linear-cli-last-log.txt
fi