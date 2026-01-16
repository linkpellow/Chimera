# Scrapegoat Orchestrator - The General
# 
# This is the control center that manages the swarm of Chimera APEX workers.

FROM node:20-alpine as builder

WORKDIR /app

# Copy package files
COPY package*.json ./
RUN npm ci

# Copy source
COPY . .

# Build
RUN npm run build

# Production stage
FROM node:20-alpine

WORKDIR /app

# Install production dependencies only
COPY package*.json ./
RUN npm ci --only=production

# Copy built application
COPY --from=builder /app/dist ./dist

# Create non-root user
RUN addgroup -g 1000 scrapegoat && \
    adduser -D -u 1000 -G scrapegoat scrapegoat && \
    chown -R scrapegoat:scrapegoat /app

USER scrapegoat

EXPOSE 8080 8081

ENV NODE_ENV=production
ENV PORT=8080

CMD ["node", "dist/index.js"]
