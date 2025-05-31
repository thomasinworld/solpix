'use client';

import { useState } from 'react';

interface UsernameClaimModalProps {
  isOpen: boolean;
  onClose: () => void;
  onClaim: (username: string) => void;
}

const takenUsernames = ['solpix', 'admin', 'test']; // Simulate taken usernames

export default function UsernameClaimModal({ isOpen, onClose, onClaim }: UsernameClaimModalProps) {
  const [username, setUsername] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  if (!isOpen) return null;

  const validate = (name: string) => {
    if (!/^[a-zA-Z0-9_]{3,16}$/.test(name)) {
      return 'Username must be 3-16 characters, alphanumeric or underscores.';
    }
    if (takenUsernames.includes(name.toLowerCase())) {
      return 'This username is already taken.';
    }
    return '';
  };

  const handleClaim = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    const err = validate(username);
    if (err) {
      setError(err);
      return;
    }
    setLoading(true);
    setTimeout(() => {
      setLoading(false);
      onClaim(username);
      onClose();
    }, 1000);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="bg-white/10 border border-white/20 rounded-2xl shadow-2xl p-8 w-full max-w-md flex flex-col items-center">
        <h2 className="text-2xl font-bold text-white mb-2">Claim your solpix username</h2>
        <p className="text-gray-200 mb-4 text-center">This will be your public identity on solpix.</p>
        <form onSubmit={handleClaim} className="w-full flex flex-col items-center gap-4">
          <input
            type="text"
            value={username}
            onChange={e => setUsername(e.target.value)}
            className="w-full rounded-lg px-4 py-3 bg-white/20 text-white placeholder-gray-300 focus:outline-none focus:ring-2 focus:ring-purple-400 text-lg font-semibold"
            placeholder="Choose a username"
            maxLength={16}
            minLength={3}
            autoFocus
            required
          />
          {error && <div className="text-red-400 text-sm font-medium">{error}</div>}
          <button
            type="submit"
            disabled={loading}
            className="w-full bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg py-3 text-lg font-bold text-white shadow-lg hover:from-pink-500 hover:to-purple-500 transition-all duration-200 disabled:opacity-60"
          >
            {loading ? 'Claiming...' : 'Claim Username'}
          </button>
        </form>
      </div>
    </div>
  );
} 