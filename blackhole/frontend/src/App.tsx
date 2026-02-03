import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Toaster } from 'react-hot-toast';
import { useAuthStore } from './stores/authStore';
import LoginPage from './pages/LoginPage';
import DashboardPage from './pages/DashboardPage';
import InboxPage from './pages/InboxPage';
import EmailDetailPage from './pages/EmailDetailPage';
import RulesPage from './pages/RulesPage';
import SettingsPage from './pages/SettingsPage';
import ApiPage from './pages/ApiPage';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: 1,
      staleTime: 5000,
    },
  },
});

function PrivateRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated } = useAuthStore();
  return isAuthenticated ? <>{children}</> : <Navigate to="/login" />;
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <div className="min-h-screen bg-blackhole-950 text-gray-100">
          <Routes>
            <Route path="/login" element={<LoginPage />} />
            <Route
              path="/"
              element={
                <PrivateRoute>
                  <DashboardPage />
                </PrivateRoute>
              }
            />
            <Route
              path="/inbox/:inboxId"
              element={
                <PrivateRoute>
                  <InboxPage />
                </PrivateRoute>
              }
            />
            <Route
              path="/email/:emailId"
              element={
                <PrivateRoute>
                  <EmailDetailPage />
                </PrivateRoute>
              }
            />
            <Route
              path="/rules"
              element={
                <PrivateRoute>
                  <RulesPage />
                </PrivateRoute>
              }
            />
            <Route
              path="/settings"
              element={
                <PrivateRoute>
                  <SettingsPage />
                </PrivateRoute>
              }
            />
            <Route
              path="/api"
              element={
                <PrivateRoute>
                  <ApiPage />
                </PrivateRoute>
              }
            />
          </Routes>
          <Toaster
            position="bottom-right"
            toastOptions={{
              className: 'bg-blackhole-800 text-white',
              duration: 4000,
            }}
          />
        </div>
      </BrowserRouter>
    </QueryClientProvider>
  );
}

export default App;
