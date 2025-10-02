import { useEffect, useState, useRef } from 'react';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { X, ChevronLeft, ChevronRight, Trash2 } from 'lucide-react';

interface LogEntry {
  timestamp: string;
  type: 'log' | 'warn' | 'error' | 'info';
  message: string;
}

export function DebugPanel() {
  const [isOpen, setIsOpen] = useState(true);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const scrollRef = useRef<HTMLDivElement>(null);
  
  useEffect(() => {
    // Store original console methods
    const originalLog = console.log;
    const originalWarn = console.warn;
    const originalError = console.error;
    const originalInfo = console.info;
    
    // Override console methods to capture logs
    console.log = (...args: any[]) => {
      const message = args.map(arg => 
        typeof arg === 'object' ? JSON.stringify(arg, null, 2) : String(arg)
      ).join(' ');
      
      setLogs(prev => [...prev, {
        timestamp: new Date().toLocaleTimeString(),
        type: 'log',
        message
      }]);
      
      originalLog.apply(console, args);
    };
    
    console.warn = (...args: any[]) => {
      const message = args.map(arg => 
        typeof arg === 'object' ? JSON.stringify(arg, null, 2) : String(arg)
      ).join(' ');
      
      setLogs(prev => [...prev, {
        timestamp: new Date().toLocaleTimeString(),
        type: 'warn',
        message
      }]);
      
      originalWarn.apply(console, args);
    };
    
    console.error = (...args: any[]) => {
      const message = args.map(arg => 
        typeof arg === 'object' ? JSON.stringify(arg, null, 2) : String(arg)
      ).join(' ');
      
      setLogs(prev => [...prev, {
        timestamp: new Date().toLocaleTimeString(),
        type: 'error',
        message
      }]);
      
      originalError.apply(console, args);
    };
    
    console.info = (...args: any[]) => {
      const message = args.map(arg => 
        typeof arg === 'object' ? JSON.stringify(arg, null, 2) : String(arg)
      ).join(' ');
      
      setLogs(prev => [...prev, {
        timestamp: new Date().toLocaleTimeString(),
        type: 'info',
        message
      }]);
      
      originalInfo.apply(console, args);
    };
    
    // Cleanup on unmount
    return () => {
      console.log = originalLog;
      console.warn = originalWarn;
      console.error = originalError;
      console.info = originalInfo;
    };
  }, []);
  
  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [logs]);
  
  const clearLogs = () => {
    setLogs([]);
  };
  
  const getLogColor = (type: LogEntry['type']) => {
    switch (type) {
      case 'error':
        return 'text-red-400';
      case 'warn':
        return 'text-yellow-400';
      case 'info':
        return 'text-blue-400';
      default:
        return 'text-foreground';
    }
  };
  
  if (!isOpen) {
    return (
      <Button
        variant="outline"
        size="icon"
        onClick={() => setIsOpen(true)}
        className="fixed left-4 bottom-4 z-50"
        data-testid="button-open-debug"
      >
        <ChevronRight className="h-4 w-4" />
      </Button>
    );
  }
  
  return (
    <div className="fixed left-0 top-0 h-screen w-80 bg-card border-r border-border flex flex-col z-40">
      <div className="flex items-center justify-between p-3 border-b border-border">
        <h3 className="font-semibold text-sm">Debug Logs</h3>
        <div className="flex gap-1">
          <Button
            variant="ghost"
            size="icon"
            onClick={clearLogs}
            className="h-7 w-7"
            data-testid="button-clear-logs"
          >
            <Trash2 className="h-3 w-3" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setIsOpen(false)}
            className="h-7 w-7"
            data-testid="button-close-debug"
          >
            <ChevronLeft className="h-3 w-3" />
          </Button>
        </div>
      </div>
      
      <ScrollArea className="flex-1">
        <div ref={scrollRef} className="p-2 space-y-1 font-mono text-xs">
          {logs.length === 0 ? (
            <div className="text-muted-foreground p-2">No logs yet...</div>
          ) : (
            logs.map((log, index) => (
              <div key={index} className="p-1 rounded hover-elevate">
                <div className="flex items-start gap-2">
                  <span className="text-muted-foreground shrink-0">{log.timestamp}</span>
                  <span className={getLogColor(log.type) + " break-all"}>{log.message}</span>
                </div>
              </div>
            ))
          )}
        </div>
      </ScrollArea>
      
      <div className="p-2 border-t border-border text-xs text-muted-foreground">
        {logs.length} log{logs.length !== 1 ? 's' : ''}
      </div>
    </div>
  );
}
