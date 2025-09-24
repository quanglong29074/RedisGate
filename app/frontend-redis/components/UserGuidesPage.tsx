import React from 'react';

const GuideSection: React.FC<{ title: string; children: React.ReactNode }> = ({ title, children }) => (
    <section className="mb-10">
        <h2 className="text-2xl font-bold text-slate-800 border-b border-slate-200 pb-2 mb-4">{title}</h2>
        <div className="prose prose-slate max-w-none">
            {children}
        </div>
    </section>
);

interface UserGuidesPageProps {
    onNavigate: (view: 'dashboard' | 'docs' | 'guides' | 'terms') => void;
}

const UserGuidesPage: React.FC<UserGuidesPageProps> = ({ onNavigate }) => {
    return (
        <div className="bg-white p-6 sm:p-10 rounded-xl shadow-md border border-slate-200">
            <div className="mb-8 text-center">
                <h1 className="text-4xl font-extrabold text-slate-900">User Guides</h1>
                <p className="mt-2 text-lg text-slate-600">Your step-by-step guide to mastering RedisGate.</p>
            </div>
            
            <GuideSection title="1. Getting Started: Account Setup">
                <p>Welcome to RedisGate! Your first step is to create an account to access all the features.</p>
                <ol>
                    <li><strong>Registration:</strong> Navigate to the main page. If you're not logged in, you'll see the authentication form. Click on "create an account". Fill in your first name, last name, username, email, and a secure password. Click "Register".</li>
                    <li><strong>Login:</strong> After successful registration, you'll be prompted to log in. Use the email and password you just created to sign in to your new account.</li>
                </ol>
            </GuideSection>

            <GuideSection title="2. Managing Organizations">
                <p>Organizations are containers for your projects and Redis instances. You can belong to multiple organizations and switch between them.</p>
                <ul>
                    <li><strong>Creating an Organization:</strong> Once logged in, you'll see the "Organizations" dashboard. Click the "+ Create" button. A modal will appear asking for an Organization Name and a Description. Fill these in and click "Create Organization".</li>
                    <li><strong>Viewing an Organization:</strong> Your newly created organization will appear as a card on the dashboard. Click anywhere on the card to view its Redis instances.</li>
                    <li><strong>Editing and Deleting:</strong> Hover over an organization card to reveal the edit (pencil) and delete (trash) icons. Use these to update the organization's details or to permanently remove it. <strong>Note:</strong> You cannot delete an organization that has active Redis instances.</li>
                </ul>
            </GuideSection>

            <GuideSection title="3. Deploying Redis Instances">
                <p>Inside an organization, you can create and manage multiple Redis instances.</p>
                <ol>
                    <li><strong>Navigate to Instances:</strong> Click on an organization card to go to its "Instances" view.</li>
                    <li><strong>Create a Redis Instance:</strong> Click the "+ Create" button. A form will appear allowing you to configure your new instance. You'll need to provide a Name, Version, Memory limit, and other settings.</li>
                    <li><strong>Monitoring Status:</strong> After creation, the instance will appear in the list with a "pending" status. The dashboard will automatically refresh, and the status will change to "running" once the instance is ready.</li>
                    <li><strong>Viewing Instance Details & API Guide:</strong> Click on any instance in the list to navigate to its detailed view. This page provides a handy API guide with pre-filled URLs for interacting with your Redis instance directly.</li>
                    <li><strong>Deleting an Instance:</strong> In the instance list, click the trash icon at the end of the row for the instance you wish to remove. Confirm the action in the pop-up modal. This action is permanent.</li>
                </ol>
            </GuideSection>

             <GuideSection title="4. Using the API">
                <p>The platform is fully API-driven. You can automate and manage your resources programmatically.</p>
                <ul>
                    <li><strong>Authentication:</strong> Most API routes are protected. Management routes (like creating organizations or instances) require a JWT Bearer token obtained upon login. Direct Redis commands require an API key specific to an organization.</li>
                    <li><strong>Finding Endpoints:</strong> A complete list of available endpoints and their functions can be found on the <a href="#" onClick={(e) => { e.preventDefault(); onNavigate('docs'); }} className="font-medium text-teal-600 hover:underline">Docs</a> page.</li>
                    <li><strong>Interacting with Redis:</strong> The easiest way to start is by navigating to an instance's detail page, which provides copy-paste ready URLs for common Redis commands like <code>GET</code>, <code>SET</code>, and <code>DEL</code>.</li>
                </ul>
            </GuideSection>
            <style>{`
                .prose ol, .prose ul { padding-left: 1.5rem; }
                .prose li { margin-bottom: 0.5rem; }
                .prose strong { color: #1e293b; }
                .prose a { color: #0d9488; text-decoration: none; }
                .prose a:hover { text-decoration: underline; }
            `}</style>
        </div>
    );
};

export default UserGuidesPage;
