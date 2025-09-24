import React from 'react';

const MethodBadge: React.FC<{ method: string }> = ({ method }) => {
    const colors: { [key: string]: string } = {
        'GET': 'bg-sky-100 text-sky-800', 'POST': 'bg-green-100 text-green-800',
        'PUT': 'bg-yellow-100 text-yellow-800', 'DELETE': 'bg-red-100 text-red-800',
    };
    return <span className={`font-mono text-xs font-bold mr-2 px-2.5 py-1 rounded-md ${colors[method] || 'bg-slate-100 text-slate-800'}`}>{method}</span>
}

const DocsPage: React.FC = () => {
    const allEndpoints = [
        {
            category: 'Public Routes',
            description: 'No authentication required.',
            endpoints: [
                { method: 'GET', path: '/health', description: 'Performs a system health check.' },
                { method: 'GET', path: '/version', description: 'Gets the current version of the system.' },
                { method: 'GET', path: '/stats', description: 'Retrieves database statistics.' },
                { method: 'POST', path: '/auth/register', description: 'Registers a new user account.' },
                { method: 'POST', path: '/auth/login', description: 'Logs in a user and returns a JWT token.' },
            ]
        },
        {
            category: 'Organizations',
            description: 'Requires user authentication (JWT Bearer Token).',
            endpoints: [
                { method: 'POST', path: '/api/organizations', description: 'Creates a new organization.' },
                { method: 'GET', path: '/api/organizations', description: 'Lists all accessible organizations.' },
                { method: 'GET', path: '/api/organizations/:org_id', description: 'Gets details for a specific organization.' },
                { method: 'PUT', path: '/api/organizations/:org_id', description: 'Updates a specific organization.' },
                { method: 'DELETE', path: '/api/organizations/:org_id', description: 'Deletes a specific organization.' },
            ]
        },
        {
            category: 'API Keys',
            description: 'Requires user authentication (JWT Bearer Token).',
            endpoints: [
                { method: 'POST', path: '/api/organizations/:org_id/api-keys', description: 'Creates a new API key for an organization.' },
                { method: 'GET', path: '/api/organizations/:org_id/api-keys', description: 'Lists all API keys for an organization.' },
                { method: 'GET', path: '/api/organizations/:org_id/api-keys/:key_id', description: 'Gets details for a specific API key.' },
                { method: 'DELETE', path: '/api/organizations/:org_id/api-keys/:key_id', description: 'Revokes a specific API key.' },
            ]
        },
        {
            category: 'Redis Instances',
            description: 'Requires user authentication (JWT Bearer Token).',
            endpoints: [
                { method: 'POST', path: '/api/organizations/:org_id/redis-instances', description: 'Creates a new Redis instance.' },
                { method: 'GET', path: '/api/organizations/:org_id/redis-instances', description: 'Lists all Redis instances in an organization.' },
                { method: 'GET', path: '/api/organizations/:org_id/redis-instances/:instance_id', description: 'Gets details for a specific Redis instance.' },
                { method: 'PUT', path: '/api/organizations/:org_id/redis-instances/:instance_id/status', description: 'Updates the status of a Redis instance.' },
                { method: 'DELETE', path: '/api/organizations/:org_id/redis-instances/:instance_id', description: 'Deletes a specific Redis instance.' },
            ]
        },
        {
            category: 'Redis HTTP API',
            description: 'Requires API key authentication.',
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
                { method: 'POST', path: '/redis/:instance_id', description: 'Executes a generic Redis command via JSON body.' },
                { method: 'GET', path: '/redis/:instance_id/*path', description: 'Catch-all route for debugging Redis requests.' },
            ]
        },
    ];

    return (
        <div>
            <div className="mb-8">
                <h1 className="text-4xl font-extrabold text-slate-900">API Documentation</h1>
                <p className="mt-2 text-lg text-slate-600">Detailed information about our API endpoints.</p>
            </div>
            
            {allEndpoints.map(category => (
                <div key={category.category} className="mb-10">
                    <h2 className="text-2xl font-bold text-slate-800 mb-2">{category.category}</h2>
                    <p className="text-slate-500 mb-4 border-b border-slate-200 pb-3">{category.description}</p>
                    <div className="space-y-4">
                        {category.endpoints.map(ep => (
                            <div key={ep.path} className="bg-white border border-slate-200 rounded-lg p-4 shadow-sm">
                                <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between">
                                    <div className="flex items-center mb-2 sm:mb-0">
                                        <MethodBadge method={ep.method} />
                                        <span className="font-mono text-sm sm:text-base text-slate-800 break-all">{ep.path}</span>
                                    </div>
                                    <p className="text-sm text-slate-600 sm:text-right flex-shrink-0 sm:ml-4">{ep.description}</p>
                                </div>
                            </div>
                        ))}
                    </div>
                </div>
            ))}
        </div>
    );
};

export default DocsPage;