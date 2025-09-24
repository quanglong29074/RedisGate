import React, { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import { Organization, RedisInstance, User, SystemStats } from '../types';
import { getOrganizations, getRedisInstances, createOrganization, createRedisInstance, updateOrganization, deleteOrganization, deleteRedisInstance, getApiKeys, getSystemHealth, getSystemStats } from '../services/api';
import { BuildingOfficeIcon, DatabaseIcon, PlusIcon, LogoutIcon, ArrowLeftIcon, SpinnerIcon, EditIcon, DeleteIcon, LogoIcon, WarningIcon, CopyIcon, CheckIcon, UserIcon, DashboardIcon, ApiKeyIcon, SettingsIcon, BellIcon, SearchIcon, SystemStatusIcon, LifeBuoyIcon, FileTextIcon, BookOpenIcon, ChevronDownIcon, HamburgerIcon, CloseIcon } from './ui/Icons';
import DocsPage from './DocsPage';
import UserGuidesPage from './UserGuidesPage';
// FIX: Import TermsOfServicePage to handle rendering for the 'terms' view.
import TermsOfServicePage from './TermsOfServicePage';

interface DashboardProps {
  user: User;
  token: string;
  onLogout: () => void;
}

interface Notification {
  id: number;
  message: string;
  type: 'success' | 'error';
  read: boolean;
  context?: {
    orgId?: string;
    instanceId?: string;
  };
}

// Reusable Modal Component
interface ModalProps {
    isOpen: boolean;
    onClose: () => void;
    title: string;
    children: React.ReactNode;
}
const Modal: React.FC<ModalProps> = ({ isOpen, onClose, title, children }) => {
    return (
        <div 
          className={`fixed inset-0 bg-black z-50 flex justify-center items-center p-4 transition-all duration-300 ease-in-out ${isOpen ? 'bg-opacity-60 opacity-100' : 'bg-opacity-0 opacity-0 pointer-events-none'}`} 
          onClick={onClose}
        >
            <div 
              className={`bg-white rounded-lg shadow-xl w-full max-w-lg p-6 relative transition-transform duration-300 ease-in-out ${isOpen ? 'scale-100' : 'scale-95'}`}
              onClick={e => e.stopPropagation()}
            >
                <h3 className="text-xl font-semibold text-slate-900 mb-4">{title}</h3>
                <button onClick={onClose} className="absolute top-4 right-4 text-slate-400 hover:text-slate-800 text-2xl leading-none">&times;</button>
                <div>{children}</div>
            </div>
        </div>
    );
};

// Reusable Confirmation Modal
interface ConfirmationModalProps {
    isOpen: boolean;
    onClose: () => void;
    onConfirm: () => void;
    title: string;
    message: string;
    confirmText?: string;
    isLoading?: boolean;
}
const ConfirmationModal: React.FC<ConfirmationModalProps> = ({ isOpen, onClose, onConfirm, title, message, confirmText = "Confirm", isLoading = false }) => {
    return (
         <div 
           className={`fixed inset-0 z-50 flex justify-center items-center p-4 transition-all duration-300 ease-in-out ${isOpen ? 'bg-black bg-opacity-60 opacity-100' : 'bg-opacity-0 opacity-0 pointer-events-none'}`}
           onClick={onClose}
         >
            <div 
              className={`bg-white rounded-lg shadow-xl w-full max-w-md p-6 relative transition-transform duration-300 ease-in-out ${isOpen ? 'scale-100' : 'scale-95'}`}
              onClick={e => e.stopPropagation()}
            >
                <div className="flex items-start gap-4">
                    <div className="mx-auto flex-shrink-0 flex items-center justify-center h-12 w-12 rounded-full bg-red-100 sm:mx-0 sm:h-10 sm:w-10">
                        <WarningIcon />
                    </div>
                    <div className="mt-0 text-left">
                        <h3 className="text-lg leading-6 font-medium text-slate-900">{title}</h3>
                        <div className="mt-2">
                            <p className="text-sm text-slate-500">{message}</p>
                        </div>
                    </div>
                </div>
                <div className="mt-5 sm:mt-4 sm:flex sm:flex-row-reverse gap-3">
                    <button type="button" onClick={onConfirm} disabled={isLoading} className="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-red-600 text-base font-medium text-white hover:bg-red-700 focus:outline-none sm:w-auto sm:text-sm disabled:bg-red-400">
                        {isLoading ? 'Deleting...' : confirmText}
                    </button>
                    <button type="button" onClick={onClose} className="mt-3 w-full inline-flex justify-center rounded-md border border-slate-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-slate-700 hover:bg-slate-50 focus:outline-none sm:mt-0 sm:w-auto sm:text-sm">
                        Cancel
                    </button>
                </div>
            </div>
        </div>
    )
};

// Reusable Alert Modal
interface AlertModalProps {
    isOpen: boolean;
    onClose: () => void;
    title: string;
    message: string;
}
const AlertModal: React.FC<AlertModalProps> = ({ isOpen, onClose, title, message }) => {
    return (
        <div 
          className={`fixed inset-0 z-50 flex justify-center items-center p-4 transition-all duration-300 ease-in-out ${isOpen ? 'bg-black bg-opacity-60 opacity-100' : 'bg-opacity-0 opacity-0 pointer-events-none'}`}
          onClick={onClose}
        >
            <div 
              className={`bg-white rounded-lg shadow-xl w-full max-w-md p-6 relative transition-transform duration-300 ease-in-out ${isOpen ? 'scale-100' : 'scale-95'}`}
              onClick={e => e.stopPropagation()}
            >
                <div className="flex items-start gap-4">
                    <div className="mx-auto flex-shrink-0 flex items-center justify-center h-12 w-12 rounded-full bg-red-100 sm:mx-0 sm:h-10 sm:w-10">
                        <WarningIcon />
                    </div>
                    <div className="mt-0 text-left">
                        <h3 className="text-lg leading-6 font-medium text-slate-900">{title}</h3>
                        <div className="mt-2">
                            <p className="text-sm text-slate-500">{message}</p>
                        </div>
                    </div>
                </div>
                <div className="mt-5 sm:mt-4 sm:flex sm:flex-row-reverse">
                    <button type="button" onClick={onClose} className="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-teal-600 text-base font-medium text-white hover:bg-teal-700 focus:outline-none sm:w-auto sm:text-sm">
                        OK
                    </button>
                </div>
            </div>
        </div>
    );
};

const InstanceDetailsView: React.FC<{ selectedOrg: Organization; selectedInstance: RedisInstance; onBack: () => void; }> = ({ selectedOrg, selectedInstance, onBack }) => {
    const [copiedText, setCopiedText] = useState('');
    const [expandedEndpoint, setExpandedEndpoint] = useState<string | null>(null);
    const [paramValues, setParamValues] = useState<{ [key: string]: string }>({});

    const handleCopy = (text: string) => {
        navigator.clipboard.writeText(text).then(() => {
            setCopiedText(text);
            setTimeout(() => setCopiedText(''), 2000);
        });
    };
    
    const BASE_API_URL = 'http://localhost:8080';

    const getParamsFromPath = (path: string): string[] => {
        const colonParams = (path.match(/:(\w+)/g) || [])
            .map(match => match.substring(1))
            .filter(param => !['instance_id', 'org_id', 'key_id'].includes(param));
    
        const starParams = path.includes('/*path') ? ['...path'] : [];
    
        return [...colonParams, ...starParams];
    };
    
    const allEndpoints = [
        {
            category: 'Redis HTTP API',
            description: 'Requires API key authentication. The endpoints below are pre-filled with your instance ID for convenience.',
            endpoints: [
                { method: 'GET', path: '/redis/:instance_id/ping', description: 'Pings the Redis instance.' },
                { method: 'GET', path: '/redis/:instance_id/set/:key/:value', description: 'Sets a key with a value.' },
                { method: 'GET', path: '/redis/:instance_id/get/:key', description: 'Gets the value of a key.' },
                { method: 'GET', path: '/redis/:instance_id/del/:key', description: 'Deletes a key.' },
                { method: 'GET', path: '/redis/:instance_id/incr/:key', description: 'Increments the value of a key.' },
                { method: 'GET', path: '/redis/:instance_id/hset/:key/:field/:value', description: 'Sets a field in a hash.' },
                { method: 'GET', path: '/redis/:instance_id/hget/:key/:field', description: 'Gets a field from a hash.' },
                { method: 'GET', path: '/redis/:instance_id/lpush/:key/:value', description: 'Prepends a value to a list.' },
                { method: 'GET', path: '/redis/:instance_id/lpop/:key', description: 'Removes and gets the first element in a list.' },
            ]
        },
        {
            category: 'Generic Redis Commands',
            description: 'Requires API key authentication.',
            endpoints: [
                { method: 'POST', path: '/redis/:instance_id', description: 'Executes a generic Redis command via JSON body.' },
                { method: 'GET', path: '/redis/:instance_id/*path', description: 'Catch-all route for debugging Redis requests.' },
            ]
        },
    ];

    return (
        <div>
             <div className="flex flex-col sm:flex-row justify-between sm:items-center gap-4 mb-6">
                <div className="flex items-center gap-4">
                    <button onClick={onBack} className="flex items-center gap-2 p-2 rounded-md hover:bg-slate-200 text-slate-500 hover:text-slate-900">
                        <ArrowLeftIcon />
                    </button>
                    <div className="min-w-0">
                        <h2 className="text-xl md:text-3xl font-bold text-slate-900 truncate">{selectedInstance.name} - API Guide</h2>
                        <p className="text-slate-500 font-mono text-sm break-all">{selectedInstance.domain}</p>
                    </div>
                </div>
            </div>
            {allEndpoints.map(category => (
                <div key={category.category} className="mb-8">
                    <h3 className="text-2xl font-bold text-slate-800 mb-2">{category.category}</h3>
                    <p className="text-slate-500 mb-4 border-b border-slate-200 pb-3">{category.description}</p>
                     <div className="space-y-2">
                        {category.endpoints.map(ep => {
                            const params = getParamsFromPath(ep.path);
                            const isExpandable = params.length > 0;
                            const isExpanded = expandedEndpoint === ep.path;
                            
                            const displayPath = ep.path
                                .replace(':instance_id', selectedInstance.id)
                                .replace(/:(\w+)/g, `{\$1}`)
                                .replace('/*path', '/{...path}');

                            const finalUrl = `${BASE_API_URL}${displayPath}`;
                            
                            const urlToCopy = Object.entries(paramValues).reduce(
                                (url, [key, value]) => url.replace(`{${key}}`, encodeURIComponent(value || `{${key}}`)),
                                finalUrl
                            );

                            const handleToggleExpand = () => {
                                if (isExpandable) {
                                    if (isExpanded) {
                                        setExpandedEndpoint(null);
                                    } else {
                                        setExpandedEndpoint(ep.path);
                                        const initialParams = params.reduce((acc, param) => ({...acc, [param]: ''}), {});
                                        setParamValues(initialParams);
                                    }
                                }
                            };

                            const methodColors: { [key: string]: { bg: string, border: string, text: string, hoverBg: string } } = {
                                'GET':    { bg: 'bg-sky-50',    border: 'border-sky-300',    text: 'text-sky-800',    hoverBg: 'hover:bg-sky-100' },
                                'POST':   { bg: 'bg-green-50',  border: 'border-green-300',  text: 'text-green-800',  hoverBg: 'hover:bg-green-100' },
                                'PUT':    { bg: 'bg-yellow-50', border: 'border-yellow-300', text: 'text-yellow-800', hoverBg: 'hover:bg-yellow-100' },
                                'DELETE': { bg: 'bg-red-50',    border: 'border-red-300',    text: 'text-red-800',    hoverBg: 'hover:bg-red-100' },
                                'ANY':    { bg: 'bg-purple-50', border: 'border-purple-300', text: 'text-purple-800', hoverBg: 'hover:bg-purple-100' }
                            };
                            const color = methodColors[ep.method] || { bg: 'bg-slate-50', border: 'border-slate-300', text: 'text-slate-800', hoverBg: 'hover:bg-slate-100' };

                            return (
                                <div key={ep.path} className={`border ${isExpanded ? color.border : 'border-slate-200'} rounded-lg overflow-hidden transition-all duration-300 ${isExpanded ? 'shadow-lg' : 'shadow-sm'}`}>
                                    <div
                                        onClick={handleToggleExpand}
                                        className={`flex flex-col sm:flex-row items-start sm:items-center justify-between p-3 gap-2 ${isExpandable ? 'cursor-pointer' : ''} ${color.bg} ${isExpandable ? color.hoverBg : ''} transition-colors`}
                                    >
                                        <div className="flex items-center gap-4 flex-1 min-w-0 w-full">
                                            <span className={`font-mono text-sm font-bold w-[70px] text-center flex-shrink-0 ${color.text}`}>{ep.method}</span>
                                            <span className="font-mono text-sm text-slate-700 break-all min-w-0">{displayPath}</span>
                                        </div>
                                        <div className="flex items-center gap-4 sm:ml-4 w-full sm:w-auto justify-end">
                                            <p className="text-sm text-slate-600 hidden md:block flex-shrink-0">{ep.description}</p>
                                            {!isExpandable && (
                                                <button onClick={(e) => { e.stopPropagation(); handleCopy(finalUrl); }} className="flex items-center gap-1.5 text-sm bg-slate-100 hover:bg-slate-200 text-slate-700 px-3 py-1.5 rounded-md transition-colors flex-shrink-0">
                                                    {copiedText === finalUrl ? <><CheckIcon /> Copied</> : <><CopyIcon /> Copy</>}
                                                </button>
                                            )}
                                            {isExpandable && (
                                                <div className={`transform transition-transform duration-200 ${isExpanded ? 'rotate-180' : ''}`}>
                                                  <ChevronDownIcon />
                                                </div>
                                            )}
                                        </div>
                                    </div>

                                    {isExpanded && (
                                        <div className="p-4 bg-white animate-fadeInUp" style={{animationDuration: '0.3s'}}>
                                            <h4 className="font-semibold text-slate-800 text-md mb-3">Parameters</h4>
                                            <div className="space-y-3">
                                                {params.map(param => (
                                                    <div key={param} className="grid grid-cols-1 sm:grid-cols-[120px_1fr] items-center gap-2 sm:gap-3">
                                                        <label htmlFor={param} className="font-mono text-sm text-slate-600 text-left sm:text-right font-medium">
                                                            {param}
                                                            <span className="text-red-500 ml-1">*</span>
                                                        </label>
                                                        <input
                                                            id={param}
                                                            type="text"
                                                            placeholder={`string`}
                                                            value={paramValues[param] || ''}
                                                            onChange={(e) => setParamValues(prev => ({ ...prev, [param]: e.target.value }))}
                                                            className="w-full p-2 bg-slate-50 border border-slate-300 rounded-md text-slate-900 focus:ring-2 focus:ring-teal-500 focus:border-teal-500 outline-none transition"
                                                        />
                                                    </div>
                                                ))}
                                            </div>
                                            <div className="border-t mt-4 pt-4 flex justify-end">
                                                <button onClick={() => handleCopy(urlToCopy)} className="flex items-center gap-1.5 text-sm bg-teal-600 hover:bg-teal-700 text-white font-semibold px-4 py-2 rounded-md transition-colors shadow-sm">
                                                    {copiedText === urlToCopy ? <><CheckIcon /> Copied</> : <><CopyIcon /> Copy URL</>}
                                                </button>
                                            </div>
                                        </div>
                                    )}
                                </div>
                            );
                        })}
                    </div>
                </div>
            ))}
        </div>
    )
}

// FIX: Added 'terms' to the View type to match the props of UserGuidesPage, resolving a TypeScript error.
type View = 'dashboard' | 'docs' | 'guides' | 'terms';

const SidebarNavLink: React.FC<{ view: View, current: View, children: React.ReactNode, setView: (view: View) => void, icon: React.ReactNode }> = ({ view, current, children, setView, icon }) => (
    <button 
        onClick={() => setView(view)} 
        className={`w-full flex items-center gap-3 px-4 py-2.5 rounded-lg text-lg font-medium transition-all duration-200 ${current === view ? 'bg-gradient-to-r from-cyan-500 to-teal-600 text-white shadow-md' : 'text-slate-300 hover:bg-slate-700/50 hover:text-white'}`}
    >
        {icon}
        <span>{children}</span>
    </button>
  );

const SidebarContent: React.FC<{ onLogout: () => void; currentView: View; setView: (view: View) => void; systemStatus: 'healthy' | 'unhealthy' | 'checking'; onLinkClick?: () => void }> = ({ onLogout, currentView, setView: rawSetView, systemStatus, onLinkClick }) => {
    
    const setView = (view: View) => {
        rawSetView(view);
        if (onLinkClick) onLinkClick();
    };
    
    return (
        <>
            <div className="flex items-center gap-3 p-5 border-b border-slate-700/50">
                <LogoIcon/>
                <h1 className="text-xl font-bold text-white">RedisGate</h1>
            </div>
            <nav className="flex-grow p-4 space-y-2">
                <SidebarNavLink view="dashboard" current={currentView} setView={setView} icon={<DashboardIcon />}>Dashboard</SidebarNavLink>
                <div className="pt-4 mt-4 border-t border-slate-700/50 space-y-2">
                    <p className="px-4 text-xs font-semibold text-slate-500 uppercase">Resources</p>
                    <SidebarNavLink view="docs" current={currentView} setView={setView} icon={<FileTextIcon />}>Docs</SidebarNavLink>
                    <SidebarNavLink view="guides" current={currentView} setView={setView} icon={<BookOpenIcon />}>User Guides</SidebarNavLink>
                </div>
            </nav>
            <div className="p-4 border-t border-slate-700/50">
                <div className="p-4 rounded-lg bg-slate-700/50 text-center">
                    <p className="text-sm font-semibold text-white flex items-center justify-center gap-2"><SystemStatusIcon />System Status</p>
                    {systemStatus === 'healthy' && <p className="text-xs text-green-400 mt-1">All services operational</p>}
                    {systemStatus === 'unhealthy' && <p className="text-xs text-red-400 mt-1">Service Disruption</p>}
                    {systemStatus === 'checking' && <p className="text-xs text-yellow-400 mt-1">Checking...</p>}
                </div>
                <button onClick={onLogout} className="w-full flex items-center justify-center gap-2 mt-4 px-4 py-2.5 rounded-lg text-sm font-medium text-slate-300 hover:bg-slate-700/50 hover:text-white transition-colors">
                    <LogoutIcon />
                    <span>Logout</span>
                </button>
            </div>
        </>
    );
};


const Dashboard: React.FC<DashboardProps> = ({ user, token, onLogout }) => {
  type DashboardSubView = 'overview' | 'org-management' | 'details';
  
  const [organizations, setOrganizations] = useState<Organization[]>([]);
  const [stats, setStats] = useState<SystemStats['tables'] | null>(null);
  const [selectedOrg, setSelectedOrg] = useState<Organization | null>(null);
  const [redisInstances, setRedisInstances] = useState<RedisInstance[]>([]);
  const [apiKeyCount, setApiKeyCount] = useState(0);
  const [loading, setLoading] = useState('');
  const [error, setError] = useState<string | null>(null);

  // View management state
  const [currentView, setCurrentView] = useState<View>('dashboard');
  const [dashboardView, setDashboardView] = useState<DashboardSubView>('overview');
  const [animation, setAnimation] = useState({ class: 'animate-fadeInUp', key: Date.now() });
  const [searchTerm, setSearchTerm] = useState('');
  const [isMobileNavOpen, setMobileNavOpen] = useState(false);

  const [selectedInstance, setSelectedInstance] = useState<RedisInstance | null>(null);
  const [isUserMenuOpen, setUserMenuOpen] = useState(false);
  const [isNotificationMenuOpen, setNotificationMenuOpen] = useState(false);
  const userMenuRef = useRef<HTMLDivElement>(null);
  const notificationMenuRef = useRef<HTMLDivElement>(null);


  // Modals state
  const [isCreateOrgModalOpen, setCreateOrgModalOpen] = useState(false);
  const [isEditOrgModalOpen, setEditOrgModalOpen] = useState(false);
  const [orgToEdit, setOrgToEdit] = useState<Organization | null>(null);
  const [orgToDelete, setOrgToDelete] = useState<Organization | null>(null);
  const [alertModal, setAlertModal] = useState({ isOpen: false, title: '', message: '' });

  const [isCreateRedisModalOpen, setCreateRedisModalOpen] = useState(false);
  const [redisToDelete, setRedisToDelete] = useState<RedisInstance | null>(null);

  const [newOrgData, setNewOrgData] = useState({ name: '', description: '', slug: '' });
  const [newRedisData, setNewRedisData] = useState({
    name: '', slug: '', max_memory: 104857600, redis_version: '7.2', persistence_enabled: true, backup_enabled: false,
  });
  
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [popupNotification, setPopupNotification] = useState<Notification | null>(null);
  const popupTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const [systemStatus, setSystemStatus] = useState<'healthy' | 'unhealthy' | 'checking'>('checking');

  const handleSetView = (view: View) => {
    if (view === 'dashboard' && currentView !== 'dashboard') {
      setSelectedOrg(null);
      setDashboardView('overview');
    }
    setCurrentView(view);
  };

  useEffect(() => {
    const checkHealth = async () => {
      try {
        const health = await getSystemHealth();
        if (health.status === 'ok' && health.database === 'healthy') {
          setSystemStatus('healthy');
        } else {
          setSystemStatus('unhealthy');
        }
      } catch (error) {
        console.error("Health check failed:", error);
        setSystemStatus('unhealthy');
      }
    };

    checkHealth(); // Initial check
    const intervalId = setInterval(checkHealth, 30000); // Check every 30 seconds

    return () => clearInterval(intervalId); // Cleanup
  }, []);

  useEffect(() => {
    if (popupNotification) {
      if (popupTimerRef.current) {
        clearTimeout(popupTimerRef.current);
      }
      popupTimerRef.current = setTimeout(() => {
        setPopupNotification(null);
      }, 5000);
    }
    return () => {
      if (popupTimerRef.current) {
        clearTimeout(popupTimerRef.current);
      }
    };
  }, [popupNotification]);

  const addNotification = (message: string, type: 'success' | 'error' = 'success', context?: Notification['context']) => {
    const newNotification: Notification = {
        id: Date.now(),
        message,
        type,
        read: false,
        context,
    };
    setNotifications(prev => [newNotification, ...prev].slice(0, 10)); // Keep last 10
    setPopupNotification(newNotification);
  };
  
  const handleNotificationClick = (notification: Notification) => {
    if (!notification.context || notification.type === 'error') return;

    setNotificationMenuOpen(false); // Close menu on click

    const { orgId } = notification.context;

    if (orgId) {
        const targetOrg = organizations.find(o => o.id === orgId);
        if (targetOrg) {
            handleSelectOrg(targetOrg);
        } else {
            fetchDashboardData().then(() => {
                const fetchedOrg = organizations.find(o => o.id === orgId);
                if (fetchedOrg) handleSelectOrg(fetchedOrg);
            });
        }
    }
  };

  const generateSlug = (name: string) => name.toLowerCase().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, '');

  const fetchDashboardData = useCallback(async () => {
    setLoading('dashboard-data');
    setError(null);
    try {
        const [orgsData, statsData] = await Promise.all([
            getOrganizations(token),
            getSystemStats()
        ]);
        setOrganizations(orgsData.items);
        setStats(statsData.tables);
    } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch dashboard data');
    } finally {
        setLoading('');
    }
  }, [token]);

  useEffect(() => { 
      if(currentView === 'dashboard' && dashboardView === 'overview') {
          fetchDashboardData(); 
      }
  }, [fetchDashboardData, currentView, dashboardView]);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
        if (userMenuRef.current && !userMenuRef.current.contains(event.target as Node)) {
            setUserMenuOpen(false);
        }
        if (notificationMenuRef.current && !notificationMenuRef.current.contains(event.target as Node)) {
            setNotificationMenuOpen(false);
        }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => {
        document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [userMenuRef, notificationMenuRef]);
  
  useEffect(() => {
    setSearchTerm('');
  }, [dashboardView]);

  const fetchRedisInstances = useCallback(async (orgId: string) => {
    const data = await getRedisInstances(token, orgId);
    setRedisInstances(data.items);
  }, [token]);

  const fetchApiKeys = useCallback(async (orgId: string) => {
    const data = await getApiKeys(token, orgId);
    setApiKeyCount(data.items.length);
  }, [token]);
  
  useEffect(() => {
      if (selectedOrg) { 
        setLoading('org-data');
        setError(null);
        Promise.all([fetchRedisInstances(selectedOrg.id), fetchApiKeys(selectedOrg.id)])
            .catch(err => setError(err instanceof Error ? err.message : 'Failed to fetch organization data'))
            .finally(() => setLoading(''));
      }
  }, [selectedOrg, fetchRedisInstances, fetchApiKeys]);

  useEffect(() => {
    if (!selectedOrg || dashboardView !== 'org-management' || currentView !== 'dashboard') {
      return;
    }

    const intervalId = setInterval(async () => {
      try {
        const data = await getRedisInstances(token, selectedOrg.id);
        setRedisInstances(data.items);
      } catch (err) { console.error("Polling error:", err); }
    }, 5000);

    return () => clearInterval(intervalId);
  }, [selectedOrg, dashboardView, token, currentView]);

  const handleSelectOrg = (org: Organization) => {
      setSelectedOrg(org);
      setAnimation({ class: 'slide-enter', key: Date.now() });
      setDashboardView('org-management');
  };

  const handleSelectInstance = (inst: RedisInstance) => {
      setSelectedInstance(inst);
      setAnimation({ class: 'slide-enter', key: Date.now() });
      setDashboardView('details');
  };

  const handleBack = () => {
      setAnimation({ class: 'slide-back-enter', key: Date.now() });
      if (dashboardView === 'details') {
          setSelectedInstance(null);
          setDashboardView('org-management');
      } else if (dashboardView === 'org-management') {
          setSelectedOrg(null);
          setRedisInstances([]);
          setApiKeyCount(0);
          setDashboardView('overview');
      }
  };

  const handleCreateOrg = async (e: React.FormEvent) => {
      e.preventDefault();
      setLoading('create-org');
      try {
          const newOrg = await createOrganization(token, { ...newOrgData, slug: newOrgData.slug || generateSlug(newOrgData.name) });
          setCreateOrgModalOpen(false);
          addNotification(`Organization '${newOrgData.name}' created.`, 'success', { orgId: newOrg.id });
          setNewOrgData({ name: '', description: '', slug: '' });
          fetchDashboardData();
      } catch (err) { 
        const message = err instanceof Error ? err.message : 'Failed to create organization';
        setError(message);
        addNotification(message, 'error');
      } 
      finally { setLoading(''); }
  };
  
  const handleEditOrg = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!orgToEdit) return;
    setLoading('edit-org');
    try {
        await updateOrganization(token, orgToEdit.id, { name: orgToEdit.name, description: orgToEdit.description, slug: orgToEdit.slug });
        setEditOrgModalOpen(false);
        setOrgToEdit(null);
        fetchDashboardData();
    } catch (err) { setError(err instanceof Error ? err.message : 'Failed to update organization'); }
    finally { setLoading(''); }
  };

  const handleDeleteOrg = async () => {
    if (!orgToDelete) return;
    setLoading('delete-org');
    setError(null);
    try {
        await deleteOrganization(token, orgToDelete.id);
        addNotification(`Organization '${orgToDelete.name}' deleted successfully.`);
        setOrgToDelete(null);
        fetchDashboardData();
        if (selectedOrg?.id === orgToDelete.id) {
            handleBack();
        }
    } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Failed to delete organization';
        if (errorMessage.includes('active Redis instances')) {
            setAlertModal({
                isOpen: true,
                title: 'Deletion Failed',
                message: 'This organization cannot be deleted because it has active Redis instances. Please remove all instances first.'
            });
        } else { 
          setError(errorMessage);
          addNotification(errorMessage, 'error');
        }
        setOrgToDelete(null);
    } finally { setLoading(''); }
  }

  const handleCreateRedis = async (e: React.FormEvent) => {
      e.preventDefault();
      if (!selectedOrg) return;
      setLoading('create-redis');
      try {
          const newInstance = await createRedisInstance(token, selectedOrg.id, { ...newRedisData, organization_id: selectedOrg.id });
          setCreateRedisModalOpen(false);
          addNotification(`Redis instance '${newRedisData.name}' created.`, 'success', { orgId: selectedOrg.id, instanceId: newInstance.id });
          setNewRedisData({ name: '', slug: '', max_memory: 104857600, redis_version: '7.2', persistence_enabled: true, backup_enabled: false });
          fetchRedisInstances(selectedOrg.id);
      } catch (err) { 
        const message = err instanceof Error ? err.message : 'Failed to create Redis instance';
        setError(message);
        addNotification(message, 'error');
       } 
      finally { setLoading(''); }
  };

  const handleDeleteRedis = async () => {
    if (!redisToDelete || !selectedOrg) return;
    setLoading('delete-redis');
    try {
        await deleteRedisInstance(token, selectedOrg.id, redisToDelete.id);
        addNotification(`Redis instance '${redisToDelete.name}' deleted successfully.`);
        setRedisToDelete(null);
        fetchRedisInstances(selectedOrg.id);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to delete Redis instance';
      setError(message); 
      addNotification(message, 'error');
    }
    finally { setLoading(''); }
  }
  
  const StatusBadge = ({ status }: { status: string }) => {
    const colorMap: { [key: string]: string } = {
        running: 'bg-green-100 text-green-800 border-green-200/60',
        pending: 'bg-yellow-100 text-yellow-800 border-yellow-200/60',
        stopped: 'bg-slate-100 text-slate-800 border-slate-200/60',
        error: 'bg-red-100 text-red-800 border-red-200/60',
    };
    return <span className={`px-2 py-1 text-xs font-medium rounded-full border ${colorMap[status] || colorMap['stopped']}`}>{status}</span>
  };

  const filteredOrganizations = useMemo(() => {
    if (!searchTerm || dashboardView !== 'overview') return organizations;
    return organizations.filter(org => org.name.toLowerCase().includes(searchTerm.toLowerCase()));
  }, [organizations, searchTerm, dashboardView]);

  const filteredRedisInstances = useMemo(() => {
      if (!searchTerm || dashboardView !== 'org-management') return redisInstances;
      return redisInstances.filter(inst => inst.name.toLowerCase().includes(searchTerm.toLowerCase()));
  }, [redisInstances, searchTerm, dashboardView]);

  const renderDashboardOverview = () => (
      <div>
          <div className="flex justify-between items-center mb-6">
              <h2 className="text-3xl font-bold text-slate-900">Overview</h2>
          </div>

          {loading === 'dashboard-data' && !stats ? <div className="flex justify-center p-8"><SpinnerIcon /></div> :
           stats ? (
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
                <div className="bg-white p-6 rounded-xl border border-slate-200 shadow-sm flex items-start justify-between">
                    <div>
                        <p className="text-sm font-medium text-slate-500">Total Organizations</p>
                        <p className="text-3xl font-bold text-slate-900">{stats.organizations.toLocaleString()}</p>
                    </div>
                    <div className="p-3 bg-teal-100 rounded-lg"><BuildingOfficeIcon className="h-6 w-6 text-teal-600" /></div>
                </div>
                 <div className="bg-white p-6 rounded-xl border border-slate-200 shadow-sm flex items-start justify-between">
                    <div>
                        <p className="text-sm font-medium text-slate-500">Total Redis Instances</p>
                        <p className="text-3xl font-bold text-slate-900">{stats.redis_instances.toLocaleString()}</p>
                    </div>
                    <div className="p-3 bg-cyan-100 rounded-lg"><DatabaseIcon className="h-6 w-6 text-cyan-600" /></div>
                </div>
                 <div className="bg-white p-6 rounded-xl border border-slate-200 shadow-sm flex items-start justify-between">
                    <div>
                        <p className="text-sm font-medium text-slate-500">Total API Keys</p>
                        <p className="text-3xl font-bold text-slate-900">{stats.api_keys.toLocaleString()}</p>
                    </div>
                    <div className="p-3 bg-green-100 rounded-lg"><ApiKeyIcon className="h-6 w-6 text-green-600" /></div>
                </div>
            </div>
           ) : null}

          <div className="flex justify-between items-center mb-6">
              <h2 className="text-2xl font-bold text-slate-800">Your Organizations</h2>
              <button onClick={() => setCreateOrgModalOpen(true)} className="flex items-center gap-2 bg-teal-600 hover:bg-teal-700 text-white font-bold py-2 px-4 rounded-lg transition-colors shadow-lg shadow-teal-500/30">
                  <PlusIcon /> Create
              </button>
          </div>
          <div className="mb-6 relative">
              <input type="search" placeholder="Search organizations..." value={searchTerm} onChange={(e) => setSearchTerm(e.target.value)} className="w-full pl-10 pr-4 py-2 rounded-lg border border-slate-300 bg-white focus:ring-2 focus:ring-teal-500 focus:border-teal-500 outline-none transition" />
              <div className="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none"><SearchIcon /></div>
          </div>
          {loading === 'dashboard-data' && organizations.length === 0 ? <div className="flex justify-center p-8"><SpinnerIcon /></div> :
           error && loading !== 'delete-org' ? <p className="text-red-700 bg-red-100 p-3 rounded-md">{error}</p> :
           organizations.length > 0 ? (
            <>
              {filteredOrganizations.length > 0 ? (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    {filteredOrganizations.map(org => (
                        <div key={org.id} className="bg-white border border-slate-200 p-6 rounded-xl shadow-md flex flex-col justify-between transition-all duration-300 hover:shadow-lg hover:border-teal-400 relative group">
                            <div className="absolute top-3 right-3 flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                                <button onClick={(e) => { e.stopPropagation(); setOrgToEdit(org); setEditOrgModalOpen(true); }} className="p-2 rounded-full bg-slate-100 hover:bg-teal-500 text-slate-500 hover:text-white"><EditIcon /></button>
                                <button onClick={(e) => { e.stopPropagation(); setOrgToDelete(org); }} className="p-2 rounded-full bg-slate-100 hover:bg-red-500 text-slate-500 hover:text-white"><DeleteIcon /></button>
                            </div>
                            <div onClick={() => handleSelectOrg(org)} className="cursor-pointer">
                                <div className="flex items-center gap-4 mb-3">
                                    <div className="p-3 bg-slate-100 rounded-lg"><BuildingOfficeIcon className="h-6 w-6 text-teal-600" /></div>
                                    <h3 className="text-xl font-semibold text-slate-800">{org.name}</h3>
                                </div>
                                <p className="text-slate-600 text-sm mb-4 h-10 overflow-hidden">{org.description}</p>
                                <div className="text-xs text-slate-500 border-t border-slate-200 pt-3">Plan: <span className="font-semibold text-teal-600">{org.plan}</span></div>
                            </div>
                        </div>
                    ))}
                </div>
              ) : (
                <div className="text-center py-10 bg-white rounded-lg border border-slate-200"><p className="text-slate-500">No organizations found matching your search.</p></div>
              )}
            </>
          ) : <div className="text-center py-10 bg-white rounded-lg border border-slate-200"><p className="text-slate-500">No organizations found. Let's create one!</p></div>}
      </div>
  );
  
  const renderOrgManagementView = () => (
      <div>
        <div className="flex justify-between items-center mb-6">
            <div className="flex items-center gap-4">
                <button onClick={handleBack} className="flex items-center gap-2 p-2 rounded-md hover:bg-slate-200 text-slate-500 hover:text-slate-900">
                    <ArrowLeftIcon />
                </button>
                <h2 className="text-xl md:text-3xl font-bold text-slate-900 truncate">{selectedOrg?.name} / Overview</h2>
            </div>
        </div>
        
        {/* Stats Section */}
        {loading === 'org-data' ? <div className="flex justify-center p-8"><SpinnerIcon /></div> :
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
            <div className="bg-white p-6 rounded-xl border border-slate-200 shadow-sm flex items-start justify-between">
                <div>
                    <p className="text-sm font-medium text-slate-500">Total Redis Instances</p>
                    <p className="text-3xl font-bold text-slate-900">{redisInstances.length}</p>
                </div>
                 <div className="p-3 bg-cyan-100 rounded-lg"><DatabaseIcon className="h-6 w-6 text-cyan-600" /></div>
            </div>
            <div className="bg-white p-6 rounded-xl border border-slate-200 shadow-sm flex items-start justify-between">
                 <div>
                    <p className="text-sm font-medium text-slate-500">Active API Keys</p>
                    <p className="text-3xl font-bold text-slate-900">{apiKeyCount}</p>
                </div>
                <div className="p-3 bg-green-100 rounded-lg"><ApiKeyIcon className="h-6 w-6 text-green-600" /></div>
            </div>
        </div>
        }
        
        {/* Instances Section */}
        <div className="flex justify-between items-center mb-6">
            <h2 className="text-2xl font-bold text-slate-800">Redis Instances</h2>
            <button onClick={() => setCreateRedisModalOpen(true)} className="flex items-center gap-2 bg-teal-600 hover:bg-teal-700 text-white font-bold py-2 px-4 rounded-lg transition-colors shadow-lg shadow-teal-500/30">
                <PlusIcon /> Create Instance
            </button>
        </div>
        <div className="mb-6 relative">
            <input type="search" placeholder="Search instances..." value={searchTerm} onChange={(e) => setSearchTerm(e.target.value)} className="w-full pl-10 pr-4 py-2 rounded-lg border border-slate-300 bg-white focus:ring-2 focus:ring-teal-500 focus:border-teal-500 outline-none transition" />
            <div className="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none"><SearchIcon /></div>
        </div>
        {loading === 'redis' ? <div className="flex justify-center p-8"><SpinnerIcon /></div> :
         error && loading !== 'delete-redis' ? <p className="text-red-700 bg-red-100 p-3 rounded-md">{error}</p> :
         redisInstances.length > 0 ? (
            filteredRedisInstances.length > 0 ? (
              <>
                {/* Mobile Card View */}
                <div className="grid grid-cols-1 gap-4 md:hidden">
                    {filteredRedisInstances.map(inst => (
                        <div key={inst.id} className="bg-white border border-slate-200 rounded-xl shadow-md p-4 space-y-3">
                            <div className="flex justify-between items-start">
                                <div onClick={() => handleSelectInstance(inst)} className="cursor-pointer flex items-center gap-3 font-semibold text-slate-800">
                                    <DatabaseIcon className="h-5 w-5 text-slate-500" />
                                    <span>{inst.name}</span>
                                </div>
                                <button onClick={(e) => { e.stopPropagation(); setRedisToDelete(inst); }} className="p-2 -mt-1 -mr-1 rounded-full bg-slate-100 hover:bg-red-500 text-slate-500 hover:text-white"><DeleteIcon /></button>
                            </div>
                            <div className="space-y-2 text-sm pl-8">
                                <div className="flex justify-between"><span className="font-medium text-slate-500">Status:</span> <StatusBadge status={inst.status} /></div>
                                <div className="flex justify-between items-start gap-2"><span className="font-medium text-slate-500 flex-shrink-0">Domain:</span> <span className="font-mono text-xs text-slate-600 text-right break-all">{inst.domain}</span></div>
                                <div className="flex justify-between"><span className="font-medium text-slate-500">Version:</span> <span className="text-slate-700">v{inst.redis_version}</span></div>
                                <div className="flex justify-between"><span className="font-medium text-slate-500">Memory:</span> <span className="text-slate-700">{(inst.max_memory / (1024 * 1024)).toFixed(0)} MB</span></div>
                            </div>
                        </div>
                    ))}
                </div>

                {/* Desktop Table View */}
                <div className="hidden md:block bg-white border border-slate-200 rounded-xl shadow-md overflow-hidden">
                    <div className="overflow-x-auto">
                        <table className="w-full text-left">
                            <thead className="bg-slate-50">
                                <tr>
                                    <th className="p-4 font-semibold text-slate-600">Name</th>
                                    <th className="p-4 font-semibold text-slate-600">Status</th>
                                    <th className="p-4 font-semibold text-slate-600">Domain</th>
                                    <th className="p-4 font-semibold text-slate-600">Version</th>
                                    <th className="p-4 font-semibold text-slate-600">Memory</th>
                                    <th className="p-4 font-semibold text-slate-600">Actions</th>
                                </tr>
                            </thead>
                            <tbody>
                                {filteredRedisInstances.map(inst => (
                                    <tr key={inst.id} className="border-t border-slate-200 hover:bg-slate-50 transition-colors cursor-pointer" onClick={() => handleSelectInstance(inst)}>
                                        <td className="p-4 flex items-center gap-3 text-slate-700"><DatabaseIcon className="h-5 w-5 text-slate-500" /> {inst.name}</td>
                                        <td className="p-4"><StatusBadge status={inst.status} /></td>
                                        <td className="p-4 font-mono text-sm text-slate-500 truncate max-w-xs">{inst.domain}</td>
                                        <td className="p-4 text-slate-700">v{inst.redis_version}</td>
                                        <td className="p-4 text-slate-700">{(inst.max_memory / (1024 * 1024)).toFixed(0)} MB</td>
                                        <td className="p-4"><button onClick={(e) => { e.stopPropagation(); setRedisToDelete(inst); }} className="p-2 rounded-full bg-slate-100 hover:bg-red-500 text-slate-500 hover:text-white"><DeleteIcon /></button></td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                </div>
              </>
            ) : (
                <div className="text-center py-10 bg-white rounded-lg border border-slate-200"><p className="text-slate-500">No instances found matching your search.</p></div>
            )
         ) : <div className="text-center py-10 bg-white rounded-lg border border-slate-200"><p className="text-slate-500">No Redis instances found. Create one to get started.</p></div>}
      </div>
  );
  
  const renderDashboardContent = () => {
    switch (dashboardView) {
        case 'overview':
            return renderDashboardOverview();
        case 'org-management':
            return renderOrgManagementView();
        case 'details':
            if (selectedOrg && selectedInstance) {
                return <InstanceDetailsView 
                        selectedInstance={selectedInstance}
                        selectedOrg={selectedOrg}
                        onBack={handleBack}
                      />;
            }
            return renderOrgManagementView();
        default:
             return renderDashboardOverview();
    }
  }
  
  const formFieldClasses = "w-full p-2 bg-slate-100 border border-slate-300 rounded-md text-slate-900 focus:ring-2 focus:ring-teal-500 focus:border-teal-500 outline-none transition";
  
  return (
    <div className="flex min-h-screen bg-slate-100 font-sans">
      {/* --- Popup Notification --- */}
      <div 
        className={`fixed top-8 right-8 z-[100] transition-all duration-300 ease-in-out ${popupNotification ? 'opacity-100 translate-x-0' : 'opacity-0 translate-x-10 pointer-events-none'}`}
      >
        {popupNotification && (
          <div className={`flex items-start gap-3 w-full max-w-sm p-4 rounded-lg shadow-2xl text-white ${popupNotification.type === 'success' ? 'bg-green-500' : 'bg-red-600'}`}>
            <div className="mt-0.5 h-5 w-5 flex-shrink-0 flex items-center justify-center rounded-full bg-white/20">
              {popupNotification.type === 'success' ? <CheckIcon/> : <WarningIcon/>}
            </div>
            <p className="text-sm font-semibold">{popupNotification.message}</p>
            <button onClick={() => setPopupNotification(null)} className="ml-auto -mr-1 -mt-1 p-1 rounded-full hover:bg-white/20">&times;</button>
          </div>
        )}
      </div>

      {/* --- Desktop Sidebar --- */}
      <aside className="w-64 bg-[#1C2434] text-gray-300 flex-col fixed h-full shadow-lg hidden md:flex">
          <SidebarContent 
            onLogout={onLogout}
            currentView={currentView}
            setView={handleSetView}
            systemStatus={systemStatus}
          />
      </aside>

      {/* --- Mobile Sidebar --- */}
      <div className={`md:hidden fixed inset-0 z-[99] transition-all duration-300 ease-in-out ${isMobileNavOpen ? 'pointer-events-auto' : 'pointer-events-none'}`}>
        {/* Backdrop */}
        <div className={`fixed inset-0 bg-black/60 transition-opacity ${isMobileNavOpen ? 'opacity-100' : 'opacity-0'}`} onClick={() => setMobileNavOpen(false)} />
        {/* Panel */}
        <div className={`fixed inset-y-0 left-0 z-[100] w-64 bg-[#1C2434] text-gray-300 shadow-xl transition-transform ease-in-out duration-300 ${isMobileNavOpen ? 'translate-x-0' : '-translate-x-full'}`}>
          <div className="flex h-full flex-col">
              <SidebarContent 
                onLogout={onLogout}
                currentView={currentView}
                setView={handleSetView}
                systemStatus={systemStatus}
                onLinkClick={() => setMobileNavOpen(false)}
              />
          </div>
        </div>
      </div>


      {/* --- Main Content --- */}
      <div className="flex-1 flex flex-col md:ml-64">
          <header className="bg-white/80 backdrop-blur-sm p-4 sticky top-0 z-40 border-b border-slate-200">
              <div className="flex justify-between items-center">
                  <div className="flex items-center gap-2">
                      <button onClick={() => setMobileNavOpen(true)} className="p-2 rounded-full text-slate-500 hover:text-slate-900 hover:bg-slate-100 md:hidden" aria-label="Open navigation">
                        <HamburgerIcon />
                      </button>
                      <div>
                        <h2 className="text-xl font-bold text-slate-800">Welcome back, {user.first_name}! 👋</h2>
                        <p className="text-sm text-slate-500">{new Date().toLocaleDateString('en-US', { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' })}</p>
                      </div>
                  </div>
                  <div className="flex items-center gap-4">
                      <div className="relative" ref={notificationMenuRef}>
                        <button onClick={() => setNotificationMenuOpen(prev => !prev)} className="p-2 rounded-full text-slate-500 hover:text-slate-900 hover:bg-slate-100 relative">
                            <BellIcon />
                            {notifications.filter(n => !n.read).length > 0 && 
                                <span className="absolute top-0 right-0 flex h-2 w-2">
                                  <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-red-400 opacity-75"></span>
                                  <span className="relative inline-flex rounded-full h-2 w-2 bg-red-500"></span>
                                </span>
                            }
                        </button>
                        {isNotificationMenuOpen && (
                          <div className="absolute right-0 mt-2 w-80 origin-top-right bg-white rounded-md shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none z-50 animate-fadeInUp" style={{animationDuration: '0.15s'}}>
                            <div className="p-3 border-b border-slate-200 flex justify-between items-center">
                                <h4 className="text-sm font-semibold text-slate-800">Notifications</h4>
                                <button onClick={() => setNotifications(prev => prev.map(n => ({...n, read: true})))} className="text-xs text-teal-600 hover:underline">Mark all as read</button>
                            </div>
                            <div className="py-1 max-h-80 overflow-y-auto">
                              {notifications.length > 0 ? notifications.map(n => (
                                <div 
                                  key={n.id} 
                                  onClick={() => handleNotificationClick(n)}
                                  className={`px-4 py-3 text-sm flex items-start gap-3 transition-colors ${!n.read ? 'bg-teal-50' : 'bg-white'} ${n.context && n.type !== 'error' ? 'cursor-pointer hover:bg-teal-100' : ''}`}
                                >
                                  <div className={`mt-1 h-5 w-5 flex-shrink-0 flex items-center justify-center rounded-full ${n.type === 'success' ? 'bg-green-100' : 'bg-red-100'}`}>
                                      {n.type === 'success' ? <CheckIcon/> : <WarningIcon/>}
                                  </div>
                                  <p className="text-slate-700">{n.message}</p>
                                </div>
                              )) : (
                                <p className="text-center text-slate-500 py-6 text-sm">No new notifications.</p>
                              )}
                            </div>
                          </div>
                        )}
                      </div>
                      <div className="relative" ref={userMenuRef}>
                          <button onClick={() => setUserMenuOpen(prev => !prev)} className="flex items-center gap-2 p-1 rounded-full hover:bg-slate-100 transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-teal-500">
                              <div className="w-9 h-9 rounded-full bg-gradient-to-tr from-cyan-500 to-teal-600 flex items-center justify-center text-white font-bold">
                                  {user.first_name.charAt(0)}{user.last_name.charAt(0)}
                              </div>
                          </button>
                          {isUserMenuOpen && (
                              <div className="absolute right-0 mt-2 w-56 origin-top-right bg-white rounded-md shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none z-50 animate-fadeInUp" style={{animationDuration: '0.15s'}}>
                                  <div className="py-1">
                                      <div className="px-4 py-2">
                                          <p className="text-sm font-medium text-slate-900 truncate">{user.first_name} {user.last_name}</p>
                                          <p className="text-sm text-slate-500 truncate">{user.email}</p>
                                      </div>
                                      <div className="border-t border-slate-200"></div>
                                      <button onClick={() => { onLogout(); setUserMenuOpen(false); }} className="w-full text-left flex items-center gap-3 px-4 py-3 text-sm text-slate-700 hover:bg-slate-100 hover:text-slate-900 transition-colors">
                                          <LogoutIcon />
                                          <span>Logout</span>
                                      </button>
                                  </div>
                              </div>
                          )}
                      </div>
                  </div>
              </div>
          </header>
          <main className="flex-grow p-4 sm:p-8">
            <div key={currentView} className="animate-fadeInUp">
                {currentView === 'dashboard' && (
                    <div key={animation.key} className={animation.class}>
                       {renderDashboardContent()}
                    </div>
                )}
                {currentView === 'docs' && <DocsPage />}
                {currentView === 'guides' && <UserGuidesPage onNavigate={(view) => setCurrentView(view)} />}
                {/* FIX: Add rendering for TermsOfServicePage when currentView is 'terms'. */}
                {currentView === 'terms' && <TermsOfServicePage />}
            </div>
          </main>
      </div>
      
      {/* --- Modals --- */}
      <Modal isOpen={isCreateOrgModalOpen} onClose={() => setCreateOrgModalOpen(false)} title="Create New Organization">
          <form onSubmit={handleCreateOrg} className="space-y-4">
              <input type="text" placeholder="Organization Name" value={newOrgData.name} onChange={e => setNewOrgData({...newOrgData, name: e.target.value})} className={formFieldClasses} required />
              <textarea placeholder="Description" value={newOrgData.description} onChange={e => setNewOrgData({...newOrgData, description: e.target.value})} className={formFieldClasses} rows={3}></textarea>
              <button type="submit" disabled={loading === 'create-org'} className="w-full bg-teal-600 hover:bg-teal-700 text-white font-bold py-2 px-4 rounded-lg disabled:bg-teal-400 transition-colors">
                  {loading === 'create-org' ? 'Creating...' : 'Create Organization'}
              </button>
              {error && loading === 'create-org' && <p className="text-sm text-red-700 text-center mt-2">{error}</p>}
          </form>
      </Modal>

      <Modal isOpen={isEditOrgModalOpen} onClose={() => setEditOrgModalOpen(false)} title="Edit Organization">
          <form onSubmit={handleEditOrg} className="space-y-4">
              <input type="text" placeholder="Organization Name" value={orgToEdit?.name || ''} onChange={e => setOrgToEdit(org => org ? {...org, name: e.target.value} : null)} className={formFieldClasses} required />
              <input type="text" placeholder="Slug" value={orgToEdit?.slug || ''} onChange={e => setOrgToEdit(org => org ? {...org, slug: e.target.value} : null)} className={formFieldClasses} required />
              <textarea placeholder="Description" value={orgToEdit?.description || ''} onChange={e => setOrgToEdit(org => org ? {...org, description: e.target.value} : null)} className={formFieldClasses} rows={3}></textarea>
              <button type="submit" disabled={loading === 'edit-org'} className="w-full bg-teal-600 hover:bg-teal-700 text-white font-bold py-2 px-4 rounded-lg disabled:bg-teal-400 transition-colors">
                  {loading === 'edit-org' ? 'Saving...' : 'Save Changes'}
              </button>
              {error && loading === 'edit-org' && <p className="text-sm text-red-700 text-center mt-2">{error}</p>}
          </form>
      </Modal>

      <ConfirmationModal isOpen={!!orgToDelete} onClose={() => setOrgToDelete(null)} onConfirm={handleDeleteOrg} title="Delete Organization" message={`Are you sure you want to delete "${orgToDelete?.name}"? This action cannot be undone.`} isLoading={loading === 'delete-org'} confirmText="Delete" />

      <Modal isOpen={isCreateRedisModalOpen} onClose={() => setCreateRedisModalOpen(false)} title="Create New Redis Instance">
          <form onSubmit={handleCreateRedis} className="space-y-4">
              <input type="text" placeholder="Instance Name" value={newRedisData.name} onChange={e => setNewRedisData({...newRedisData, name: e.target.value, slug: generateSlug(e.target.value) })} className={formFieldClasses} required />
              <input type="text" placeholder="Slug" value={newRedisData.slug} onChange={e => setNewRedisData({...newRedisData, slug: e.target.value})} className={formFieldClasses} required />
              <select value={newRedisData.redis_version} onChange={e => setNewRedisData({...newRedisData, redis_version: e.target.value})} className={formFieldClasses}>
                  <option value="7.2">Redis 7.2</option>
                  <option value="7.0">Redis 7.0</option>
                  <option value="6.2">Redis 6.2</option>
              </select>
              <input type="number" placeholder="Max Memory (MB)" value={newRedisData.max_memory / (1024*1024)} onChange={e => setNewRedisData({...newRedisData, max_memory: parseInt(e.target.value) * 1024 * 1024})} className={formFieldClasses} required />
              <div className="flex items-center justify-between pt-2">
                  <label htmlFor="persistence_enabled" className="text-slate-700">Enable Persistence</label>
                  <input type="checkbox" id="persistence_enabled" checked={newRedisData.persistence_enabled} onChange={e => setNewRedisData({...newRedisData, persistence_enabled: e.target.checked})} className="h-5 w-5 rounded bg-slate-100 border-slate-300 text-teal-600 focus:ring-teal-500 focus:ring-offset-white" />
              </div>
              <div className="flex items-center justify-between">
                  <label htmlFor="backup_enabled" className="text-slate-700">Enable Backups</label>
                  <input type="checkbox" id="backup_enabled" checked={newRedisData.backup_enabled} onChange={e => setNewRedisData({...newRedisData, backup_enabled: e.target.checked})} className="h-5 w-5 rounded bg-slate-100 border-slate-300 text-teal-600 focus:ring-teal-500 focus:ring-offset-white" />
              </div>
              <button type="submit" disabled={loading === 'create-redis'} className="w-full bg-teal-600 hover:bg-teal-700 text-white font-bold py-2 px-4 rounded-lg disabled:bg-teal-400 !mt-6">
                  {loading === 'create-redis' ? 'Creating...' : 'Create Instance'}
              </button>
              {error && loading === 'create-redis' && <p className="text-sm text-red-700 text-center mt-2">{error}</p>}
          </form>
      </Modal>

      <ConfirmationModal isOpen={!!redisToDelete} onClose={() => setRedisToDelete(null)} onConfirm={handleDeleteRedis} title="Delete Redis Instance" message={`Are you sure you want to delete "${redisToDelete?.name}"? This action is permanent.`} isLoading={loading === 'delete-redis'} confirmText="Delete" />
      
      <AlertModal
          isOpen={alertModal.isOpen}
          onClose={() => setAlertModal({ isOpen: false, title: '', message: '' })}
          title={alertModal.title}
          message={alertModal.message}
      />
    </div>
  );
};

export default Dashboard;