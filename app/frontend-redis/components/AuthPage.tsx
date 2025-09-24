import React, { useState } from 'react';
import { login, register } from '../services/api';
import { User, AuthData } from '../types';
import { ArrowLeftIcon, LogoIcon, EyeIcon, EyeSlashIcon } from './ui/Icons';

interface AuthPageProps {
  onLoginSuccess: (token: string, user: User) => void;
  onBack: () => void;
  initialView: 'login' | 'register';
}

const AuthPage: React.FC<AuthPageProps> = ({ onLoginSuccess, onBack, initialView }) => {
  const [isLoginView, setIsLoginView] = useState(initialView === 'login');
  const [formData, setFormData] = useState({
    email: '',
    username: '',
    password: '',
    confirmPassword: '',
    first_name: '',
    last_name: '',
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);


  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setFormData({ ...formData, [e.target.name]: e.target.value });
    setError(null);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    // Enhanced client-side validation for registration
    if (!isLoginView) {
      if (formData.password.length < 8) {
        setError("Password must be at least 8 characters long.");
        setLoading(false);
        return;
      }
      if (formData.password !== formData.confirmPassword) {
        setError("Passwords do not match.");
        setLoading(false);
        return;
      }
    }

    try {
      if (isLoginView) {
        const { email, password } = formData;
        const data: AuthData = await login({ email, password });
        onLoginSuccess(data.token, data.user);
      } else {
        const { email, username, password, first_name, last_name } = formData;
        await register({ email, username, password, first_name, last_name });
        alert('Registration successful! Please log in.');
        setIsLoginView(true);
        setFormData({ ...formData, password: '', confirmPassword: ''});
      }
    } catch (err) {
      if (err instanceof Error) {
        const errorMessage = err.message;
        
        if (errorMessage.toLowerCase().includes('failed to fetch')) {
          setError('Could not connect to the server. Please check your network and try again.');
        } else if (errorMessage.includes("User already exists with this email or username")) {
            setError("An account with this email already exists.");
        } else if (errorMessage.includes("Validation error") && errorMessage.includes('"username"')) {
            const minMatch = errorMessage.match(/"min":\s*Number\((\d+)\)/);
            const maxMatch = errorMessage.match(/"max":\s*Number\((\d+)\)/);
            if (errorMessage.includes('"length"') && minMatch && maxMatch) {
                setError(`Username must be between ${minMatch[1]} and ${maxMatch[1]} characters.`);
            } else {
                setError("There is an issue with the username provided. Please check it and try again.");
            }
        } else {
          setError(errorMessage);
        }
      } else {
        setError('An unexpected error occurred.');
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex items-center justify-center min-h-screen bg-slate-100 p-4">
      <div className="w-full max-w-md p-8 space-y-6 bg-white rounded-2xl shadow-xl relative">
        <button onClick={onBack} className="absolute top-4 left-4 p-2 rounded-full hover:bg-slate-100 transition-colors" aria-label="Back to welcome page">
          <ArrowLeftIcon />
        </button>
        <div className="flex justify-center mb-4 pt-8">
            <LogoIcon />
        </div>
        <div className="text-center">
          <h2 className="text-3xl font-extrabold text-slate-900">
            {isLoginView ? 'Welcome Back' : 'Create Your Account'}
          </h2>
          <p className="mt-2 text-sm text-slate-600">
            {isLoginView ? 'Or ' : 'Already have an account? '}
            <button
              onClick={() => {
                setIsLoginView(!isLoginView);
                setError(null);
              }}
              className="font-medium text-teal-600 hover:text-teal-500 transition-colors"
            >
              {isLoginView ? 'create an account' : 'sign in'}
            </button>
          </p>
        </div>
        <form className="mt-8 space-y-6" onSubmit={handleSubmit}>
          {error && <p className="text-sm text-red-700 text-center bg-red-100 p-3 rounded-md">{error}</p>}
          <div className="space-y-4">
            {!isLoginView && (
              <>
                <div className="grid grid-cols-2 gap-4">
                  <input name="first_name" type="text" required value={formData.first_name} onChange={handleChange} className="input-field" placeholder="First Name"/>
                  <input name="last_name" type="text" required value={formData.last_name} onChange={handleChange} className="input-field" placeholder="Last Name"/>
                </div>
                <input name="username" type="text" required value={formData.username} onChange={handleChange} className="input-field" placeholder="Username"/>
              </>
            )}
            <input name="email" type="email" autoComplete="email" required value={formData.email} onChange={handleChange} className="input-field" placeholder="Email address"/>
            <div className="relative">
                <input name="password" type={showPassword ? "text" : "password"} autoComplete="current-password" required value={formData.password} onChange={handleChange} className="input-field pr-10" placeholder="Password"/>
                <button type="button" onClick={() => setShowPassword(!showPassword)} className="absolute inset-y-0 right-0 flex items-center pr-3 text-slate-500 hover:text-slate-700" aria-label={showPassword ? "Hide password" : "Show password"}>
                    {showPassword ? <EyeSlashIcon /> : <EyeIcon />}
                </button>
            </div>
            {!isLoginView && (
                 <div className="relative">
                    <input name="confirmPassword" type={showConfirmPassword ? "text" : "password"} required value={formData.confirmPassword} onChange={handleChange} className="input-field pr-10" placeholder="Confirm Password"/>
                     <button type="button" onClick={() => setShowConfirmPassword(!showConfirmPassword)} className="absolute inset-y-0 right-0 flex items-center pr-3 text-slate-500 hover:text-slate-700" aria-label={showConfirmPassword ? "Hide password" : "Show password"}>
                        {showConfirmPassword ? <EyeSlashIcon /> : <EyeIcon />}
                    </button>
                 </div>
            )}
          </div>

          <div>
            <button type="submit" disabled={loading} className="group relative w-full flex justify-center py-3 px-4 border border-transparent text-base font-semibold rounded-xl text-white bg-gradient-to-r from-cyan-500 to-teal-600 hover:shadow-xl transition-all duration-300 transform hover:scale-105 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-teal-500 disabled:opacity-50">
              {loading ? 'Processing...' : (isLoginView ? 'Sign In' : 'Register')}
            </button>
          </div>
        </form>
      </div>
      <style>{`
        .input-field {
            appearance: none;
            position: relative;
            display: block;
            width: 100%;
            padding: 0.85rem;
            border-radius: 0.75rem; /* rounded-xl */
            border: 1px solid #cbd5e1;
            background-color: #f1f5f9;
            color: #1e293b;
            outline: none;
            transition: border-color 0.2s, box-shadow 0.2s;
        }
        .input-field:focus {
            z-index: 10;
            border-color: #14b8a6; /* teal-500 */
            box-shadow: 0 0 0 2px rgba(20, 184, 166, 0.4); /* teal-500 with opacity */
        }
      `}</style>
    </div>
  );
};

export default AuthPage;
