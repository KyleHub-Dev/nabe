FROM node:20-bookworm
WORKDIR /app
RUN corepack enable
COPY package.json pnpm-workspace.yaml ./
COPY apps/api/package.json apps/api/package.json
RUN pnpm install --filter @nabe/api --frozen-lockfile=false
COPY apps/api apps/api
WORKDIR /app/apps/api
EXPOSE 8080
CMD ["pnpm", "dev"]
