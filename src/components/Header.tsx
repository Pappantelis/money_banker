import type { User } from "../types";

interface HeaderProps {
  user: User;
  onLogout: () => void;
}

export function Header({ user, onLogout }: HeaderProps) {
  return (
    <header className="header">
      <div className="header-left">
        <h1>Monthly Bank Usage</h1>
      </div>

      <div className="header-right">
        <div className="user-info">
          {user.photo_url && (
            <img
              src={user.photo_url}
              alt={`${user.f_name} ${user.l_name}`}
              className="user-avatar"
            />
          )}
          <span className="user-name">{user.f_name} {user.l_name}</span>
        </div>
        <button className="logout-button" onClick={onLogout}>
          Logout
        </button>
      </div>
    </header>
  );
}
