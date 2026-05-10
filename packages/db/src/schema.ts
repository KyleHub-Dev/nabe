import { pgTable, text, timestamp, uuid, jsonb } from 'drizzle-orm/pg-core';

export const users = pgTable('users', {
  id: uuid('id').primaryKey().defaultRandom(),
  email: text('email').notNull().unique(),
  displayName: text('display_name').notNull(),
  createdAt: timestamp('created_at', { withTimezone: true }).notNull().defaultNow()
});

export const dnsInstances = pgTable('dns_instances', {
  id: uuid('id').primaryKey().defaultRandom(),
  name: text('name').notNull(),
  engine: text('engine').notNull(),
  baseUrl: text('base_url').notNull(),
  createdAt: timestamp('created_at', { withTimezone: true }).notNull().defaultNow()
});

export const deviceClients = pgTable('device_clients', {
  id: uuid('id').primaryKey().defaultRandom(),
  ownerUserId: uuid('owner_user_id').notNull().references(() => users.id),
  dnsInstanceId: uuid('dns_instance_id').notNull().references(() => dnsInstances.id),
  name: text('name').notNull(),
  clientTokenHash: text('client_token_hash').notNull(),
  status: text('status').notNull().default('active'),
  createdAt: timestamp('created_at', { withTimezone: true }).notNull().defaultNow()
});

export const instanceAccess = pgTable('instance_access', {
  id: uuid('id').primaryKey().defaultRandom(),
  userId: uuid('user_id').notNull().references(() => users.id),
  dnsInstanceId: uuid('dns_instance_id').notNull().references(() => dnsInstances.id),
  role: text('role').notNull(),
  createdAt: timestamp('created_at', { withTimezone: true }).notNull().defaultNow()
});

export const auditEvents = pgTable('audit_events', {
  id: uuid('id').primaryKey().defaultRandom(),
  actorUserId: uuid('actor_user_id').references(() => users.id),
  action: text('action').notNull(),
  targetType: text('target_type').notNull(),
  targetId: text('target_id').notNull(),
  metadata: jsonb('metadata'),
  createdAt: timestamp('created_at', { withTimezone: true }).notNull().defaultNow()
});

export const edgeNodes = pgTable('edge_nodes', {
  id: uuid('id').primaryKey().defaultRandom(),
  name: text('name').notNull(),
  status: text('status').notNull().default('enrolling'),
  createdAt: timestamp('created_at', { withTimezone: true }).notNull().defaultNow()
});

export const edgeHeartbeats = pgTable('edge_heartbeats', {
  id: uuid('id').primaryKey().defaultRandom(),
  edgeNodeId: uuid('edge_node_id').notNull().references(() => edgeNodes.id),
  status: text('status').notNull(),
  payload: jsonb('payload'),
  observedAt: timestamp('observed_at', { withTimezone: true }).notNull()
});
