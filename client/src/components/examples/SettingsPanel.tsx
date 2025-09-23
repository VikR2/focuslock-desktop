import { useState } from "react";
import SettingsPanel from '../SettingsPanel';

export default function SettingsPanelExample() {
  const [settings, setSettings] = useState({
    strictMode: false,
    autostart: true,
    notificationCadence: 'normal',
    defaultBlockMode: 'soft' as 'hard' | 'soft',
    hotkeysEnabled: true,
  });

  const handleSettingChange = (key: string, value: any) => {
    setSettings(prev => ({ ...prev, [key]: value }));
    console.log(`Setting ${key} changed to:`, value);
  };

  const handleSave = () => {
    console.log('Settings saved:', settings);
  };

  return (
    <div className="p-4">
      <SettingsPanel 
        settings={settings}
        onSettingChange={handleSettingChange}
        onSave={handleSave}
      />
    </div>
  );
}