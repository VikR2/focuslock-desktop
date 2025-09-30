import type { Express } from "express";
import { createServer, type Server } from "http";
import { storage } from "./storage";
import { 
  insertSessionSchema, 
  insertBlockRuleSchema, 
  insertFavoriteSchema, 
  insertSettingSchema,
  updateSessionSchema,
  updateBlockRuleSchema
} from "@shared/schema";

export async function registerRoutes(app: Express): Promise<Server> {
  // Health check endpoint
  app.get("/api/health", (req, res) => {
    res.json({ status: "ok", timestamp: Date.now() });
  });

  // Session Management Routes
  app.post("/api/sessions", async (req, res) => {
    try {
      const sessionData = insertSessionSchema.parse(req.body);
      const session = await storage.createSession(sessionData);
      res.status(201).location(`/api/sessions/${session.id}`).json(session);
    } catch (error) {
      if (error instanceof Error && error.name === 'ZodError') {
        res.status(400).json({ error: "Invalid session data" });
      } else {
        res.status(500).json({ error: "Internal server error" });
      }
    }
  });

  app.get("/api/sessions/current", async (req, res) => {
    try {
      const session = await storage.getCurrentSession();
      res.json(session || null);
    } catch (error) {
      res.status(500).json({ error: "Failed to get current session" });
    }
  });

  app.patch("/api/sessions/:id", async (req, res) => {
    try {
      const { id } = req.params;
      const updates = updateSessionSchema.parse(req.body);
      const session = await storage.updateSession(id, updates);
      res.json(session);
    } catch (error) {
      if (error instanceof Error && error.message.includes('not found')) {
        res.status(404).json({ error: "Session not found" });
      } else if (error instanceof Error && error.name === 'ZodError') {
        res.status(400).json({ error: "Invalid update data" });
      } else {
        res.status(500).json({ error: "Internal server error" });
      }
    }
  });

  // Block Rules Routes
  app.get("/api/block-rules", async (req, res) => {
    try {
      const rules = await storage.getBlockRules();
      res.json(rules);
    } catch (error) {
      res.status(500).json({ error: "Failed to get block rules" });
    }
  });

  app.post("/api/block-rules", async (req, res) => {
    try {
      const ruleData = insertBlockRuleSchema.parse(req.body);
      const rule = await storage.addBlockRule(ruleData);
      res.status(201).location(`/api/block-rules/${rule.id}`).json(rule);
    } catch (error) {
      if (error instanceof Error && error.name === 'ZodError') {
        res.status(400).json({ error: "Invalid block rule data" });
      } else {
        res.status(500).json({ error: "Internal server error" });
      }
    }
  });

  app.patch("/api/block-rules/:id", async (req, res) => {
    try {
      const { id } = req.params;
      const updates = updateBlockRuleSchema.parse(req.body);
      const rule = await storage.updateBlockRule(id, updates);
      res.json(rule);
    } catch (error) {
      if (error instanceof Error && error.message.includes('not found')) {
        res.status(404).json({ error: "Block rule not found" });
      } else if (error instanceof Error && error.name === 'ZodError') {
        res.status(400).json({ error: "Invalid update data" });
      } else {
        res.status(500).json({ error: "Internal server error" });
      }
    }
  });

  app.delete("/api/block-rules/:id", async (req, res) => {
    try {
      const { id } = req.params;
      await storage.removeBlockRule(id);
      res.status(204).send();
    } catch (error) {
      if (error instanceof Error && error.message.includes('not found')) {
        res.status(404).json({ error: "Block rule not found" });
      } else {
        res.status(500).json({ error: "Internal server error" });
      }
    }
  });

  // Favorites Routes
  app.get("/api/favorites", async (req, res) => {
    try {
      const favorites = await storage.getFavorites();
      res.json(favorites);
    } catch (error) {
      res.status(500).json({ error: "Failed to get favorites" });
    }
  });

  app.post("/api/favorites", async (req, res) => {
    try {
      const favoriteData = insertFavoriteSchema.parse(req.body);
      const favorite = await storage.addFavorite(favoriteData);
      res.status(201).location(`/api/favorites/${favorite.id}`).json(favorite);
    } catch (error) {
      if (error instanceof Error && error.name === 'ZodError') {
        res.status(400).json({ error: "Invalid favorite data" });
      } else {
        res.status(500).json({ error: "Internal server error" });
      }
    }
  });

  app.delete("/api/favorites/:id", async (req, res) => {
    try {
      const { id } = req.params;
      await storage.removeFavorite(id);
      res.status(204).send();
    } catch (error) {
      if (error instanceof Error && error.message.includes('not found')) {
        res.status(404).json({ error: "Favorite not found" });
      } else {
        res.status(500).json({ error: "Internal server error" });
      }
    }
  });

  // Settings Routes
  app.get("/api/settings/:key", async (req, res) => {
    try {
      const { key } = req.params;
      const setting = await storage.getSetting(key);
      res.json(setting || null);
    } catch (error) {
      res.status(500).json({ error: "Failed to get setting" });
    }
  });

  app.post("/api/settings", async (req, res) => {
    try {
      const settingData = insertSettingSchema.parse(req.body);
      const setting = await storage.setSetting(settingData);
      res.status(201).location(`/api/settings/${setting.key}`).json(setting);
    } catch (error) {
      if (error instanceof Error && error.name === 'ZodError') {
        res.status(400).json({ error: "Invalid setting data" });
      } else {
        res.status(500).json({ error: "Internal server error" });
      }
    }
  });

  // App Discovery Routes
  app.get("/api/apps/search", async (req, res) => {
    try {
      const { q } = req.query;
      if (!q || typeof q !== 'string') {
        res.status(400).json({ error: "Query parameter 'q' is required" });
        return;
      }

      const apps = await storage.searchApps(q);
      res.json(apps);
    } catch (error) {
      res.status(500).json({ error: "Failed to search apps" });
    }
  });

  app.get("/api/apps", async (req, res) => {
    try {
      const apps = await storage.getAllApps();
      res.json(apps);
    } catch (error) {
      res.status(500).json({ error: "Failed to get apps" });
    }
  });

  const httpServer = createServer(app);

  return httpServer;
}
