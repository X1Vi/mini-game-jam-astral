#!/bin/bash
#
# Run Git Mailmap Setup for Astra Legends
# This script configures git user and applies mailmap
#

echo "🔧 Running Git Mailmap Setup..."
echo ""

# Set git user configuration
git config user.name "X1Vi"
git config user.email "gundeep477@gmail.com"

echo "✅ Git user configured: X1Vi <gundeep477@gmail.com>"
echo ""
echo "📄 Mailmap entries:"
echo "   X1Vi <gundeep477@gmail.com> <g_raven <gundeep@ravencast.io>>"
echo ""
echo "✅ Setup complete!"
echo ""
echo "📌 To view commits with updated names:"
echo "   git log --mailmap --oneline"
echo ""
echo "📌 To see commit statistics:"
echo "   git shortlog --mailmap -sn"
