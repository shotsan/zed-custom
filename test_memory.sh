#!/bin/bash
# Test script to verify memory functionality

echo "Testing memory database..."

# Check if database file exists after running
DB_PATH="$HOME/.config/zed/memories.db"

if [ -f "$DB_PATH" ]; then
    echo "✓ Database file exists at: $DB_PATH"
    
    # Check if table exists
    TABLE_CHECK=$(sqlite3 "$DB_PATH" "SELECT name FROM sqlite_master WHERE type='table' AND name='memories';" 2>&1)
    
    if [ -n "$TABLE_CHECK" ]; then
        echo "✓ memories table exists"
        
        # Count memories
        COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM memories;" 2>&1)
        echo "✓ Memory count: $COUNT"
        
        # Show all memories
        echo ""
        echo "Stored memories:"
        sqlite3 "$DB_PATH" "SELECT content FROM memories;" 2>&1
    else
        echo "✗ memories table does NOT exist"
        echo "Database schema:"
        sqlite3 "$DB_PATH" ".schema"
    fi
else
    echo "✗ Database file does NOT exist at: $DB_PATH"
    echo "Expected location: $DB_PATH"
    ls -la "$HOME/.config/zed/" | grep -i "db\|memory"
fi
