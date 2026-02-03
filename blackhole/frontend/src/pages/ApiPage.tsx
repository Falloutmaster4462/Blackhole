import { useQuery } from '@tanstack/react-query';
import { statsApi } from '../api';
import {
  Mail, Clock, CheckCircle, AlertCircle,
  Activity, Zap, ArrowUpCircle, ArrowDownCircle,
} from 'lucide-react';

export default function ApiPage() {
  const { data: overview, isLoading: loadingOverview } = useQuery({
    queryKey: ['stats-overview'],
    queryFn:  statsApi.getOverview,
  });

  const { data: latency, isLoading: loadingLatency } = useQuery({
    queryKey: ['stats-latency'],
    queryFn:  statsApi.getLatency,
  });

  return (
    <div className="p-8 max-w-5xl mx-auto">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-white">API & Stats</h1>
        <p className="text-gray-400 mt-1">Real-time overview of your Blackhole Mail instance</p>
      </div>

      {/* ── Overview cards ── */}
      <h2 className="text-lg font-semibold text-gray-300 mb-4 flex items-center gap-2">
        <Activity size={18} className="text-blue-400" /> Email Overview
      </h2>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-10">
        <StatCard icon={Mail}         label="Total Emails"  value={overview?.total_emails}    color="blue"   loading={loadingOverview} />
        <StatCard icon={Clock}        label="New"           value={overview?.new_emails}      color="yellow" loading={loadingOverview} />
        <StatCard icon={CheckCircle}  label="Resolved"      value={overview?.resolved_emails} color="green"  loading={loadingOverview} />
        <StatCard icon={AlertCircle}  label="Overdue"       value={overview?.overdue_emails}  color="red"    loading={loadingOverview} />
      </div>

      {/* ── Avg response time ── */}
      <div className="card mb-10 flex items-center gap-4">
        <div className="w-12 h-12 bg-gradient-to-br from-purple-600 to-purple-700 rounded-lg flex items-center justify-center shrink-0">
          <Clock className="text-white" size={22} />
        </div>
        <div>
          <p className="text-sm text-gray-400">Avg. First-Response Time</p>
          {loadingOverview ? (
            <div className="h-7 w-24 bg-blackhole-800 rounded animate-pulse mt-1" />
          ) : (
            <p className="text-2xl font-bold text-white mt-0.5">
              {overview?.avg_response_time_hours.toFixed(1)} <span className="text-sm font-normal text-gray-400">hours</span>
            </p>
          )}
        </div>
      </div>

      {/* ── Latency cards ── */}
      <h2 className="text-lg font-semibold text-gray-300 mb-4 flex items-center gap-2">
        <Zap size={18} className="text-yellow-400" /> Delivery Latency
      </h2>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 mb-6">
        <LatencyCard
          title="Ingest Latency"
          icon={ArrowDownCircle}
          avg={latency?.avg_ingest_ms}
          p95={latency?.p95_ingest_ms}
          p99={latency?.p99_ingest_ms}
          loading={loadingLatency}
        />
        <LatencyCard
          title="Delivery Latency"
          icon={ArrowUpCircle}
          avg={latency?.avg_delivery_ms}
          p95={latency?.p95_delivery_ms}
          p99={latency?.p99_delivery_ms}
          loading={loadingLatency}
        />
      </div>

      {/* ── Endpoint reference ── */}
      <h2 className="text-lg font-semibold text-gray-300 mb-4 mt-10 flex items-center gap-2">
        <Activity size={18} className="text-green-400" /> Live Endpoint Reference
      </h2>
      <div className="card">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-blackhole-800 text-left">
              <th className="pb-2 text-gray-400 font-medium">Method</th>
              <th className="pb-2 text-gray-400 font-medium">Endpoint</th>
              <th className="pb-2 text-gray-400 font-medium">Description</th>
            </tr>
          </thead>
          <tbody className="text-gray-300">
            {ENDPOINTS.map((ep, i) => (
              <tr key={i} className="border-b border-blackhole-800 last:border-0">
                <td className="py-2">
                  <span className={`inline-block px-2 py-0.5 rounded text-xs font-bold ${METHOD_COLOR[ep.method]}`}>
                    {ep.method}
                  </span>
                </td>
                <td className="py-2 font-mono text-blue-300">{ep.path}</td>
                <td className="py-2 text-gray-400">{ep.desc}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

// ─── helpers ─────────────────────────────────────────────────────────────────

const GRAD: Record<string, string> = {
  blue:   'from-blue-600 to-blue-700',
  yellow: 'from-yellow-600 to-yellow-700',
  green:  'from-green-600 to-green-700',
  red:    'from-red-600 to-red-700',
};

function StatCard({ icon: Icon, label, value, color, loading }: {
  icon: React.FC<{ size?: number; className?: string }>;
  label: string; value?: number; color: string; loading: boolean;
}) {
  return (
    <div className="card">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm text-gray-400">{label}</p>
          {loading ? (
            <div className="h-9 w-20 bg-blackhole-800 rounded animate-pulse mt-1" />
          ) : (
            <p className="text-3xl font-bold text-white mt-1">{value ?? 0}</p>
          )}
        </div>
        <div className={`w-12 h-12 bg-gradient-to-br ${GRAD[color]} rounded-lg flex items-center justify-center`}>
          <Icon className="text-white" size={24} />
        </div>
      </div>
    </div>
  );
}

function LatencyCard({ title, icon: Icon, avg, p95, p99, loading }: {
  title: string;
  icon: React.FC<{ size?: number; className?: string }>;
  avg?: number; p95?: number; p99?: number; loading: boolean;
}) {
  return (
    <div className="card">
      <div className="flex items-center gap-3 mb-3">
        <Icon size={20} className="text-yellow-400" />
        <h3 className="font-semibold text-white">{title}</h3>
      </div>
      {loading ? (
        <div className="space-y-2">
          <div className="h-5 w-32 bg-blackhole-800 rounded animate-pulse" />
          <div className="h-5 w-24 bg-blackhole-800 rounded animate-pulse" />
          <div className="h-5 w-28 bg-blackhole-800 rounded animate-pulse" />
        </div>
      ) : (
        <div className="space-y-1.5 text-sm">
          <div className="flex justify-between">
            <span className="text-gray-400">Average</span>
            <span className="font-semibold text-green-400">{avg?.toFixed(1)} ms</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-400">P95</span>
            <span className="font-semibold text-yellow-400">{p95} ms</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-400">P99</span>
            <span className="font-semibold text-red-400">{p99} ms</span>
          </div>
        </div>
      )}
    </div>
  );
}

const METHOD_COLOR: Record<string, string> = {
  GET:    'bg-blue-900/60 text-blue-300',
  POST:   'bg-green-900/60 text-green-300',
  PUT:    'bg-yellow-900/60 text-yellow-300',
  DELETE: 'bg-red-900/60 text-red-300',
};

const ENDPOINTS = [
  { method:'POST',   path:'/api/auth/login',          desc:'Authenticate and receive a token' },
  { method:'GET',    path:'/api/auth/me',             desc:'Get current user info' },
  { method:'GET',    path:'/api/emails',              desc:'List emails (filterable)' },
  { method:'POST',   path:'/api/emails',              desc:'Send / create an email' },
  { method:'GET',    path:'/api/emails/:id',          desc:'Get full email detail' },
  { method:'PUT',    path:'/api/emails/:id',          desc:'Update email (state / tags / assign)' },
  { method:'PUT',    path:'/api/emails/:id/state',    desc:'Transition email state' },
  { method:'PUT',    path:'/api/emails/:id/assign',   desc:'Assign email to a user' },
  { method:'GET',    path:'/api/emails/:id/notes',    desc:'List internal notes' },
  { method:'POST',   path:'/api/emails/:id/notes',    desc:'Add an internal note' },
  { method:'GET',    path:'/api/emails/:id/routing',  desc:'Explain routing decision' },
  { method:'DELETE', path:'/api/emails/:id',          desc:'Delete an email' },
  { method:'GET',    path:'/api/inboxes',             desc:'List inboxes' },
  { method:'POST',   path:'/api/inboxes',             desc:'Create inbox' },
  { method:'GET',    path:'/api/inboxes/:id/emails',  desc:'Emails in a specific inbox' },
  { method:'GET',    path:'/api/rules',               desc:'List automation rules' },
  { method:'POST',   path:'/api/rules',               desc:'Create rule' },
  { method:'PUT',    path:'/api/rules/:id/toggle',    desc:'Toggle rule on/off' },
  { method:'GET',    path:'/api/users',               desc:'List users' },
  { method:'GET',    path:'/api/stats/overview',      desc:'Email overview statistics' },
  { method:'GET',    path:'/api/stats/latency',       desc:'Ingest & delivery latency' },
  { method:'GET',    path:'/api/audit',               desc:'Audit log entries' },
];
