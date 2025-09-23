import { sql } from "drizzle-orm";
import { pgTable, text, varchar, integer, boolean } from "drizzle-orm/pg-core";
import { createInsertSchema } from "drizzle-zod";
import { z } from "zod";

// FocusLock data models
export const favorites = pgTable("favorites", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  appId: text("app_id").notNull(),
  displayName: text("display_name").notNull(),
  pinnedOrder: integer("pinned_order"),
  iconHint: text("icon_hint"),
});

export const blockRules = pgTable("block_rules", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  appId: text("app_id").notNull(),
  matchKind: text("match_kind").notNull(), // 'exe'|'package'|'lnk'|'path'|'regex'
  mode: text("mode").notNull(), // 'hard'|'soft'
});

export const sessions = pgTable("sessions", {
  id: varchar("id").primaryKey().default(sql`gen_random_uuid()`),
  startUtc: integer("start_utc").notNull(),
  endUtc: integer("end_utc").notNull(),
  status: text("status").notNull(), // 'scheduled'|'running'|'completed'|'canceled'
  durationSecs: integer("duration_secs").notNull(),
});

export const settings = pgTable("settings", {
  key: text("key").primaryKey(),
  value: text("value").notNull(),
});

// Enum schemas for validation
export const matchKindSchema = z.enum(['exe', 'package', 'lnk', 'path', 'regex']);
export const blockModeSchema = z.enum(['hard', 'soft']);
export const sessionStatusSchema = z.enum(['scheduled', 'running', 'completed', 'canceled']);

// Insert schemas
export const insertFavoriteSchema = createInsertSchema(favorites).omit({ id: true });
export const insertBlockRuleSchema = createInsertSchema(blockRules).omit({ id: true }).extend({
  matchKind: matchKindSchema,
  mode: blockModeSchema,
});
export const insertSessionSchema = createInsertSchema(sessions).omit({ id: true }).extend({
  status: sessionStatusSchema,
});
export const insertSettingSchema = createInsertSchema(settings);

// Update schemas
export const updateSessionSchema = insertSessionSchema.partial();
export const updateBlockRuleSchema = insertBlockRuleSchema.partial();
export const updateSettingSchema = insertSettingSchema.partial();

// Types
export type Favorite = typeof favorites.$inferSelect;
export type InsertFavorite = z.infer<typeof insertFavoriteSchema>;
export type BlockRule = typeof blockRules.$inferSelect;
export type InsertBlockRule = z.infer<typeof insertBlockRuleSchema>;
export type Session = typeof sessions.$inferSelect;
export type InsertSession = z.infer<typeof insertSessionSchema>;
export type Setting = typeof settings.$inferSelect;
export type InsertSetting = z.infer<typeof insertSettingSchema>;

// Additional types for the app
export type MatchKind = 'exe' | 'package' | 'lnk' | 'path' | 'regex';
export type BlockMode = 'hard' | 'soft';
export type SessionStatus = 'scheduled' | 'running' | 'completed' | 'canceled';

export interface AppSummary {
  appId: string;
  displayName: string;
  exeOrTarget?: string;
  packageFamily?: string;
  iconHint?: string;
}
