import React, { useState, useEffect, useCallback } from 'react';
import Dashboard from './components/Dashboard';
import AuthPage from './components/AuthPage';
import WelcomePage from './components/WelcomePage';
import DocsPage from './components/DocsPage';
import UserGuidesPage from './components/UserGuidesPage';
import TermsOfServicePage from './components/TermsOfServicePage';
import { User } from './types';

type PublicView = 'welcome' | 'docs' | 'guides' | 'terms';
type PageView = PublicView | 'dashboard' | 'auth';

const App: React.FC = () => {
  const [token, setToken] = useState<string | null>(localStorage.getItem('authToken'));
  const [user, setUser] = useState<User | null>(JSON.parse(localStorage.getItem('authUser') || 'null'));
  const [page, setPage] = useState<PageView>(token ? 'dashboard' : 'welcome');
  const [authMode, setAuthMode] = useState<'login' | 'register'>('login');

  useEffect(() => {
    const storedToken = localStorage.getItem('authToken');
    const storedUser = localStorage.getItem('authUser');
    if (storedToken && storedUser) {
      setToken(storedToken);
      setUser(JSON.parse(storedUser));
      setPage('dashboard');
    } else {
      setPage('welcome');
    }
  }, []);

  const handleLoginSuccess = useCallback((newToken: string, newUser: User) => {
    localStorage.setItem('authToken', newToken);
    localStorage.setItem('authUser', JSON.stringify(newUser));
    setToken(newToken);
    setUser(newUser);
    setPage('dashboard');
  }, []);

  const handleLogout = useCallback(() => {
    localStorage.removeItem('authToken');
    localStorage.removeItem('authUser');
    setToken(null);
    setUser(null);
    setPage('welcome');
  }, []);

  const handleNavigateToLogin = () => {
      setAuthMode('login');
      setPage('auth');
  };

  const handleNavigateToRegister = () => {
      setAuthMode('register');
      setPage('auth');
  };

  const renderPage = () => {
    if (page === 'dashboard' && token && user) {
      return <Dashboard user={user} onLogout={handleLogout} token={token} />;
    }
    if (page === 'auth') {
      return <AuthPage onLoginSuccess={handleLoginSuccess} onBack={() => setPage('welcome')} initialView={authMode} />;
    }
    
    // Render public-facing pages within the WelcomePage layout
    let pageContent: React.ReactNode = null;
    switch(page) {
      case 'docs':
        pageContent = <div key={page} className="bg-white p-6 sm:p-10 rounded-xl shadow-md border border-slate-200 my-16 animate-fadeInUp"><DocsPage /></div>;
        break;
      case 'guides':
        pageContent = <div key={page} className="my-16 animate-fadeInUp"><UserGuidesPage onNavigate={(view) => setPage(view)} /></div>;
        break;
      case 'terms':
        pageContent = <div key={page} className="my-16 animate-fadeInUp"><TermsOfServicePage /></div>;
        break;
    }

    return (
      <WelcomePage
        activePage={page as PublicView}
        onNavigate={setPage}
        onNavigateToLogin={handleNavigateToLogin}
        onNavigateToRegister={handleNavigateToRegister}
      >
        {pageContent}
      </WelcomePage>
    );
  };

  return (
    <div className="min-h-screen bg-slate-100 text-slate-900">
      {renderPage()}
    </div>
  );
};

export default App;