import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useParams, useNavigate } from 'react-router-dom';
import {
  Mail, Inbox as InboxIcon, Send, Archive, Clock,
  Search, Filter, RefreshCw, Settings,
  Tag, User, Edit3, BarChart2
} from 'lucide-react';
import { emailApi, inboxApi } from '../api';
import { formatDistanceToNow } from 'date-fns';
import type { Email, EmailState } from '../types';
import clsx from 'clsx';
import ComposeEmailModal from '../components/ComposeEmailModal';

export default function InboxPage() {
  const { inboxId } = useParams<{ inboxId: string }>();
  const navigate   = useNavigate();

  const [searchQuery,    setSearchQuery]    = useState('');
  const [selectedState,  setSelectedState]  = useState<EmailState | undefined>();
  const [selectedEmail,  setSelectedEmail]  = useState<string | null>(null);
  const [isComposeOpen,  setIsComposeOpen]  = useState(false);          // ← compose toggle

  // ── data ──────────────────────────────────────────────────────────────
  const { data: inbox } = useQuery({
    queryKey: ['inbox', inboxId],
    queryFn:  () => (inboxId ? inboxApi.get(inboxId) : null),
    enabled:  !!inboxId,
  });

  const { data: emailsData, isLoading, refetch } = useQuery({
    queryKey: ['emails', inboxId, searchQuery, selectedState],
    queryFn:  () => emailApi.list({
      inbox_id:  inboxId,
      state:     selectedState,
      search:    searchQuery || undefined,
      page:      1,
      page_size: 50,
    }),
  });

  // ── handlers ──────────────────────────────────────────────────────────
  const handleEmailClick = (id: string) => {
    setSelectedEmail(id);
    navigate(`/email/${id}`);
  };

  const getStateColor = (state: EmailState) => ({
    NEW:       'text-blue-400 bg-blue-900/20',
    CLAIMED:   'text-yellow-400 bg-yellow-900/20',
    RESPONDED: 'text-purple-400 bg-purple-900/20',
    RESOLVED:  'text-green-400 bg-green-900/20',
    ARCHIVED:  'text-gray-400 bg-gray-900/20',
  })[state];

  // ── render ────────────────────────────────────────────────────────────
  return (
    <div className="flex h-screen bg-blackhole-950">

      {/* ── sidebar ── */}
      <div className="w-64 bg-blackhole-900 border-r border-blackhole-800 flex flex-col">
        <div className="p-4 border-b border-blackhole-800">
          <div className="flex items-center space-x-2">
            <div className="w-8 h-8 bg-gradient-to-br from-blue-600 to-purple-600 rounded-lg flex items-center justify-center">
              <span className="text-white font-bold text-sm">🕳️</span>
            </div>
            <h1 className="text-xl font-bold text-white">Blackhole</h1>
          </div>
        </div>

        <nav className="flex-1 overflow-y-auto p-2">
          <div className="space-y-1">
            <SidebarItem icon={Mail}        label="All Mail"  badge={emailsData?.total} onClick={() => navigate('/inbox/00000000-0000-0000-0000-000000000003')} />
            <SidebarItem icon={InboxIcon}   label="Inbox"     active={!selectedState} onClick={() => { setSelectedState(undefined); }} />
            <SidebarItem icon={Send}        label="Sent"      onClick={() => { setSelectedState(undefined); }} />
            <SidebarItem icon={Archive}     label="Archive"   onClick={() => setSelectedState('ARCHIVED')} />
            <SidebarItem icon={Clock}       label="Snoozed"   onClick={() => { setSelectedState(undefined); }} />
          </div>

          <div className="mt-6">
            <h3 className="px-3 text-xs font-semibold text-gray-400 uppercase tracking-wider">Inboxes</h3>
            <div className="mt-2 space-y-1">
              <InboxItem name="Support" color="#3B82F6" count={12} onClick={() => navigate('/inbox/00000000-0000-0000-0000-000000000003')} />
              <InboxItem name="Sales"   color="#10B981" count={5}  onClick={() => navigate('/inbox/00000000-0000-0000-0000-000000000004')} />
              <InboxItem name="Info"    color="#F59E0B" count={3}  onClick={() => navigate('/inbox/00000000-0000-0000-0000-000000000005')} />
            </div>
          </div>

          <div className="mt-6 border-t border-blackhole-800 pt-3">
            <h3 className="px-3 text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">Tools</h3>
            <div className="space-y-1">
              <SidebarItem icon={Settings}    label="Rules"      onClick={() => navigate('/rules')} />
              <SidebarItem icon={BarChart2}   label="API & Stats" onClick={() => navigate('/api')} />
            </div>
          </div>
        </nav>

        <div className="p-4 border-t border-blackhole-800">
          <button onClick={() => navigate('/settings')} className="w-full flex items-center space-x-2 p-2 rounded-lg hover:bg-blackhole-800 transition-colors">
            <div className="w-8 h-8 bg-blue-600 rounded-full flex items-center justify-center">
              <User size={16} />
            </div>
            <div className="flex-1 text-left">
              <div className="text-sm font-medium">Admin User</div>
              <div className="text-xs text-gray-400">admin@blackhole.dev</div>
            </div>
            <Settings size={16} className="text-gray-400" />
          </button>
        </div>
      </div>

      {/* ── main content ── */}
      <div className="flex-1 flex flex-col min-w-0">

        {/* header */}
        <div className="h-16 bg-blackhole-900 border-b border-blackhole-800 flex items-center justify-between px-6 shrink-0">
          <div className="flex items-center space-x-3">
            <h2 className="text-xl font-semibold text-white">{inbox?.name || 'Inbox'}</h2>
            <span className="text-sm text-gray-400">{emailsData?.total || 0} emails</span>
          </div>

          {/* action buttons row */}
          <div className="flex items-center space-x-2">
            {/* ── COMPOSE BUTTON ── */}
            <button
              onClick={() => setIsComposeOpen(true)}
              className="flex items-center gap-1.5 px-4 py-1.5 bg-blue-600 hover:bg-blue-500 text-white text-sm font-medium rounded-lg shadow-lg shadow-blue-900/40 transition-colors"
            >
              <Edit3 size={15} /> Compose
            </button>

            <button onClick={() => refetch()} className="p-2 hover:bg-blackhole-800 rounded-lg transition-colors">
              <RefreshCw size={18} className="text-gray-400" />
            </button>
            <button className="p-2 hover:bg-blackhole-800 rounded-lg transition-colors">
              <Filter size={18} className="text-gray-400" />
            </button>
          </div>
        </div>

        {/* search + filters */}
        <div className="p-4 bg-blackhole-900 border-b border-blackhole-800 shrink-0">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" size={18} />
            <input
              type="text"
              placeholder="Search emails…"
              value={searchQuery}
              onChange={e => setSearchQuery(e.target.value)}
              className="w-full pl-10 pr-4 py-2 bg-blackhole-800 border border-blackhole-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-600 text-white placeholder-gray-400"
            />
          </div>

          <div className="mt-3 flex space-x-2">
            {(['NEW', 'CLAIMED', 'RESPONDED', 'RESOLVED'] as EmailState[]).map(state => (
              <button
                key={state}
                onClick={() => setSelectedState(selectedState === state ? undefined : state)}
                className={clsx(
                  'px-3 py-1 rounded-full text-xs font-medium transition-colors',
                  selectedState === state ? getStateColor(state) : 'text-gray-400 bg-blackhole-800 hover:bg-blackhole-700'
                )}
              >
                {state}
              </button>
            ))}
          </div>
        </div>

        {/* email list */}
        <div className="flex-1 overflow-y-auto">
          {isLoading ? (
            <div className="flex items-center justify-center h-64">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
            </div>
          ) : !emailsData?.emails.length ? (
            <div className="flex flex-col items-center justify-center h-64 text-gray-400">
              <Mail size={48} className="mb-4 opacity-50" />
              <p className="text-lg">No emails found</p>
            </div>
          ) : (
            <div className="divide-y divide-blackhole-800">
              {emailsData.emails.map(email => (
                <EmailListItem
                  key={email.id}
                  email={email}
                  isSelected={selectedEmail === email.id}
                  onClick={() => handleEmailClick(email.id)}
                />
              ))}
            </div>
          )}
        </div>
      </div>

      {/* ── Compose modal (rendered at root so it floats above everything) ── */}
      <ComposeEmailModal isOpen={isComposeOpen} onClose={() => setIsComposeOpen(false)} />
    </div>
  );
}

// ─── sub-components ─────────────────────────────────────────────────────────

function SidebarItem({ icon: Icon, label, badge, active = false, onClick }: {
  icon: React.FC<{ size?: number; className?: string }>; label: string; badge?: number; active?: boolean; onClick?: () => void;
}) {
  return (
    <button onClick={onClick} className={clsx(
      'w-full flex items-center space-x-3 px-3 py-2 rounded-lg transition-colors',
      active ? 'bg-blue-600 text-white' : 'text-gray-300 hover:bg-blackhole-800'
    )}>
      <Icon size={18} />
      <span className="flex-1 text-left text-sm font-medium">{label}</span>
      {badge !== undefined && (
        <span className="text-xs px-2 py-0.5 rounded-full bg-blackhole-800 text-gray-300">{badge}</span>
      )}
    </button>
  );
}

function InboxItem({ name, color, count, onClick }: { name: string; color: string; count: number; onClick?: () => void }) {
  return (
    <button onClick={onClick} className="w-full flex items-center space-x-3 px-3 py-2 rounded-lg hover:bg-blackhole-800 transition-colors text-gray-300">
      <div className="w-3 h-3 rounded-full" style={{ backgroundColor: color }} />
      <span className="flex-1 text-left text-sm font-medium">{name}</span>
      <span className="text-xs text-gray-400">{count}</span>
    </button>
  );
}

function EmailListItem({ email, isSelected, onClick }: { email: Email; isSelected: boolean; onClick: () => void }) {
  return (
    <div onClick={onClick}
      className={clsx('p-4 cursor-pointer transition-colors hover:bg-blackhole-800',
        isSelected && 'bg-blackhole-800 border-l-4 border-blue-600'
      )}
    >
      <div className="flex items-start justify-between mb-1">
        <div className="flex-1 min-w-0">
          <div className="flex items-center space-x-2 mb-0.5">
            <span className="font-medium text-white truncate">{email.from_name || email.from_address}</span>
            <span className={clsx('state-badge', `state-${email.state}`)}>{email.state}</span>
          </div>
          <h3 className="text-sm font-semibold text-white truncate mb-0.5">{email.subject}</h3>
          <p className="text-sm text-gray-400 truncate">{email.body_text?.substring(0, 100)}</p>
        </div>
        <div className="ml-4 flex flex-col items-end space-y-1 shrink-0">
          <span className="text-xs text-gray-400">
            {formatDistanceToNow(new Date(email.received_at), { addSuffix: true })}
          </span>
          {email.delivery_latency_ms != null && (
            <span className="text-xs text-green-400 font-medium">{email.delivery_latency_ms} ms</span>
          )}
        </div>
      </div>

      {email.tags.length > 0 && (
        <div className="flex items-center space-x-1 mt-1.5">
          <Tag size={12} className="text-gray-400" />
          {email.tags.map(tag => (
            <span key={tag} className="text-xs px-2 py-0.5 rounded-full bg-blackhole-700 text-gray-300">{tag}</span>
          ))}
        </div>
      )}
    </div>
  );
}
