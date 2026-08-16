import React, { useState, useMemo } from "react";

interface SessionRecord {
  id: string;
  title: string;
  tags: string[];
  durationMs: number;
  date: string;
}

const DUMMY_SESSIONS: SessionRecord[] = [
  { id: "1", title: "React Hooks Deep Dive", tags: ["frontend", "react"], durationMs: 4500000, date: "2026-08-15" },
  { id: "2", title: "Rust Ownership Model", tags: ["backend", "rust"], durationMs: 3600000, date: "2026-08-16" },
  { id: "3", title: "System Design Prep", tags: ["architecture"], durationMs: 7200000, date: "2026-08-17" },
];

export const MediaLibrary: React.FC = () => {
  const [sessions, setSessions] = useState<SessionRecord[]>(DUMMY_SESSIONS);
  const [searchTerm, setSearchTerm] = useState("");

  const filteredSessions = useMemo(() => {
    return sessions.filter((s) => s.title.toLowerCase().includes(searchTerm.toLowerCase()));
  }, [sessions, searchTerm]);

  const formatTime = (ms: number) => {
    const totalSeconds = Math.floor(ms / 1000);
    const m = Math.floor(totalSeconds / 60);
    const s = totalSeconds % 60;
    return `${m}:${s.toString().padStart(2, "0")}`;
  };

  return (
    <div className="w-full flex flex-col h-full bg-gray-800 rounded-xl border border-gray-700 overflow-hidden">
      <div className="p-4 border-b border-gray-700 flex justify-between items-center bg-gray-900/50">
        <h2 className="text-xl font-semibold">Media Library</h2>
        <input
          type="text"
          placeholder="Search sessions..."
          className="px-3 py-1 bg-gray-700 border border-gray-600 rounded text-sm focus:outline-none focus:border-blue-500"
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
        />
      </div>
      <div className="p-4 overflow-y-auto max-h-[400px] flex flex-col gap-3">
        {filteredSessions.length > 0 ? (
          filteredSessions.map((session) => (
            <div key={session.id} className="p-3 bg-gray-700/50 rounded-lg hover:bg-gray-700 cursor-pointer transition-colors flex justify-between items-center">
              <div>
                <h3 className="font-medium text-white">{session.title}</h3>
                <div className="flex gap-2 mt-1">
                  {session.tags.map(tag => (
                    <span key={tag} className="text-xs px-2 py-0.5 bg-blue-900/50 text-blue-300 rounded">
                      {tag}
                    </span>
                  ))}
                </div>
              </div>
              <div className="text-right">
                <div className="text-sm text-gray-300">{formatTime(session.durationMs)}</div>
                <div className="text-xs text-gray-500">{session.date}</div>
              </div>
            </div>
          ))
        ) : (
          <div className="text-center text-gray-500 py-8">No sessions found</div>
        )}
      </div>
    </div>
  );
};
