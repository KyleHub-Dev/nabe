FROM node:lts-bookworm
WORKDIR /app
RUN corepack enable
COPY package.json pnpm-workspace.yaml ./
COPY apps/worker/package.json apps/worker/package.json
RUN pnpm install --filter @nabe/worker --frozen-lockfile=false
COPY apps/worker apps/worker
WORKDIR /app/apps/worker
CMD ["pnpm", "dev"]
