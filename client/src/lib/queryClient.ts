import { QueryClient, QueryFunction } from "@tanstack/react-query";

// Detect if running in Tauri desktop app
const isTauri = Boolean((window as any).__TAURI__);

// Tauri invoke helper
async function getTauriInvoke() {
  try {
    const tauriInvoke = (window as any).__TAURI__?.core?.invoke;
    if (tauriInvoke) return tauriInvoke;
    const { invoke } = await import(/* @vite-ignore */ '@tauri-apps/api/core');
    return invoke;
  } catch (error) {
    console.error('Failed to load Tauri invoke:', error);
    throw new Error('Tauri invoke not available');
  }
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
  // Use Tauri commands when in desktop mode
  if (isTauri) {
    const invoke = await getTauriInvoke();
    
    try {
      // Map HTTP operations to Tauri commands
      if (method === 'GET') {
        if (url === '/api/favorites') {
          const result = await invoke('get_favorites');
          return new Response(JSON.stringify(result), { status: 200 });
        }
        if (url === '/api/block-rules') {
          const result = await invoke('get_block_rules');
          return new Response(JSON.stringify(result), { status: 200 });
        }
        if (url === '/api/sessions') {
          const result = await invoke('get_sessions');
          return new Response(JSON.stringify(result), { status: 200 });
        }
        if (url === '/api/settings') {
          const result = await invoke('get_settings');
          return new Response(JSON.stringify(result), { status: 200 });
        }
        if (url.startsWith('/api/settings/')) {
          const key = url.split('/').pop();
          const result = await invoke('get_settings');
          const setting = (result as any[]).find((s: any) => s.key === key);
          return new Response(JSON.stringify(setting || null), { status: setting ? 200 : 404 });
        }
      } else if (method === 'POST') {
        if (url === '/api/favorites') {
          const result = await invoke('create_favorite', { favorite: data });
          return new Response(JSON.stringify(result), { status: 201 });
        }
        if (url === '/api/block-rules') {
          const result = await invoke('create_block_rule', { rule: data });
          return new Response(JSON.stringify(result), { status: 201 });
        }
        if (url === '/api/sessions') {
          const result = await invoke('create_session', { session: data });
          return new Response(JSON.stringify(result), { status: 201 });
        }
        if (url === '/api/settings') {
          const { key, value } = data as { key: string; value: string };
          const result = await invoke('upsert_setting', { key, value });
          return new Response(JSON.stringify(result), { status: 200 });
        }
      } else if (method === 'PATCH') {
        if (url.startsWith('/api/sessions/')) {
          const id = url.split('/').pop();
          const result = await invoke('update_session', { id, updates: data });
          return new Response(JSON.stringify(result), { status: 200 });
        }
      } else if (method === 'DELETE') {
        if (url.startsWith('/api/favorites/')) {
          const id = url.split('/').pop();
          await invoke('delete_favorite', { id });
          return new Response(null, { status: 204 });
        }
        if (url.startsWith('/api/block-rules/')) {
          const id = url.split('/').pop();
          await invoke('delete_block_rule', { id });
          return new Response(null, { status: 204 });
        }
      }
    } catch (error: any) {
      throw new Error(error?.message || 'Tauri command failed');
    }
  }

  // Fallback to HTTP fetch for web mode
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
    // Use Tauri commands when in desktop mode
    if (isTauri) {
      const invoke = await getTauriInvoke();
      const endpoint = queryKey[0] as string;
      
      try {
        const command = endpointToCommand[endpoint];
        if (command) {
          return await invoke(command);
        }
      } catch (error: any) {
        throw new Error(error?.message || 'Tauri command failed');
      }
    }

    // Fallback to HTTP fetch for web mode
    const res = await fetch(queryKey.join("/") as string, {
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
      retry: false,
    },
    mutations: {
      retry: false,
    },
  },
});
