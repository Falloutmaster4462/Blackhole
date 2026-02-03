import { useState, useEffect, useRef } from 'react';
import { X, Send, ChevronDown } from 'lucide-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { emailApi } from '../api';
import toast from 'react-hot-toast';
import clsx from 'clsx';

interface ComposeEmailModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function ComposeEmailModal({ isOpen, onClose }: ComposeEmailModalProps) {
  const queryClient = useQueryClient();
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // ── form state ──────────────────────────────────────────────────────────
  const [to, setTo]           = useState('');
  const [cc, setCc]           = useState('');
  const [bcc, setBcc]         = useState('');
  const [subject, setSubject] = useState('');
  const [body, setBody]       = useState('');
  const [showCc, setShowCc]   = useState(false);
  const [showBcc, setShowBcc] = useState(false);

  // ── derived validation ──────────────────────────────────────────────────
  const toList     = to.split(',').map(s => s.trim()).filter(Boolean);
  const canSend    = toList.length > 0 && subject.trim().length > 0 && body.trim().length > 0;

  // ── focus textarea when modal opens ─────────────────────────────────────
  useEffect(() => {
    if (isOpen && textareaRef.current) textareaRef.current.focus();
  }, [isOpen]);

  // ── keyboard shortcuts ──────────────────────────────────────────────────
  useEffect(() => {
    if (!isOpen) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') { e.preventDefault(); onClose(); }
      if ((e.ctrlKey || e.metaKey) && e.key === 'Return') { e.preventDefault(); if (canSend) doSend(); }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isOpen, canSend, to, cc, bcc, subject, body]);

  // ── mutation ────────────────────────────────────────────────────────────
  const send = useMutation({
    mutationFn: () => emailApi.send({
      to:      toList,
      cc:      cc  ? cc.split(',').map(s => s.trim()).filter(Boolean)  : undefined,
      bcc:     bcc ? bcc.split(',').map(s => s.trim()).filter(Boolean) : undefined,
      subject: subject.trim(),
      body_text: body,
    }),
    onSuccess: (email) => {
      queryClient.invalidateQueries({ queryKey: ['emails'] });
      const latency = email.delivery_latency_ms;
      toast.success(
        latency != null
          ? `Sent via Resend in ${latency} ms`
          : 'Email saved (no Resend key configured – stored locally)',
        { duration: 3500 }
      );
      reset();
      onClose();
    },
    onError: (err: any) => {
      const msg = err?.response?.data?.error ?? err?.message ?? 'Unknown error';
      toast.error(`Send failed: ${msg}`);
    },
  });

  const doSend = () => { if (canSend && !send.isPending) send.mutate(); };

  const reset = () => {
    setTo(''); setCc(''); setBcc(''); setSubject(''); setBody('');
    setShowCc(false); setShowBcc(false);
  };

  // ── render guard ────────────────────────────────────────────────────────
  if (!isOpen) return null;

  // ── shared input className ──────────────────────────────────────────────
  const inputCls = 'flex-1 min-w-0 bg-transparent border-none outline-none text-white placeholder-gray-500 text-sm py-1.5';

  return (
    /* backdrop */
    <div
      className="fixed inset-0 z-50 flex items-end sm:items-center justify-center bg-black/60"
      onClick={(e) => e.target === e.currentTarget && onClose()}
    >
      {/* modal shell */}
      <div className="bg-blackhole-900 border border-blackhole-700 rounded-t-2xl sm:rounded-2xl shadow-2xl w-full sm:max-w-2xl max-h-[90vh] flex flex-col animate-fade-in">

        {/* ── header ── */}
        <div className="flex items-center justify-between px-5 pt-4 pb-2">
          <h2 className="text-base font-semibold text-white flex items-center gap-2">
            <Send size={16} className="text-blue-400" /> New message
          </h2>
          <button onClick={onClose} className="p-1 rounded hover:bg-blackhole-800 text-gray-400 hover:text-white">
            <X size={18} />
          </button>
        </div>

        {/* ── address fields ── */}
        <div className="px-5 border-b border-blackhole-800 pb-3 space-y-0.5">

          {/* To */}
          <div className="flex items-center gap-2">
            <span className="text-xs font-semibold text-gray-500 w-7 text-right shrink-0">To</span>
            <input className={inputCls} placeholder="recipient@example.com" value={to}
              onChange={e => setTo(e.target.value)} disabled={send.isPending} autoComplete="off" />
            {/* Cc / Bcc toggle pills */}
            <div className="flex gap-1 shrink-0">
              {!showCc  && <button onClick={() => setShowCc(true)}  className="text-xs text-blue-400 hover:text-blue-300 px-1.5 py-0.5 rounded hover:bg-blackhole-800">Cc</button>}
              {!showBcc && <button onClick={() => setShowBcc(true)} className="text-xs text-blue-400 hover:text-blue-300 px-1.5 py-0.5 rounded hover:bg-blackhole-800">Bcc</button>}
            </div>
          </div>

          {/* Cc */}
          {showCc && (
            <div className="flex items-center gap-2">
              <span className="text-xs font-semibold text-gray-500 w-7 text-right shrink-0">Cc</span>
              <input className={inputCls} placeholder="cc@example.com" value={cc} onChange={e => setCc(e.target.value)} />
              <button onClick={() => { setShowCc(false); setCc(''); }} className="text-gray-500 hover:text-gray-300"><X size={14} /></button>
            </div>
          )}

          {/* Bcc */}
          {showBcc && (
            <div className="flex items-center gap-2">
              <span className="text-xs font-semibold text-gray-500 w-7 text-right shrink-0">Bcc</span>
              <input className={inputCls} placeholder="bcc@example.com" value={bcc} onChange={e => setBcc(e.target.value)} />
              <button onClick={() => { setShowBcc(false); setBcc(''); }} className="text-gray-500 hover:text-gray-300"><X size={14} /></button>
            </div>
          )}

          {/* Subject */}
          <div className="flex items-center gap-2 pt-1">
            <span className="text-xs font-semibold text-gray-500 w-7 text-right shrink-0">Sub</span>
            <input className={inputCls} placeholder="Subject" value={subject}
              onChange={e => setSubject(e.target.value)} disabled={send.isPending} autoComplete="off" />
          </div>
        </div>

        {/* ── body ── */}
        <div className="flex-1 overflow-y-auto px-5 py-3">
          <textarea
            ref={textareaRef}
            className="w-full h-full min-h-[140px] bg-transparent text-gray-200 text-sm resize-none outline-none placeholder-gray-600"
            placeholder="Write your message…"
            value={body}
            onChange={e => setBody(e.target.value)}
            disabled={send.isPending}
          />
        </div>

        {/* ── footer ── */}
        <div className="flex items-center justify-between px-5 py-3 border-t border-blackhole-800">
          {/* left: shortcut hint */}
          <span className="text-xs text-gray-600">
            {send.isPending ? 'Sending…' : <><kbd className="px-1.5 py-0.5 rounded bg-blackhole-800 text-gray-400 text-xs">⌘</kbd><span className="mx-1 text-gray-600">+</span><kbd className="px-1.5 py-0.5 rounded bg-blackhole-800 text-gray-400 text-xs">↵</kbd><span className="ml-1.5">to send</span></>}
          </span>

          {/* right: cancel + send */}
          <div className="flex items-center gap-2">
            <button onClick={onClose} disabled={send.isPending}
              className="px-3 py-1.5 text-sm text-gray-400 hover:text-white rounded hover:bg-blackhole-800 disabled:opacity-40">
              Cancel
            </button>
            <button onClick={doSend} disabled={!canSend || send.isPending}
              className={clsx(
                'flex items-center gap-1.5 px-4 py-1.5 rounded-lg text-sm font-medium transition-colors',
                canSend && !send.isPending
                  ? 'bg-blue-600 hover:bg-blue-500 text-white shadow-lg shadow-blue-900/40'
                  : 'bg-blackhole-800 text-gray-500 cursor-not-allowed'
              )}
            >
              <Send size={15} />
              {send.isPending ? 'Sending…' : 'Send'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
