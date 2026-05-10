FROM node:20-bookworm AS base
WORKDIR /app
RUN corepack enable
COPY package.json pnpm-workspace.yaml ./
COPY apps/web/package.json apps/web/package.json
RUN pnpm install --filter @nabe/web --frozen-lockfile=false
COPY apps/web apps/web
WORKDIR /app/apps/web
EXPOSE 5173
CMD ["pnpm", "dev"]
