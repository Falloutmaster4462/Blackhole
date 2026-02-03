import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';

// The dashboard is just a landing-pad that forwards to the default Support inbox.
// All the real UI lives in InboxPage (which owns the sidebar).
export default function DashboardPage() {
  const navigate = useNavigate();

  useEffect(() => {
    navigate('/inbox/00000000-0000-0000-0000-000000000003', { replace: true });
  }, [navigate]);

  // Render nothing — the effect will navigate away on mount.
  return null;
}
