# Install dependencies only when needed
FROM node:18-slim AS deps
WORKDIR /app

# Install dependencies based on the preferred package manager
COPY apps/chat-client-next/package.json . 
COPY apps/chat-client-next/package-lock.json* . 
RUN npm ci


# Rebuild the source code only when needed
FROM node:18-slim AS builder

WORKDIR /app

COPY --from=deps /app/node_modules ./node_modules
COPY apps/chat-client-next/ .
# Disable next info stuff
ENV NEXT_TELEMETRY_DISABLED 1

RUN npm run build

# Production image, copy all the files and run next
FROM node:18-slim AS runner
WORKDIR /app

ENV NODE_ENV production
# Uncomment the following line in case you want to disable telemetry during runtime.
# ENV NEXT_TELEMETRY_DISABLED 1

RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs

COPY --from=builder /app/public ./public

# Automatically leverage output traces to reduce image size
# https://nextjs.org/docs/advanced-features/output-file-tracing
COPY --from=builder --chown=nextjs:nodejs /app/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/.next/static ./.next/static

USER nextjs

EXPOSE 3000

ENV PORT 3000

CMD ["node", "server.js"]