import { useState } from "react";
import RulesTable from '../RulesTable';
import type { BlockRule, BlockMode } from "@shared/schema";

export default function RulesTableExample() {
  const [rules, setRules] = useState<BlockRule[]>([
    {
      id: '1',
      appId: 'discord.exe',
      matchKind: 'exe',
      mode: 'hard',
    },
    {
      id: '2',
      appId: 'chrome.exe',
      matchKind: 'exe',
      mode: 'soft',
    },
    {
      id: '3',
      appId: 'steam.exe',
      matchKind: 'exe',
      mode: 'hard',
    },
  ]);

  const handleDeleteRule = (id: string) => {
    setRules(prev => prev.filter(rule => rule.id !== id));
    console.log(`Deleted rule ${id}`);
  };

  const handleEditRule = (rule: BlockRule) => {
    console.log(`Edit rule for ${rule.appId}`);
  };

  const handleToggleMode = (id: string, mode: BlockMode) => {
    setRules(prev => prev.map(rule => 
      rule.id === id ? { ...rule, mode } : rule
    ));
    console.log(`Toggled mode for rule ${id} to ${mode}`);
  };

  return (
    <div className="p-4 max-w-3xl">
      <RulesTable 
        rules={rules}
        onDeleteRule={handleDeleteRule}
        onEditRule={handleEditRule}
        onToggleMode={handleToggleMode}
      />
    </div>
  );
}