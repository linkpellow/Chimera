#!/bin/bash
# Railway Setup Script for Chimera
# This script helps set up the two services on Railway

set -e

echo "🚀 Chimera Railway Setup"
echo "========================"
echo ""

# Check if Railway CLI is installed
if ! command -v railway &> /dev/null; then
    echo "❌ Railway CLI not found. Install it with: npm i -g @railway/cli"
    exit 1
fi

# Check if logged in
if ! railway whoami &> /dev/null; then
    echo "❌ Not logged in. Run: railway login"
    exit 1
fi

echo "✅ Railway CLI ready"
echo ""

# Check if project is linked
if ! railway status &> /dev/null; then
    echo "⚠️  No project linked. Run: railway link -p scrape-goat"
    exit 1
fi

PROJECT_INFO=$(railway status)
echo "📦 Project Info:"
echo "$PROJECT_INFO"
echo ""

echo "📋 Next Steps:"
echo ""
echo "1. Create chimera-brain service:"
echo "   - Go to Railway Dashboard"
echo "   - Click 'New Service' → 'GitHub Repo'"
echo "   - Select your Chimera repo"
echo "   - Set Root Directory: /chimera-brain"
echo "   - Add variables:"
echo "     PORT=50052"
echo "     REDIS_URL=\${{Redis.REDIS_URL}}"
echo ""
echo "2. Create/Upgrade scrapegoat-worker-swarm service:"
echo "   - Go to Railway Dashboard"
echo "   - Click 'New Service' → 'GitHub Repo' (or upgrade existing)"
echo "   - Set Root Directory: /chimera-core"
echo "   - Add variables:"
echo "     CHIMERA_VISION_ADDR=http://chimera-brain.railway.internal:50052"
echo "     REDIS_URL=\${{Redis.REDIS_URL}}"
echo "     RUST_LOG=info"
echo ""
echo "3. Scale the swarm:"
echo "   - Set replica count to 50+"
echo ""
echo "📖 Full guide: See RAILWAY_DEPLOYMENT.md"
echo ""
