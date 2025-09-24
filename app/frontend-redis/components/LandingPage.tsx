import React from 'react';
import { RocketIcon, BuildingOfficeIcon, DatabaseIcon } from './ui/Icons';

interface LandingPageProps {
  onGetStarted: () => void;
}

const LandingPage: React.FC<LandingPageProps> = ({ onGetStarted }) => {
  const features = [
    {
      icon: <BuildingOfficeIcon />,
      title: "Manage Organizations",
      description: "Easily create, edit, and manage all your organizations from a single, centralized dashboard.",
    },
    {
      icon: <DatabaseIcon />,
      title: "Deploy Redis Instances",
      description: "Spin up new Redis instances within your organizations in seconds with configurable settings.",
    },
    {
      icon: <RocketIcon />,
      title: "Real-time Monitoring",
      description: "Keep an eye on the status of your instances with live updates directly on your dashboard.",
    },
  ];

  return (
    <div>
      <div className="text-center py-16 sm:py-24 px-4">
        <h1 className="text-4xl sm:text-5xl md:text-6xl font-extrabold tracking-tight text-slate-900 mb-4">
          Welcome to RedisGate
        </h1>
        <p className="text-lg sm:text-xl text-slate-600 max-w-3xl mx-auto mb-8">
          The all-in-one solution for managing your organizations and Redis instances.
          Streamline your workflow, monitor status in real-time, and scale with confidence.
        </p>
        <button
          onClick={onGetStarted}
          className="bg-indigo-600 hover:bg-indigo-700 text-white font-bold py-3 px-8 rounded-lg transition-transform transform hover:scale-105 shadow-lg shadow-indigo-500/30 text-lg"
        >
          Go to Dashboard
        </button>
      </div>

      <div className="py-16 sm:py-24 bg-white rounded-2xl shadow-lg border border-slate-200">
        <div className="container mx-auto px-4">
          <h2 className="text-3xl font-bold text-center text-slate-900 mb-12">
            Powerful Features at Your Fingertips
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8 text-center">
            {features.map((feature, index) => (
              <div key={index} className="p-6">
                <div className="flex items-center justify-center h-16 w-16 rounded-xl bg-indigo-100 mx-auto mb-5">
                  {feature.icon}
                </div>
                <h3 className="text-xl font-semibold text-slate-800 mb-2">{feature.title}</h3>
                <p className="text-slate-600">{feature.description}</p>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};

export default LandingPage;