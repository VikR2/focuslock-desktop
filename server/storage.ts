import { 
  type Favorite, 
  type InsertFavorite,
  type BlockRule,
  type InsertBlockRule,
  type Session,
  type InsertSession,
  type Setting,
  type InsertSetting
} from "@shared/schema";
import { randomUUID } from "crypto";

// modify the interface with any CRUD methods
// you might need

export interface IStorage {
  // Favorites
  getFavorites(): Promise<Favorite[]>;
  addFavorite(favorite: InsertFavorite): Promise<Favorite>;
  removeFavorite(id: string): Promise<void>;
  
  // Block Rules
  getBlockRules(): Promise<BlockRule[]>;
  addBlockRule(rule: InsertBlockRule): Promise<BlockRule>;
  removeBlockRule(id: string): Promise<void>;
  updateBlockRule(id: string, updates: Partial<BlockRule>): Promise<BlockRule>;
  
  // Sessions
  getCurrentSession(): Promise<Session | undefined>;
  createSession(session: InsertSession): Promise<Session>;
  updateSession(id: string, updates: Partial<Session>): Promise<Session>;
  
  // Settings
  getSetting(key: string): Promise<Setting | undefined>;
  setSetting(setting: InsertSetting): Promise<Setting>;
}

export class MemStorage implements IStorage {
  private favorites: Map<string, Favorite>;
  private blockRules: Map<string, BlockRule>;
  private sessions: Map<string, Session>;
  private settings: Map<string, Setting>;

  constructor() {
    this.favorites = new Map();
    this.blockRules = new Map();
    this.sessions = new Map();
    this.settings = new Map();
  }

  // Favorites
  async getFavorites(): Promise<Favorite[]> {
    return Array.from(this.favorites.values()).sort((a, b) => (a.pinnedOrder || 0) - (b.pinnedOrder || 0));
  }

  async addFavorite(insertFavorite: InsertFavorite): Promise<Favorite> {
    const id = randomUUID();
    const favorite: Favorite = { 
      ...insertFavorite, 
      id,
      pinnedOrder: insertFavorite.pinnedOrder ?? null,
      iconHint: insertFavorite.iconHint ?? null
    };
    this.favorites.set(id, favorite);
    return favorite;
  }

  async removeFavorite(id: string): Promise<void> {
    this.favorites.delete(id);
  }

  // Block Rules
  async getBlockRules(): Promise<BlockRule[]> {
    return Array.from(this.blockRules.values());
  }

  async addBlockRule(insertRule: InsertBlockRule): Promise<BlockRule> {
    const id = randomUUID();
    const rule: BlockRule = { ...insertRule, id };
    this.blockRules.set(id, rule);
    return rule;
  }

  async removeBlockRule(id: string): Promise<void> {
    this.blockRules.delete(id);
  }

  async updateBlockRule(id: string, updates: Partial<BlockRule>): Promise<BlockRule> {
    const rule = this.blockRules.get(id);
    if (!rule) throw new Error(`Block rule ${id} not found`);
    const updated = { ...rule, ...updates };
    this.blockRules.set(id, updated);
    return updated;
  }

  // Sessions
  async getCurrentSession(): Promise<Session | undefined> {
    return Array.from(this.sessions.values()).find(session => session.status === 'running');
  }

  async createSession(insertSession: InsertSession): Promise<Session> {
    const id = randomUUID();
    const session: Session = { ...insertSession, id };
    this.sessions.set(id, session);
    return session;
  }

  async updateSession(id: string, updates: Partial<Session>): Promise<Session> {
    const session = this.sessions.get(id);
    if (!session) throw new Error(`Session ${id} not found`);
    const updated = { ...session, ...updates };
    this.sessions.set(id, updated);
    return updated;
  }

  // Settings
  async getSetting(key: string): Promise<Setting | undefined> {
    return this.settings.get(key);
  }

  async setSetting(insertSetting: InsertSetting): Promise<Setting> {
    const setting: Setting = { ...insertSetting };
    this.settings.set(setting.key, setting);
    return setting;
  }
}

export const storage = new MemStorage();
