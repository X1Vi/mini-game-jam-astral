#!/bin/bash
#
# Git Mailmap Configuration Script for Astra Legends
# This script sets up git user configuration and applies the mailmap
#

set -e

echo "🔧 Git Mailmap Configuration Script"
echo "===================================="
echo ""

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "❌ Error: Not a git repository"
    echo "Please run this script from the project root directory"
    exit 1
fi

# Set user configuration
echo "📝 Setting up git user configuration..."
git config user.name "X1Vi"
git config user.email "gundeep477@gmail.com"
echo "✅ Git user configured as: X1Vi <gundeep477@gmail.com>"
echo ""

# Check if .mailmap exists
if [ -f ".mailmap" ]; then
    echo "✅ .mailmap file found"
    echo ""
    
    # Show what's in the mailmap
    echo "📄 Current mailmap entries:"
    cat .mailmap
    echo ""
    
    # Verify mailmap is working
    echo "🔍 Verifying mailmap is active..."
    if git config core.mailmap > /dev/null 2>&1; then
        echo "✅ Mailmap is configured"
    else
        echo "⚠️  Mailmap needs to be enabled (it should be auto-detected)"
    fi
else
    echo "❌ .mailmap file not found"
    echo "Please create a .mailmap file first"
    exit 1
fi

echo ""
echo "📊 Test: Showing recent commits with updated author info:"
echo "----------------------------------------------------------"
git log --format="%an <%ae>" -5

echo ""
echo "✅ Configuration complete!"
echo ""
echo "📌 How to use:"
echo "   - All new commits will be attributed to X1Vi <gundeep477@gmail.com>"
echo "   - Existing commits will show updated author info with: git log --mailmap"
echo "   - To see original vs updated names: git shortlog --mailmap -sn"
echo ""
echo "🔧 To update existing commits (rewrites history):"
echo "   git filter-branch --env-filter 'export GIT_AUTHOR_NAME=\"X1Vi\"; export GIT_AUTHOR_EMAIL=\"gundeep477@gmail.com\"' HEAD"
echo ""
echo "⚠️  WARNING: Rewriting history should only be done on private branches!"
echo ""
