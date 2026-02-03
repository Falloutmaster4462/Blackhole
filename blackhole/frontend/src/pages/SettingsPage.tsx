import { useNavigate } from 'react-router-dom';
import { Settings, BarChart2, Mail, Inbox as InboxIcon, Send, Archive, Clock, User } from 'lucide-react';

export default function SettingsPage() {
  const navigate = useNavigate();

  return (
    <div className="flex h-screen bg-blackhole-950">
      {/* ── shared sidebar ── */}
      <Sidebar navigate={navigate} />

      {/* ── main ── */}
      <div className="flex-1 overflow-y-auto p-8">
        <div className="max-w-3xl mx-auto">
          <h1 className="text-3xl font-bold text-white mb-8">Settings</h1>

          {/* Account */}
          <section className="card mb-6">
            <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
              <User size={18} className="text-blue-400" /> Account
            </h2>
            <div className="space-y-3">
              <SettingRow label="Name"  value="Admin User" />
              <SettingRow label="Email" value="admin@blackhole.dev" />
              <SettingRow label="Role"  value="ADMIN" />
            </div>
          </section>

          {/* Inboxes */}
          <section className="card mb-6">
            <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
              <InboxIcon size={18} className="text-blue-400" /> Inboxes
            </h2>
            <div className="space-y-2">
              <InboxRow name="Support" addr="support@blackhole.dev" color="#3B82F6" />
              <InboxRow name="Sales"   addr="sales@blackhole.dev"   color="#10B981" />
              <InboxRow name="Info"    addr="info@blackhole.dev"    color="#F59E0B" />
            </div>
          </section>

          {/* SMTP */}
          <section className="card mb-6">
            <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
              <Mail size={18} className="text-blue-400" /> SMTP Server
            </h2>
            <p className="text-sm text-gray-400 mb-3">
              Blackhole exposes a local SMTP endpoint you can point any mail client at.
            </p>
            <div className="bg-blackhole-800 rounded-lg p-3 font-mono text-sm text-green-300">
              <span className="text-gray-500">Host:</span> localhost&nbsp;&nbsp;
              <span className="text-gray-500">Port:</span> 2525&nbsp;&nbsp;
              <span className="text-gray-500">TLS:</span> none
            </div>
          </section>

          {/* Resend */}
          <section className="card">
            <h2 className="text-lg font-semibold text-white mb-2 flex items-center gap-2">
              <Send size={18} className="text-blue-400" /> Resend Integration
            </h2>
            <p className="text-sm text-gray-400">
              Set <code className="text-blue-300 bg-blackhole-800 px-1.5 py-0.5 rounded">RESEND_API_KEY</code> in your
              <code className="text-blue-300 bg-blackhole-800 px-1.5 py-0.5 rounded ml-1">.env</code> to enable real outbound delivery.
              Without it, composed emails are stored locally only.
            </p>
          </section>
        </div>
      </div>
    </div>
  );
}

function SettingRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between py-2 border-b border-blackhole-800 last:border-0">
      <span className="text-sm text-gray-400">{label}</span>
      <span className="text-sm font-medium text-white">{value}</span>
    </div>
  );
}

function InboxRow({ name, addr, color }: { name: string; addr: string; color: string }) {
  return (
    <div className="flex items-center gap-3 py-2 border-b border-blackhole-800 last:border-0">
      <div className="w-3 h-3 rounded-full shrink-0" style={{ backgroundColor: color }} />
      <span className="text-sm font-medium text-white flex-1">{name}</span>
      <span className="text-xs text-gray-500 font-mono">{addr}</span>
    </div>
  );
}

/* ── shared sidebar ── */
function Sidebar({ navigate }: { navigate: (to: string) => void }) {
  return (
    <div className="w-64 bg-blackhole-900 border-r border-blackhole-800 flex flex-col shrink-0">
      <div className="p-4 border-b border-blackhole-800">
        <div className="flex items-center space-x-2 cursor-pointer" onClick={() => navigate('/')}>
          <div className="w-8 h-8 bg-gradient-to-br from-blue-600 to-purple-600 rounded-lg flex items-center justify-center">
            <span className="text-white font-bold text-sm">🕳️</span>
          </div>
          <h1 className="text-xl font-bold text-white">Blackhole</h1>
        </div>
      </div>
      <nav className="flex-1 overflow-y-auto p-2">
        <div className="space-y-1">
          <NavBtn icon={Mail}      label="All Mail"   onClick={() => navigate('/')} />
          <NavBtn icon={InboxIcon} label="Inbox"      onClick={() => navigate('/')} />
          <NavBtn icon={Send}      label="Sent"       onClick={() => navigate('/')} />
          <NavBtn icon={Archive}   label="Archive"    onClick={() => navigate('/')} />
          <NavBtn icon={Clock}     label="Snoozed"    onClick={() => navigate('/')} />
        </div>
        <div className="mt-6">
          <h3 className="px-3 text-xs font-semibold text-gray-400 uppercase tracking-wider">Inboxes</h3>
          <div className="mt-2 space-y-1">
            <NavBtn icon={null} label="Support" onClick={() => navigate('/inbox/00000000-0000-0000-0000-000000000003')} dot="#3B82F6" />
            <NavBtn icon={null} label="Sales"   onClick={() => navigate('/inbox/00000000-0000-0000-0000-000000000004')} dot="#10B981" />
            <NavBtn icon={null} label="Info"    onClick={() => navigate('/inbox/00000000-0000-0000-0000-000000000005')} dot="#F59E0B" />
          </div>
        </div>
        <div className="mt-6 border-t border-blackhole-800 pt-3">
          <h3 className="px-3 text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">Tools</h3>
          <div className="space-y-1">
            <NavBtn icon={Settings}  label="Rules"       onClick={() => navigate('/rules')} />
            <NavBtn icon={BarChart2} label="API & Stats" onClick={() => navigate('/api')} />
          </div>
        </div>
      </nav>
      <div className="p-4 border-t border-blackhole-800">
        <button onClick={() => navigate('/settings')} className="w-full flex items-center space-x-2 p-2 rounded-lg hover:bg-blackhole-800 transition-colors">
          <div className="w-8 h-8 bg-blue-600 rounded-full flex items-center justify-center"><User size={16} /></div>
          <div className="flex-1 text-left">
            <div className="text-sm font-medium text-white">Admin User</div>
            <div className="text-xs text-gray-400">admin@blackhole.dev</div>
          </div>
          <Settings size={16} className="text-gray-400" />
        </button>
      </div>
    </div>
  );
}

function NavBtn({ icon: Icon, label, onClick, active, dot }: {
  icon: any; label: string; onClick: () => void; active?: boolean; dot?: string;
}) {
  return (
    <button onClick={onClick} className={`w-full flex items-center space-x-3 px-3 py-2 rounded-lg transition-colors ${active ? 'bg-blue-600 text-white' : 'text-gray-300 hover:bg-blackhole-800'}`}>
      {dot ? <div className="w-3 h-3 rounded-full" style={{ backgroundColor: dot }} /> : Icon ? <Icon size={18} /> : null}
      <span className="flex-1 text-left text-sm font-medium">{label}</span>
    </button>
  );
}
