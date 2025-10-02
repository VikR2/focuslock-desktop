import { createContext, useContext, useEffect, useState, ReactNode } from 'react';

export interface LogEntry {
  timestamp: string;
  type: 'log' | 'warn' | 'error' | 'info';
  message: string;
}

interface LogsContextType {
  logs: LogEntry[];
  clearLogs: () => void;
}

const LogsContext = createContext<LogsContextType | undefined>(undefined);

export function LogsProvider({ children }: { children: ReactNode }) {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  
  useEffect(() => {
    // Store original console methods
    const originalLog = console.log;
    const originalWarn = console.warn;
    const originalError = console.error;
    const originalInfo = console.info;
    
    // Override console methods to capture logs globally
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
  
  const clearLogs = () => {
    setLogs([]);
  };
  
  return (
    <LogsContext.Provider value={{ logs, clearLogs }}>
      {children}
    </LogsContext.Provider>
  );
}

export function useLogs() {
  const context = useContext(LogsContext);
  if (context === undefined) {
    throw new Error('useLogs must be used within a LogsProvider');
  }
  return context;
}
