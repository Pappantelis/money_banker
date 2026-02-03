import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Header } from "./components/Header";
import { Dashboard } from "./components/Dashboard";
import { TransactionList } from "./components/TransactionList";
import { AddTransaction } from "./components/AddTransaction";
import type { User, Transaction, Category, MonthlySummary, CreateTransaction } from "./types";

function App() {
  const [user, setUser] = useState<User | null>(null);
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [summary, setSummary] = useState<MonthlySummary | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [showAddModal, setShowAddModal] = useState(false);
  const [selectedDate, setSelectedDate] = useState(() => {
    const now = new Date();
    return { year: now.getFullYear(), month: now.getMonth() + 1 };
  });

  // Load initial data
  useEffect(() => {
    loadUserData();
  }, []);

  // Load transactions when date changes
  useEffect(() => {
    if (user) {
      loadTransactions();
      loadSummary();
    }
  }, [user, selectedDate]);

  async function loadUserData() {
    try {
      setIsLoading(true);
      const userData = await invoke<User>("get_current_user");
      setUser(userData);
      const cats = await invoke<Category[]>("get_categories");
      setCategories(cats);
    } catch (error) {
      console.error("Failed to load user:", error);
    } finally {
      setIsLoading(false);
    }
  }

  async function loadTransactions() {
    try {
      const txs = await invoke<Transaction[]>("get_transactions", {
        filter: {
          year: selectedDate.year,
          month: selectedDate.month,
        },
      });
      setTransactions(txs);
    } catch (error) {
      console.error("Failed to load transactions:", error);
    }
  }

  async function loadSummary() {
    try {
      const sum = await invoke<MonthlySummary>("get_monthly_summary", {
        year: selectedDate.year,
        month: selectedDate.month,
      });
      setSummary(sum);
    } catch (error) {
      console.error("Failed to load summary:", error);
    }
  }

  async function handleAddTransaction(tx: CreateTransaction) {
    try {
      await invoke("add_transaction", { transaction: tx });
      await loadTransactions();
      await loadSummary();
      setShowAddModal(false);
    } catch (error) {
      console.error("Failed to add transaction:", error);
      throw error;
    }
  }

  async function handleLogout() {
    try {
      await invoke("logout");
      setUser(null);
      setTransactions([]);
      setSummary(null);
    } catch (error) {
      console.error("Failed to logout:", error);
    }
  }

  async function handleLogin() {
    try {
      setIsLoading(true);
      await invoke("login");
      await loadUserData();
    } catch (error) {
      console.error("Failed to login:", error);
    } finally {
      setIsLoading(false);
    }
  }

  function handleDateChange(year: number, month: number) {
    setSelectedDate({ year, month });
  }

  if (isLoading) {
    return (
      <div className="app loading">
        <div className="spinner"></div>
        <p>Loading...</p>
      </div>
    );
  }

  if (!user) {
    return (
      <div className="app login-screen">
        <div className="login-container">
          <h1>Monthly Bank Usage</h1>
          <p>Track your expenses and income</p>
          <button className="login-button" onClick={handleLogin}>
            <svg viewBox="0 0 24 24" width="24" height="24">
              <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
              <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
              <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>
              <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
            </svg>
            Sign in with Google
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="app">
      <Header user={user} onLogout={handleLogout} />

      <main className="main-content">
        <Dashboard
          summary={summary}
          selectedDate={selectedDate}
          onDateChange={handleDateChange}
          onAddClick={() => setShowAddModal(true)}
        />

        <TransactionList
          transactions={transactions}
          categories={categories}
        />
      </main>

      {showAddModal && (
        <AddTransaction
          categories={categories}
          onSubmit={handleAddTransaction}
          onClose={() => setShowAddModal(false)}
        />
      )}
    </div>
  );
}

export default App;
