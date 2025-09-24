import React, { useState, useEffect, useRef } from 'react';
import { LogoIcon, HamburgerIcon, CloseIcon, ArrowUpIcon, GitHubIcon, KubernetesIcon, HttpApiIcon, PerformanceIcon, OperatorIcon, SecurityIcon, CloudNativeIcon, UserIcon, DatabaseIcon, BuildingOfficeIcon, ApiKeyIcon } from './ui/Icons';
import { getSystemStats } from '../services/api';
import { SystemStats } from '../types';

type PublicView = 'welcome' | 'docs' | 'guides' | 'terms';

interface WelcomePageProps {
  onNavigateToLogin: () => void;
  onNavigateToRegister: () => void;
  onNavigate: (page: PublicView) => void;
  activePage: PublicView;
  children?: React.ReactNode;
}

const WelcomeContent: React.FC<{onNavigateToRegister: () => void;}> = ({ onNavigateToRegister }) => {
  const [stats, setStats] = useState<SystemStats['tables'] | null>(null);

  useEffect(() => {
    getSystemStats()
      .then(data => setStats(data.tables))
      .catch(console.error);
  }, []);

  const formatStat = (n: number): string => {
    if (n >= 1000) {
      const num = n / 1000;
      // Use toFixed(1) and remove .0 if it exists, otherwise keep the decimal
      const formattedNum = num.toFixed(1).replace(/\.0$/, '');
      return `${formattedNum}k+`;
    }
    if (n >= 100) return `${Math.floor(n / 100) * 100}+`;
    if (n >= 10) return `${Math.floor(n / 10) * 10}+`;
    return n.toString();
  };

  const statItems = stats ? [
    { name: 'Users', value: formatStat(stats.users), icon: <UserIcon /> },
    { name: 'Organizations', value: formatStat(stats.organizations), icon: <BuildingOfficeIcon className="h-5 w-5" /> },
    { name: 'Redis Instances', value: formatStat(stats.redis_instances), icon: <DatabaseIcon className="h-5 w-5" /> },
    { name: 'API Keys Issued', value: formatStat(stats.api_keys), icon: <ApiKeyIcon /> },
  ] : [];

  const features = [
    {
      name: 'Declarative Provisioning',
      description: 'Create and manage dedicated Redis instances using a simple Kubernetes Custom Resource (RedisHttpInstance).',
      icon: KubernetesIcon,
      gradient: 'from-blue-500 to-sky-600',
    },
    {
      name: 'HTTP/S Access',
      description: 'Interact with Redis using a standard RESTful API, eliminating the need for TCP clients in serverless or edge environments.',
      icon: HttpApiIcon,
      gradient: 'from-green-400 to-teal-500',
    },
    {
      name: 'High-Performance Gateway',
      description: 'A single, multi-tenant gateway written in Rust handles all traffic, offering high concurrency and low latency.',
      icon: PerformanceIcon,
      gradient: 'from-amber-400 to-orange-500',
    },
    {
      name: 'Automated Lifecycle Management',
      description: 'A Kubernetes Operator handles the entire lifecycle of Redis instances, from provisioning to decommissioning.',
      icon: OperatorIcon,
      gradient: 'from-violet-500 to-purple-500',
    },
    {
      name: 'Secure by Design',
      description: 'Each instance is isolated within its own namespace and protected by a unique, auto-generated API key.',
      icon: SecurityIcon,
      gradient: 'from-rose-400 to-red-500',
    },
    {
      name: 'Cloud-Native Integration',
      description: 'Built to leverage Kubernetes for resource management, automatic rollbacks, health monitoring, and status synchronization.',
      icon: CloudNativeIcon,
      gradient: 'from-cyan-400 to-sky-500',
    },
  ];

  const FeatureCard: React.FC<{ feature: (typeof features)[0]; index: number }> = ({ feature, index }) => {
    const [isVisible, setIsVisible] = useState(false);
    const cardRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
      const observer = new IntersectionObserver(
        ([entry]) => {
          if (entry.isIntersecting) {
            setIsVisible(true);
            observer.unobserve(entry.target);
          }
        },
        { threshold: 0.1 }
      );

      const currentRef = cardRef.current;
      if (currentRef) {
        observer.observe(currentRef);
      }

      return () => {
        if (currentRef) {
          observer.unobserve(currentRef);
        }
      };
    }, []);

    return (
      <div
        ref={cardRef}
        className={`flex flex-col items-start transition-all duration-500 ${
          isVisible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-8'
        }`}
        style={{ transitionDelay: `${index * 100}ms` }}
      >
        <div className={`p-4 rounded-xl bg-gradient-to-br ${feature.gradient} text-white shadow-lg mb-4`}>
          <feature.icon />
        </div>
        <h3 className="text-xl font-bold text-slate-900">{feature.name}</h3>
        <p className="mt-2 text-base leading-7 text-slate-600">{feature.description}</p>
      </div>
    );
  };

  return (
    <>
      {/* Hero section */}
      <div className="relative isolate px-6 pt-14 lg:px-8 overflow-hidden">
        <div className="mx-auto max-w-3xl py-32 sm:py-48 lg:py-56 text-center">
            <h1 className="text-4xl font-bold tracking-tight text-slate-900 sm:text-6xl animate-fadeInUp" style={{ animationDelay: '100ms' }}>
                Bridging Redis to the Serverless & Edge World.
            </h1>
            <p className="mt-8 text-lg leading-8 text-slate-600 animate-fadeInUp" style={{ animationDelay: '400ms' }}>
               RedisGate is a Kubernetes-native HTTP gateway that makes high-performance Redis accessible from environments like Cloudflare Workers and Vercel Edge Functions where TCP sockets are prohibited.
            </p>
            <div className="mt-10 flex items-center justify-center gap-x-6 animate-fadeInUp" style={{ animationDelay: '550ms' }}>
                <button
                    onClick={onNavigateToRegister}
                    className="rounded-xl bg-gradient-to-r from-cyan-500 to-teal-600 px-4 py-2.5 text-base font-semibold text-white shadow-lg transition-all duration-300 transform hover:scale-105 hover:shadow-xl"
                >
                    Start Free Trial
                </button>
                <a
                    href="https://github.com/AI-Decenter/RedisGate"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="flex items-center gap-2 rounded-xl bg-white px-4 py-2.5 text-base font-semibold text-slate-900 shadow-md ring-1 ring-inset ring-slate-300 hover:bg-slate-50 transition-all duration-300 transform hover:scale-105"
                >
                    <GitHubIcon />
                    View on GitHub
                </a>
            </div>
        </div>
      </div>
      
      {/* Stats Section */}
      {stats && (
          <div className="bg-slate-900 py-24 sm:py-32">
            <div className="mx-auto max-w-7xl px-6 lg:px-8">
              <div className="mx-auto max-w-2xl lg:max-w-none">
                <div className="text-center">
                  <h2 className="text-3xl font-bold tracking-tight text-white sm:text-4xl">Trusted by developers worldwide</h2>
                  <p className="mt-4 text-lg leading-8 text-gray-300">Powering projects of all sizes, from solo experiments to enterprise-scale applications.</p>
                </div>
                <dl className="mt-16 grid grid-cols-1 gap-x-8 gap-y-10 text-center sm:grid-cols-2 lg:grid-cols-4">
                  {statItems.map((stat) => (
                    <div key={stat.name} className="flex flex-col items-center">
                      <dd className="order-first text-3xl font-semibold tracking-tight text-white">{stat.value}</dd>
                      <dt className="text-sm font-semibold leading-6 text-gray-300 flex items-center gap-2 mt-2">{stat.icon} {stat.name}</dt>
                    </div>
                  ))}
                </dl>
              </div>
            </div>
          </div>
        )}

      {/* Feature section */}
      <div className="bg-white py-24 sm:py-32">
        <div className="mx-auto max-w-7xl px-6 lg:px-8">
            <div className="mx-auto grid max-w-2xl grid-cols-1 gap-x-12 gap-y-16 sm:grid-cols-2 lg:mx-0 lg:max-w-none lg:grid-cols-3 lg:gap-x-16">
                {features.map((feature, index) => (
                    <FeatureCard key={feature.name} feature={feature} index={index} />
                ))}
            </div>
        </div>
      </div>
    </>
  );
};


const WelcomePage: React.FC<WelcomePageProps> = ({ onNavigateToLogin, onNavigateToRegister, onNavigate, activePage, children }) => {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [showScrollTop, setShowScrollTop] = useState(false);

  useEffect(() => {
    const checkScrollTop = () => {
      if (!showScrollTop && window.pageYOffset > 400){
        setShowScrollTop(true);
      } else if (showScrollTop && window.pageYOffset <= 400){
        setShowScrollTop(false);
      }
    };
    window.addEventListener('scroll', checkScrollTop);
    return () => window.removeEventListener('scroll', checkScrollTop);
  }, [showScrollTop]);

  const scrollTop = () => {
    window.scrollTo({top: 0, behavior: 'smooth'});
  };

  const navItems: { name: string; page: PublicView }[] = [
    { name: 'Docs', page: 'docs' },
    { name: 'User Guides', page: 'guides' },
  ];

  return (
    <div className="bg-slate-50">
      <header className="bg-white/80 backdrop-blur-md sticky top-0 z-50 border-b border-slate-200/60">
        <nav className="mx-auto flex max-w-7xl items-center justify-between p-4 lg:px-8" aria-label="Global">
          <div className="flex lg:flex-1">
            <button onClick={() => onNavigate('welcome')} className="-m-1.5 p-1.5 flex items-center gap-2">
              <LogoIcon />
              <span className="text-xl font-bold text-slate-800">RedisGate</span>
            </button>
          </div>
          <div className="flex lg:hidden">
            <button
              type="button"
              className="-m-2.5 inline-flex items-center justify-center rounded-md p-2.5 text-gray-700"
              onClick={() => setMobileMenuOpen(true)}
            >
              <span className="sr-only">Open main menu</span>
              <HamburgerIcon />
            </button>
          </div>
          <div className="hidden lg:flex lg:gap-x-12">
            {navItems.map((item) => (
              <button key={item.name} onClick={() => onNavigate(item.page)} className={`text-lg font-semibold leading-6 transition-colors ${activePage === item.page ? 'text-teal-600' : 'text-gray-900 hover:text-teal-600'}`}>
                {item.name}
              </button>
            ))}
          </div>
          <div className="hidden lg:flex lg:flex-1 lg:justify-end items-center gap-x-4">
             <button
                onClick={onNavigateToLogin}
                className="rounded-xl bg-white px-4 py-2.5 text-sm font-semibold text-slate-900 shadow-sm ring-1 ring-inset ring-slate-200 hover:bg-slate-100 transition-all duration-300 transform hover:scale-105"
              >
                Log in
            </button>
            <button
                onClick={onNavigateToRegister}
                className="group inline-flex items-center justify-center rounded-xl bg-gradient-to-r from-cyan-500 to-teal-600 px-4 py-2.5 text-sm font-semibold text-white shadow-md transition-all duration-300 transform hover:scale-105 hover:shadow-xl"
              >
                Register
            </button>
          </div>
        </nav>
      </header>

      {/* Mobile Menu - Moved outside of header to fix stacking context issues */}
      <div 
        className={`lg:hidden fixed inset-0 z-[99] transition-all duration-300 ease-in-out ${mobileMenuOpen ? 'pointer-events-auto' : 'pointer-events-none'}`}
        role="dialog" 
        aria-modal="true"
      >
        {/* Backdrop */}
        <div className={`fixed inset-0 bg-black/25 transition-opacity ${mobileMenuOpen ? 'opacity-100' : 'opacity-0 pointer-events-none'}`} onClick={() => setMobileMenuOpen(false)} />
        
        {/* Panel */}
        <div className={`fixed inset-y-0 right-0 z-[100] w-full max-w-sm transform bg-white shadow-xl transition-transform ease-in-out duration-300 ${mobileMenuOpen ? 'translate-x-0' : 'translate-x-full'}`}>
            <div className="flex h-full flex-col">
                <div className="flex items-center justify-between p-4 border-b border-slate-200">
                    <button onClick={() => { setMobileMenuOpen(false); onNavigate('welcome'); }} className="-m-1.5 p-1.5 flex items-center gap-2">
                      <LogoIcon />
                      <span className="text-xl font-bold text-slate-800">RedisGate</span>
                    </button>
                    <button
                      type="button"
                      className="-m-2.5 rounded-md p-2.5 text-gray-700"
                      onClick={() => setMobileMenuOpen(false)}
                    >
                      <span className="sr-only">Close menu</span>
                      <CloseIcon />
                    </button>
                </div>
                <div className="mt-6 flow-root flex-grow overflow-y-auto">
                    <div className="-my-6 divide-y divide-gray-500/10">
                         <div className="space-y-2 py-6 px-4">
                            {navItems.map((item) => (
                                <button key={item.name} onClick={() => { setMobileMenuOpen(false); onNavigate(item.page); }} className={`-mx-3 block w-full text-left rounded-lg px-3 py-2 text-lg font-semibold leading-7 ${activePage === item.page ? 'bg-gray-50 text-teal-600' : 'text-gray-900 hover:bg-gray-50'}`}>
                                    {item.name}
                                </button>
                            ))}
                        </div>
                        <div className="py-6 px-4 space-y-4">
                            <button
                              onClick={() => { setMobileMenuOpen(false); onNavigateToRegister(); }}
                              className="w-full rounded-xl bg-gradient-to-r from-cyan-500 to-teal-600 px-3.5 py-3 text-center text-base font-semibold leading-7 text-white shadow-md hover:shadow-lg transition-all duration-300 transform hover:scale-105"
                            >
                              Register
                            </button>
                            <button
                              onClick={() => { setMobileMenuOpen(false); onNavigateToLogin(); }}
                              className="w-full rounded-xl bg-white px-3.5 py-3 text-center text-base font-semibold leading-7 text-slate-900 shadow-sm ring-1 ring-inset ring-slate-200 hover:bg-slate-100 transition-all duration-300 transform hover:scale-105"
                            >
                              Log in
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
      </div>

      <main className="isolate">
        {children || <WelcomeContent onNavigateToRegister={onNavigateToRegister} />}
      </main>

      {/* Footer */}
      <footer className="bg-slate-900" aria-labelledby="footer-heading">
        <div className="mx-auto max-w-7xl px-6 py-12 sm:py-16 lg:px-8">
             <div className="mt-8 border-t border-white/10 pt-8 sm:mt-10 lg:mt-12 flex flex-col sm:flex-row items-center justify-center gap-x-6 gap-y-2">
                <p className="text-xs leading-5 text-gray-400">&copy; {new Date().getFullYear()} RedisGate. All rights reserved.</p>
                <button onClick={() => onNavigate('terms')} className="text-xs leading-5 text-gray-400 hover:text-white transition-colors">
                    Terms of Service
                </button>
            </div>
        </div>
      </footer>
      
      {/* Scroll to top button */}
      <button 
        onClick={scrollTop}
        className={`fixed bottom-8 right-8 p-3 rounded-full bg-teal-600 text-white shadow-lg transition-all duration-300 ${showScrollTop ? 'opacity-100 scale-100' : 'opacity-0 scale-90 pointer-events-none'}`}
        aria-label="Scroll to top"
      >
        <ArrowUpIcon />
      </button>

    </div>
  );
};

export default WelcomePage;