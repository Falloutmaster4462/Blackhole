import { useParams } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { emailApi } from '../api';
import { ArrowLeft, Clock, User, Tag } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { formatDistanceToNow } from 'date-fns';
import DOMPurify from 'dompurify';

export default function EmailDetailPage() {
  const { emailId } = useParams<{ emailId: string }>();
  const navigate = useNavigate();

  const { data, isLoading } = useQuery({
    queryKey: ['email', emailId],
    queryFn: () => emailApi.get(emailId!),
    enabled: !!emailId,
  });

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  if (!data) return null;

  const { email, attachments, internal_notes, state_history } = data;

  return (
    <div className="min-h-screen bg-blackhole-950 p-8">
      <button
        onClick={() => navigate(-1)}
        className="flex items-center space-x-2 text-gray-400 hover:text-white mb-6"
      >
        <ArrowLeft size={20} />
        <span>Back to inbox</span>
      </button>

      <div className="max-w-5xl mx-auto">
        <div className="card mb-6">
          <div className="flex items-start justify-between mb-4">
            <div className="flex-1">
              <h1 className="text-2xl font-bold text-white mb-2">{email.subject}</h1>
              <div className="flex items-center space-x-4 text-sm text-gray-400">
                <div className="flex items-center space-x-2">
                  <User size={16} />
                  <span>{email.from_name || email.from_address}</span>
                </div>
                <div className="flex items-center space-x-2">
                  <Clock size={16} />
                  <span>{formatDistanceToNow(new Date(email.received_at), { addSuffix: true })}</span>
                </div>
              </div>
            </div>
            <span className={`state-badge state-${email.state}`}>
              {email.state}
            </span>
          </div>

          {email.tags.length > 0 && (
            <div className="flex items-center space-x-2 mb-4">
              <Tag size={16} className="text-gray-400" />
              {email.tags.map((tag) => (
                <span key={tag} className="px-2 py-1 rounded-full bg-blackhole-800 text-sm text-gray-300">
                  {tag}
                </span>
              ))}
            </div>
          )}

          <div className="border-t border-blackhole-800 pt-4 mt-4">
            {email.body_html ? (
              <div
                className="prose prose-invert max-w-none"
                dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(email.body_html) }}
              />
            ) : (
              <pre className="whitespace-pre-wrap text-gray-300 font-sans">
                {email.body_text}
              </pre>
            )}
          </div>

          {attachments.length > 0 && (
            <div className="border-t border-blackhole-800 pt-4 mt-4">
              <h3 className="text-sm font-semibold text-gray-300 mb-2">Attachments</h3>
              <div className="space-y-2">
                {attachments.map((attachment) => (
                  <div
                    key={attachment.id}
                    className="flex items-center space-x-2 p-2 bg-blackhole-800 rounded-lg"
                  >
                    <span className="text-sm text-white">{attachment.filename}</span>
                    <span className="text-xs text-gray-400">
                      ({(attachment.size_bytes / 1024).toFixed(1)} KB)
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* State History */}
        {state_history.length > 0 && (
          <div className="card">
            <h3 className="text-lg font-semibold text-white mb-4">State History</h3>
            <div className="space-y-3">
              {state_history.map((history) => (
                <div key={history.id} className="flex items-center space-x-3 text-sm">
                  <span className={`state-badge state-${history.from_state}`}>
                    {history.from_state}
                  </span>
                  <span className="text-gray-400">→</span>
                  <span className={`state-badge state-${history.to_state}`}>
                    {history.to_state}
                  </span>
                  <span className="text-gray-400">
                    {formatDistanceToNow(new Date(history.created_at), { addSuffix: true })}
                  </span>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
