import { useEffect, useRef } from 'react';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Trash2, ScrollText } from 'lucide-react';
import { useLogs, type LogEntry } from '@/contexts/LogsContext';

export default function Logs() {
  const { logs, clearLogs } = useLogs();
  const scrollRef = useRef<HTMLDivElement>(null);
  
  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [logs]);
  
  const getLogColor = (type: LogEntry['type']) => {
    switch (type) {
      case 'error':
        return 'text-red-500 dark:text-red-400';
      case 'warn':
        return 'text-yellow-600 dark:text-yellow-400';
      case 'info':
        return 'text-blue-600 dark:text-blue-400';
      default:
        return 'text-foreground';
    }
  };
  
  const getLogBadge = (type: LogEntry['type']) => {
    switch (type) {
      case 'error':
        return 'bg-red-500/10 text-red-500 dark:text-red-400';
      case 'warn':
        return 'bg-yellow-500/10 text-yellow-600 dark:text-yellow-400';
      case 'info':
        return 'bg-blue-500/10 text-blue-600 dark:text-blue-400';
      default:
        return 'bg-muted text-muted-foreground';
    }
  };
  
  return (
    <div className="flex-1 p-6">
      <div className="max-w-6xl mx-auto">
        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <CardTitle className="flex items-center gap-2">
                <ScrollText className="w-5 h-5" />
                Application Logs
              </CardTitle>
              <div className="flex items-center gap-3">
                <span className="text-sm text-muted-foreground" data-testid="text-log-count">
                  {logs.length} log{logs.length !== 1 ? 's' : ''}
                </span>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={clearLogs}
                  data-testid="button-clear-logs"
                >
                  <Trash2 className="w-4 h-4 mr-2" />
                  Clear Logs
                </Button>
              </div>
            </div>
          </CardHeader>
          <CardContent>
            <ScrollArea className="h-[600px] w-full rounded-md border">
              <div ref={scrollRef} className="p-4 space-y-2 font-mono text-sm">
                {logs.length === 0 ? (
                  <div className="text-center text-muted-foreground py-12" data-testid="text-no-logs">
                    <ScrollText className="w-12 h-12 mx-auto mb-3 opacity-50" />
                    <p>No logs yet</p>
                    <p className="text-xs mt-1">Application activity will appear here</p>
                  </div>
                ) : (
                  logs.map((log, index) => (
                    <div 
                      key={index} 
                      className="p-3 rounded-md hover-elevate border border-border/50"
                      data-testid={`log-entry-${index}`}
                    >
                      <div className="flex items-start gap-3">
                        <span 
                          className={`px-2 py-0.5 rounded text-xs font-medium ${getLogBadge(log.type)}`}
                          data-testid={`badge-log-type-${log.type}`}
                        >
                          {log.type.toUpperCase()}
                        </span>
                        <span 
                          className="text-muted-foreground text-xs shrink-0 mt-0.5"
                          data-testid={`text-log-timestamp-${index}`}
                        >
                          {log.timestamp}
                        </span>
                        <span 
                          className={`${getLogColor(log.type)} break-all flex-1`}
                          data-testid={`text-log-message-${index}`}
                        >
                          {log.message}
                        </span>
                      </div>
                    </div>
                  ))
                )}
              </div>
            </ScrollArea>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
