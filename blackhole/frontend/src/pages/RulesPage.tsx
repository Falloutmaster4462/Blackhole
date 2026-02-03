import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { ruleApi } from '../api';
import { Settings, BarChart2, Plus, Trash2, ToggleLeft, ToggleRight } from 'lucide-react';
import { Mail, Inbox as InboxIcon, Send, Archive, Clock, User } from 'lucide-react';
import toast from 'react-hot-toast';
import type { Rule } from '../types';

export default function RulesPage() {
  const navigate    = useNavigate();
  const queryClient = useQueryClient();

  const { data: rules, isLoading } = useQuery<Rule[]>({
    queryKey: ['rules'],
    queryFn:  ruleApi.list,
  });

  const toggleMut = useMutation({
    mutationFn: (id: string) => ruleApi.toggle(id),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ['rules'] }); },
    onError:   () => toast.error('Failed to toggle rule'),
  });

  const deleteMut = useMutation({
    mutationFn: (id: string) => ruleApi.delete(id),
    onSuccess: () => { queryClient.invalidateQueries({ queryKey: ['rules'] }); toast.success('Rule deleted'); },
    onError:   () => toast.error('Failed to delete rule'),
  });

  return (
    <div className="flex h-screen bg-blackhole-950">
      {/* ── shared sidebar ── */}
      <Sidebar navigate={navigate} />

      {/* ── main ── */}
      <div className="flex-1 overflow-y-auto p-8">
        <div className="max-w-4xl mx-auto">
          <div className="flex items-center justify-between mb-8">
            <h1 className="text-3xl font-bold text-white">Rules & Automation</h1>
            <button className="flex items-center gap-1.5 px-4 py-1.5 bg-blue-600 hover:bg-blue-500 text-white text-sm font-medium rounded-lg transition-colors">
              <Plus size={16} /> New Rule
            </button>
          </div>

          {isLoading ? (
            <div className="flex items-center justify-center h-64">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
            </div>
          ) : !rules?.length ? (
            <div className="card text-center py-16">
              <p className="text-gray-400 text-lg">No rules yet</p>
              <p className="text-gray-500 text-sm mt-1">Create a rule to automate email routing, tagging, and SLA tracking.</p>
            </div>
          ) : (
            <div className="space-y-3">
              {rules.map(rule => (
                <div key={rule.id} className="card flex items-start justify-between gap-4">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <h3 className="font-semibold text-white truncate">{rule.name}</h3>
                      <span className="text-xs px-2 py-0.5 rounded-full bg-blackhole-800 text-gray-400">
                        Priority {rule.priority}
                      </span>
                    </div>
                    {rule.description && <p className="text-sm text-gray-400">{rule.description}</p>}
                    <div className="mt-2 flex flex-wrap gap-2 text-xs">
                      <span className="px-2 py-0.5 rounded bg-blue-900/40 text-blue-300">
                        Conditions: {JSON.stringify(rule.conditions)}
                      </span>
                      <span className="px-2 py-0.5 rounded bg-green-900/40 text-green-300">
                        Actions: {JSON.stringify(rule.actions)}
                      </span>
                    </div>
                  </div>
                  <div className="flex items-center gap-2 shrink-0">
                    <button
                      onClick={() => toggleMut.mutate(rule.id)}
                      className="p-1.5 rounded hover:bg-blackhole-800 transition-colors"
                      title={rule.is_active ? 'Disable' : 'Enable'}
                    >
                      {rule.is_active
                        ? <ToggleRight size={22} className="text-green-400" />
                        : <ToggleLeft  size={22} className="text-gray-500" />
                      }
                    </button>
                    <button
                      onClick={() => deleteMut.mutate(rule.id)}
                      className="p-1.5 rounded hover:bg-blackhole-800 text-gray-500 hover:text-red-400 transition-colors"
                    >
                      <Trash2 size={18} />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

/* ── tiny shared sidebar (duplicated here to avoid a big refactor) ── */
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
            <NavBtn icon={Settings}  label="Rules"       onClick={() => navigate('/rules')}      active />
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
