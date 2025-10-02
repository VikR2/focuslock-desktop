import { QueryClient, QueryFunction } from "@tanstack/react-query";

// Platform detection state
let platformDetected = false;
let isTauriPlatform = false;

// Improved async platform detection
async function detectPlatform(): Promise<boolean> {
  if (platformDetected) {
    return isTauriPlatform;
  }

  try {
    // Try multiple detection methods
    const hasTauriGlobal = Boolean((window as any).__TAURI__);
    
    if (hasTauriGlobal) {
      // Verify Tauri is actually functional by trying to import
      try {
        await import(/* @vite-ignore */ '@tauri-apps/api/core');
        isTauriPlatform = true;
        console.log('[Platform] Detected: Tauri (Desktop Mode)');
      } catch (importError) {
        console.warn('[Platform] Tauri global found but import failed:', importError);
        isTauriPlatform = false;
      }
    } else {
      isTauriPlatform = false;
      console.log('[Platform] Detected: Web Mode');
    }
  } catch (error) {
    console.error('[Platform] Detection failed, defaulting to web mode:', error);
    isTauriPlatform = false;
  }

  platformDetected = true;
  return isTauriPlatform;
}

// Tauri invoke helper with better error handling
async function getTauriInvoke() {
  try {
    // Try to get invoke from global first (fastest)
    const tauriInvoke = (window as any).__TAURI__?.core?.invoke;
    if (tauriInvoke) {
      console.log('[Tauri] Using global __TAURI__.core.invoke');
      return tauriInvoke;
    }
    
    // Fallback to dynamic import
    console.log('[Tauri] Loading invoke via dynamic import');
    const { invoke } = await import(/* @vite-ignore */ '@tauri-apps/api/core');
    return invoke;
  } catch (error) {
    console.error('[Tauri] Failed to load invoke:', error);
    throw new Error('Tauri invoke not available - this should not happen in desktop mode');
  }
}

// Public Tauri command helper for direct invocations
export async function callTauriCommand<T>(command: string, args?: Record<string, any>): Promise<T> {
  const invoke = await getTauriInvoke();
  return invoke(command, args);
}

// Map API endpoints to Tauri commands
const endpointToCommand: Record<string, string> = {
  '/api/favorites': 'get_favorites',
  '/api/block-rules': 'get_block_rules',
  '/api/sessions': 'get_sessions',
  '/api/settings': 'get_settings',
};

async function throwIfResNotOk(res: Response) {
  if (!res.ok) {
    const text = (await res.text()) || res.statusText;
    throw new Error(`${res.status}: ${text}`);
  }
}

export async function apiRequest(
  method: string,
  url: string,
  data?: unknown | undefined,
): Promise<Response> {
  const isTauri = await detectPlatform();
  
  // Use Tauri commands when in desktop mode
  if (isTauri) {
    console.log(`[API] Tauri mode: ${method} ${url}`);
    const invoke = await getTauriInvoke();
    
    try {
      // Map HTTP operations to Tauri commands
      if (method === 'GET') {
        if (url === '/api/favorites') {
          console.log('[API] Invoking: get_favorites');
          const result = await invoke('get_favorites');
          return new Response(JSON.stringify(result), { status: 200 });
        }
        if (url === '/api/block-rules') {
          console.log('[API] Invoking: get_block_rules');
          const result = await invoke('get_block_rules');
          return new Response(JSON.stringify(result), { status: 200 });
        }
        if (url === '/api/sessions') {
          console.log('[API] Invoking: get_sessions');
          const result = await invoke('get_sessions');
          return new Response(JSON.stringify(result), { status: 200 });
        }
        if (url === '/api/settings') {
          console.log('[API] Invoking: get_settings');
          const result = await invoke('get_settings');
          return new Response(JSON.stringify(result), { status: 200 });
        }
        if (url.startsWith('/api/settings/')) {
          const key = url.split('/').pop();
          console.log(`[API] Invoking: get_settings (filtering by key: ${key})`);
          const result = await invoke('get_settings');
          const setting = (result as any[]).find((s: any) => s.key === key);
          return new Response(JSON.stringify(setting || null), { status: setting ? 200 : 404 });
        }
      } else if (method === 'POST') {
        if (url === '/api/favorites') {
          console.log('[API] Invoking: create_favorite');
          const result = await invoke('create_favorite', { favorite: data });
          return new Response(JSON.stringify(result), { status: 201 });
        }
        if (url === '/api/block-rules') {
          console.log('[API] Invoking: create_block_rule');
          const result = await invoke('create_block_rule', { rule: data });
          return new Response(JSON.stringify(result), { status: 201 });
        }
        if (url === '/api/sessions') {
          console.log('[API] Invoking: create_session');
          const result = await invoke('create_session', { session: data });
          return new Response(JSON.stringify(result), { status: 201 });
        }
        if (url === '/api/settings') {
          console.log('[API] Invoking: upsert_setting');
          const { key, value } = data as { key: string; value: string };
          const result = await invoke('upsert_setting', { key, value });
          return new Response(JSON.stringify(result), { status: 200 });
        }
      } else if (method === 'PATCH') {
        if (url.startsWith('/api/sessions/')) {
          const id = url.split('/').pop();
          console.log(`[API] Invoking: update_session (id: ${id})`);
          const result = await invoke('update_session', { id, updates: data });
          return new Response(JSON.stringify(result), { status: 200 });
        }
        if (url.startsWith('/api/block-rules/')) {
          const id = url.split('/').pop();
          console.log(`[API] Invoking: update_block_rule (id: ${id})`);
          const result = await invoke('update_block_rule', { id, updates: data });
          return new Response(JSON.stringify(result), { status: 200 });
        }
      } else if (method === 'DELETE') {
        if (url.startsWith('/api/favorites/')) {
          const id = url.split('/').pop();
          console.log(`[API] Invoking: delete_favorite (id: ${id})`);
          await invoke('delete_favorite', { id });
          return new Response(null, { status: 204 });
        }
        if (url.startsWith('/api/block-rules/')) {
          const id = url.split('/').pop();
          console.log(`[API] Invoking: delete_block_rule (id: ${id})`);
          await invoke('delete_block_rule', { id });
          return new Response(null, { status: 204 });
        }
      }
      
      // If we get here in Tauri mode, the endpoint is not handled
      const errorMsg = `Unhandled Tauri endpoint: ${method} ${url}`;
      console.error(`[API] ${errorMsg}`);
      throw new Error(errorMsg);
    } catch (error: any) {
      console.error('[API] Tauri command failed:', error);
      throw new Error(error?.message || 'Tauri command failed');
    }
  }

  // Fallback to HTTP fetch for web mode
  console.log(`[API] Web mode: ${method} ${url}`);
  const res = await fetch(url, {
    method,
    headers: data ? { "Content-Type": "application/json" } : {},
    body: data ? JSON.stringify(data) : undefined,
    credentials: "include",
  });

  await throwIfResNotOk(res);
  return res;
}

type UnauthorizedBehavior = "returnNull" | "throw";
export const getQueryFn: <T>(options: {
  on401: UnauthorizedBehavior;
}) => QueryFunction<T> =
  ({ on401: unauthorizedBehavior }) =>
  async ({ queryKey }) => {
    const isTauri = await detectPlatform();
    
    // Use Tauri commands when in desktop mode
    if (isTauri) {
      const invoke = await getTauriInvoke();
      const endpoint = queryKey[0] as string;
      
      console.log(`[Query] Tauri mode: ${endpoint}`);
      
      try {
        const command = endpointToCommand[endpoint];
        if (command) {
          console.log(`[Query] Invoking: ${command}`);
          return await invoke(command);
        }
        
        // If we get here in Tauri mode, the endpoint is not handled
        const errorMsg = `Unhandled Tauri endpoint in queryFn: ${endpoint}`;
        console.error(`[Query] ${errorMsg}`);
        throw new Error(errorMsg);
      } catch (error: any) {
        console.error('[Query] Tauri command failed:', error);
        throw new Error(error?.message || 'Tauri command failed');
      }
    }

    // Fallback to HTTP fetch for web mode
    const url = queryKey.join("/") as string;
    console.log(`[Query] Web mode: ${url}`);
    const res = await fetch(url, {
      credentials: "include",
    });

    if (unauthorizedBehavior === "returnNull" && res.status === 401) {
      return null;
    }

    await throwIfResNotOk(res);
    return await res.json();
  };

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      queryFn: getQueryFn({ on401: "throw" }),
      refetchInterval: false,
      refetchOnWindowFocus: false,
      staleTime: Infinity,
      retry: (failureCount, error: any) => {
        // Don't retry on fetch errors in desktop mode (indicates missing endpoint mapping)
        if (error?.message?.includes('failed to fetch') || 
            error?.message?.includes('Unhandled Tauri endpoint')) {
          console.error('[Query] Not retrying - endpoint issue:', error.message);
          return false;
        }
        // Retry up to 2 times for other errors
        return failureCount < 2;
      },
    },
    mutations: {
      retry: false,
    },
  },
});
