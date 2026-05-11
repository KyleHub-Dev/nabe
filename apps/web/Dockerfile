FROM node:lts-bookworm AS base
WORKDIR /app
RUN corepack enable
COPY package.json pnpm-workspace.yaml ./
COPY pnpm-lock.yaml ./
COPY apps/web/package.json apps/web/package.json
COPY packages/validation/package.json packages/validation/package.json
RUN pnpm install --filter @nabe/web... --frozen-lockfile=false
COPY apps/web apps/web
COPY packages/validation packages/validation
WORKDIR /app/apps/web
EXPOSE 5173
CMD ["pnpm", "dev"]
